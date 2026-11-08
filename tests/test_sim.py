"""Checks that should hold however the model is tuned.

Run with:  python -m pytest -q
The short runs below take about half a minute in total.
"""
import json

import numpy as np
import pytest

from isostate import default_params, init_state, record, run
from isostate.econ import les, service_mix


@pytest.fixture(scope="module")
def short_runs():
    P = default_params(seed=3, end_year=1756)
    return run(P, progress=False, check_money=True), run(P, progress=False, check_money=True)


def test_money_is_conserved_and_runs_are_deterministic(short_runs):
    # run() asserts money_total == money_start + issued every month; reaching here means it held
    a, b = short_runs
    for name, series in a["state"]["series"].items():
        np.testing.assert_array_equal(series, b["state"]["series"][name], err_msg=name)
    assert [e["title"] for e in a["events"]] == [e["title"] for e in b["events"]]


def test_rent_falls_with_distance_to_market():
    st, C, _ = init_state(default_params(seed=5))
    land = st["world"]["land"].ravel() & (st["catch"] >= 0) & ~st["urban"]
    rent = st["land_rent_only"][land]
    dist = st["dcatch"][land]
    ok = np.isfinite(dist)
    # Spearman rank correlation without scipy.stats
    r1 = np.argsort(np.argsort(rent[ok]))
    r2 = np.argsort(np.argsort(dist[ok]))
    rho = np.corrcoef(r1, r2)[0, 1]
    assert rho < -0.3, f"von Thunen gradient missing: rank correlation {rho:.2f}"


def test_no_hard_walls_in_household_demand():
    # one town, three goods that can all serve one need at different efficiencies
    props = np.array([[1.0], [0.35], [3.0]])
    price = np.array([[1.0, 1.0, 40.0]])
    share, unit = service_mix(price, props, theta=0.25)
    assert np.all(share[0, :, 0] > 0), "every good with the property should get some share"
    assert share[0, :, 0].sum() == pytest.approx(1.0)
    # the expensive but efficient good still loses to the cheap one per unit of service
    assert share[0, 0, 0] > share[0, 2, 0]


def test_linear_expenditure_spends_the_budget():
    unit = np.array([[1.0, 2.0, 0.5]])
    gamma = np.array([0.5, 0.1, 0.2])
    beta = np.array([0.5, 0.3, 0.2])
    q, sub = les(np.array([10.0]), unit, gamma, beta)
    assert (q * unit).sum() == pytest.approx(10.0)
    poor, _ = les(np.array([0.3]), unit, gamma, beta)
    assert (poor * unit).sum() == pytest.approx(0.3)


def test_pack_round_trip(short_runs):
    res = short_runs[0]
    blob = record.pack(res, label="test")
    hlen = int(np.frombuffer(blob[:4], np.uint32)[0])
    assert (4 + hlen) % 8 == 0
    header = json.loads(blob[4:4 + hlen])
    arrays = {a["name"]: a for a in header["arrays"]}
    base = 4 + hlen
    for name in ("use", "mix_a", "mix_s", "series", "firm_Kvals"):
        a = arrays[name]
        assert a["offset"] % 8 == 0
        got = np.frombuffer(blob, dtype=a["dtype"], count=a["nbytes"] // np.dtype(a["dtype"]).itemsize,
                            offset=base + a["offset"]).reshape(a["shape"])
        if name in res["rec"]:
            np.testing.assert_array_equal(got, res["rec"][name])
    n_years = header["meta"]["n_years"]
    assert arrays["use"]["shape"] == [n_years, res["state"]["N"]]
    lite = record.pack(res, label="lite", lite=True)
    lh = json.loads(lite[4:4 + int(np.frombuffer(lite[:4], np.uint32)[0])])
    assert lh["meta"]["lite"] and not lh["firms"]
    assert "use" not in {a["name"] for a in lh["arrays"]}


def test_display_mix_is_well_formed(short_runs):
    rec = short_runs[0]["rec"]
    a, b, s = rec["mix_a"], rec["mix_b"], rec["mix_s"]
    assert a.max() < len(record.DISP) and b.max() < len(record.DISP)
    # the first class is the larger of the two
    assert s.min() >= 127
