"""Knowledge: discovery from use, diffusion between towns, learning by doing.

Nothing is unlocked by a tree. A method becomes discoverable once the economy
has handled enough of the goods it combines (Weitzman-style recombination),
appears first in the town that handles most of them, and spreads along trade
routes. Its productivity then climbs a learning curve with cumulative output.
"""
import numpy as np

from .names import surname

FIRST_NAMES = ["Abraham", "Benjamin", "Caleb", "Daniel", "Edmund", "Ezra", "George", "Hannah",
               "Isaac", "Jedediah", "John", "Josiah", "Lydia", "Martha", "Matthew", "Nathaniel",
               "Obadiah", "Richard", "Samuel", "Sarah", "Thomas", "William", "Zachariah", "Eliza"]


def discovery(st, C, P, rng, year, events):
    found = []
    for t in range(C["NT"]):
        if st["tech_year"][t] >= 0:
            continue
        X0 = st["disc_X0"][t]
        req = X0 > 0
        if not req.any():
            continue
        progress = max((st["cumprod"][req] / X0[req]).min(), 0.0)
        hz = P["discovery_rate"] * progress ** 4 / (1.0 + progress ** 4)
        if rng.random() < hz:
            act = np.flatnonzero(st["town_active"])
            score = (st["town_prod_year"][act][:, req] / X0[req][None, :]).sum(axis=1)
            # the likeliest place is the one handling most of the ingredients,
            # but any town that handles them at all has a chance
            w = (score + 1e-9 * st["pop"][act]) ** 2
            i = int(act[rng.choice(len(act), p=w / w.sum())])
            st["know"][i, t] = max(st["know"][i, t], 0.25)
            st["tech_year"][t] = year
            st["tech_town"][t] = i
            who = f"{rng.choice(FIRST_NAMES)} {surname(rng)}"
            town = st["town_name"][i]
            events.append(dict(
                year=year + 0.5, kind="discovery", clock="live", weight=0.9,
                title=C["tech_headline"][t].format(town=town, who=who),
                text=C["tech_story"][t].format(town=town, who=who),
                town=i, cell=int(st["town_node"][i]), tech=t,
            ))
            found.append(t)
    return found


def diffusion(st, C, P):
    act = st["town_active"]
    c = st["town_cost"]
    Wm = np.where(np.isfinite(c), np.exp(-c / P["diffusion_dist"]), 0.0)
    np.fill_diagonal(Wm, 0.0)
    Wm[~act] = 0.0
    Wm[:, ~act] = 0.0
    rs = Wm.sum(axis=1, keepdims=True)
    Wn = np.where(rs > 0, Wm / np.maximum(rs, 1e-12), 0.0) * np.minimum(rs, 1.0)
    k = st["know"]
    discovered = (st["tech_year"] >= 0) | C["tech_known"]
    contact = Wn @ k
    dk = P["diffusion_rate"] * (1 - k) * (k + 0.7 * contact)
    k_new = np.clip(k + dk * discovered[None, :] * act[:, None], 0.0, 1.0)
    st["know"] = k_new
