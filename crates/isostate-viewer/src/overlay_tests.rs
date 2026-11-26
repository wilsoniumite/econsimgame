use super::*;
use crate::replay::{Good, Header, Meta, Town};

/// A strip of four squares in one year: sea, then two squares that trade at
/// Ashby (which stands on the first of them), then one that trades at Brill.
fn strip() -> Replay {
    let town = |name: &str, node: usize| Town { name: name.into(), node, founded: 1700 };
    let good = |name: &str, label: &str| Good { name: name.into(), label: label.into(), kind: None, weight: None, price: None, begins: None };
    let header = Header {
        meta: Meta { label: String::new(), lite: false, seed: 1, w: 4, h: 1, start_year: 1750, n_years: 1, town_bytes: 0 },
        goods: vec![good("grain", "Grain"), good("coal", "Coal")],
        methods: Vec::new(),
        towns: vec![town("Ashby", 1), town("Brill", 3)],
        firms: Vec::new(),
        events: Vec::new(),
        disp: Vec::new(),
        uses: Vec::new(),
        arrays: Vec::new(),
        series: Default::default(),
        land: None,
        crossings: Vec::new(),
        techs: Vec::new(),
    };
    Replay {
        header,
        w: 4,
        h: 1,
        n: 4,
        ny: 1,
        y0: 1750,
        n_towns: 2,
        elev: vec![0; 4],
        terrain: vec![0, 1, 1, 2],
        river: vec![0; 4],
        down: vec![-1; 4],
        use_: vec![0; 4],
        inten: vec![0; 4],
        trans: vec![0; 4],
        catch: vec![0, 1, 1, 2],
        // a month's rent of 0.001, 1 and 1000 on the record's scale; none for the sea
        rent: vec![0, 1, 128, 255],
        // quarter units of path cost: at the market, 3 units away, and no way at all
        dist: vec![0, 0, 12, 255],
        mix_a: vec![0; 4],
        mix_b: vec![0; 4],
        mix_s: vec![255; 4],
        town_active: vec![1, 1],
        town_pop: vec![100.0, 50.0],
        town_urban: vec![10.0, 5.0],
        town_wage: vec![2.0, 3.0],
        // grain in both towns; coal only at Brill
        town_price: vec![1.0, 0.0, 1.5, 0.8],
        firm_first: Vec::new(),
        firm_len: Vec::new(),
        firm_off: Vec::new(),
        firm_k: Vec::new(),
        series_names: Vec::new(),
        series: Vec::new(),
        series_len: 12,
        filled: 1,
        owner: Vec::new(),
        built: Vec::new(),
        res: Vec::new(),
        works: Vec::new(),
        workings: Vec::new(),
        rent_scale: Vec::new(),
        builds: Default::default(),
        land: None,
        holds: Default::default(),
        traffic: Default::default(),
    }
}

/// The strip run for `ny` years, each year the first's, with what the ground
/// holds and what each town made: grain grown on farmland (the second square
/// twice as fertile as the third, which is half farmland), coal raised at a
/// working on the last square, and the coal under the two land squares of Ashby.
fn strip_years(ny: usize) -> Replay {
    let mut r = strip();
    let per_year = |v: &Vec<u8>| -> Vec<u8> { (0..ny).flat_map(|_| v.iter().copied()).collect() };
    r.ny = ny;
    r.filled = ny;
    r.header.meta.n_years = ny;
    for layer in [&mut r.use_, &mut r.inten, &mut r.trans, &mut r.catch, &mut r.rent, &mut r.dist] {
        *layer = per_year(layer);
    }
    // farmland on the three land squares, the third half of it pasture
    r.mix_a = per_year(&vec![0, 1, 1, 1]);
    r.mix_b = per_year(&vec![0, 1, 2, 1]);
    r.mix_s = per_year(&vec![255, 255, 128, 255]);
    r.town_active = (0..ny).flat_map(|_| [1, 1]).collect();
    r.town_wage = (0..ny).flat_map(|_| [2.0, 3.0]).collect();
    r.town_price = (0..ny).flat_map(|_| [1.0, 0.0, 1.5, 0.8]).collect();
    r.holds.sectors = vec![("farm".into(), 0), ("coalmine".into(), 1)];
    r.holds.fields = vec![("fertility".into(), vec![0, 200, 100, 50]), ("coal".into(), vec![0, 255, 51, 0])];
    // Ashby grows 30 grain, Brill 12; Brill raises 5 coal
    r.holds.made_towns = 2;
    r.holds.made = (0..ny).flat_map(|_| [30.0, 0.0, 12.0, 5.0]).collect();
    r.workings = (0..ny).map(|_| vec![replay::Working { cell: 3, class: 8, opened: 1750, hands: 4.0, out: 5.0, heap: 20.0, surface: 0.01 }]).collect();
    // traffic: carts from the third square to Ashby's, by cart, and from Brill's to the third, by rail
    let mut goods = vec![0u8; 4 * 4];
    goods[4] = isostate_core::traffic::goods_byte(3_000.0);
    goods[2 * 4] = isostate_core::traffic::goods_byte(90_000.0);
    r.traffic.goods = per_year(&goods);
    r.traffic.mode = per_year(&vec![0, 0, isostate_core::traffic::RAIL, 0]);
    r
}

fn tint(w: &Wash, c: usize) -> [u8; 4] {
    [w.cells[c * 4], w.cells[c * 4 + 1], w.cells[c * 4 + 2], w.cells[c * 4 + 3]]
}

/// A view as it is painted, its colours spread over the year's own figures.
fn wash_year(r: &Replay, k: usize, kind: Overlay, which: usize, month: Option<&Month>) -> Wash {
    wash(r, k, kind, which, month, Scaling::reckon(r, k, kind, which, Range::Year))
}

#[test]
fn the_sea_is_left_alone_and_land_carries_its_market() {
    let r = strip();
    for kind in Overlay::ALL.into_iter().filter(|&k| k != Overlay::Deposits) {
        let w = wash_year(&r, 0, kind, 0, None);
        assert_eq!(tint(&w, 0), [0, 0, 0, 0], "{kind:?} tints the sea");
        assert_eq!((tint(&w, 1)[3], tint(&w, 2)[3], tint(&w, 3)[3]), (1, 1, 2), "{kind:?} regions");
    }
}

#[test]
fn prices_and_wages_run_from_the_cheapest_town_to_the_dearest() {
    let r = strip();
    let w = wash_year(&r, 0, Overlay::Price, 0, None);
    // the cheapest and the dearest are named in the note; the ends lie a little inside them
    assert!(w.legend.note.ends_with("The cheapest market is Ashby at \u{00a3}1.00, the dearest Brill at \u{00a3}1.50."), "{}", w.legend.note);
    assert_eq!((w.legend.low.as_str(), w.legend.high.as_str()), ("and below", "and above"));
    assert_eq!(w.legend.range, Some(Range::Year));
    assert_eq!(&tint(&w, 1)[..3], &rgb(MADDER[0]));
    assert_eq!(&tint(&w, 3)[..3], &rgb(MADDER[4]));
    assert!(w.lines);
    // the round figures along the scale sit between the ends, in order
    let at: Vec<f32> = w.legend.ticks.iter().map(|t| t.0).collect();
    assert!(!at.is_empty() && at.windows(2).all(|p| p[0] < p[1]) && at.iter().all(|&a| (0.0..=1.0).contains(&a)), "{:?}", w.legend.ticks);
    // coal has a price only at Brill: Ashby's squares are blank, and the inspector says so
    let w = wash_year(&r, 0, Overlay::Price, 1, None);
    assert_eq!(&tint(&w, 1)[..3], &rgb(BLANK));
    assert_eq!(said(&r, 0, 2, Overlay::Price, 1, None).unwrap(), "Price: Ashby's market has no price for coal this year.");
    assert_eq!(said(&r, 0, 3, Overlay::Price, 1, None).unwrap(), "Price: \u{00a3}0.80 for coal at Brill, the market this square trades in.");
    // a square outside every market area has no price to take
    let mut lone = strip();
    lone.catch[3] = 0;
    assert_eq!(said(&lone, 0, 3, Overlay::Price, 0, None).unwrap(), "Price: it trades in no town's market.");
    let w = wash_year(&r, 0, Overlay::Wages, 0, None);
    assert!(w.legend.note.ends_with("The lowest pay is at Ashby, \u{00a3}2.00, the highest at Brill, \u{00a3}3.00."), "{}", w.legend.note);
    assert_eq!(said(&r, 0, 2, Overlay::Wages, 0, None).unwrap(), "Wages: a month's work pays \u{00a3}2.00 at Ashby.");
}

#[test]
fn where_the_clock_shows_a_month_prices_and_wages_are_the_months() {
    let r = strip();
    let mut months = Months::default();
    // January to March of the second year, as a live game sends them
    for (m, grain) in [(0, 1.3), (1, 1.25), (2, 1.2)] {
        months.add(&isostate_core::LiveMonth { year: 1751, month: m, index: 12 + m, series: Vec::new(), wage: vec![2.5, 2.8], price: vec![grain, 0.0, 1.1, 0.9], cash: Vec::new(), events: Vec::new() });
    }
    // the clock a little into April: March is the last month over
    let t = crate::clock::at(0, 3) + 0.04;
    assert_eq!(month_shown(&months, t, true), Some(14));
    // at the turn the last month over is December of the year before, which is not held: the year's averages
    assert_eq!(month_shown(&months, 0.0, true), None);
    // where the bar shows no month, the year's
    assert_eq!(month_shown(&months, t, false), None);
    // a month that has not come: the newest of its year that has
    assert_eq!(month_shown(&months, crate::clock::at(0, 8), true), Some(14));
    // and a year the viewer holds nothing of
    assert_eq!(month_shown(&months, crate::clock::at(4, 8), true), None);

    let month = Month::of(&r, &months, 14);
    assert_eq!(month.label, "March 1751");
    let w = wash_year(&r, 0, Overlay::Price, 0, Some(&month));
    assert_eq!(w.legend.title, "The price of grain in March 1751");
    assert!(w.legend.note.contains("at the month's end"), "{}", w.legend.note);
    // the month's prices, not the year's (which have Ashby the cheaper)
    assert!(w.legend.note.ends_with("The cheapest market is Brill at \u{00a3}1.10, the dearest Ashby at \u{00a3}1.20."), "{}", w.legend.note);
    assert_eq!(said(&r, 0, 2, Overlay::Price, 0, Some(&month)).unwrap(), "Price: \u{00a3}1.20 for grain at Ashby in March 1751, the market this square trades in.");
    assert_eq!(said(&r, 0, 2, Overlay::Price, 1, Some(&month)).unwrap(), "Price: Ashby's market has no price for coal this month.");
    let w = wash_year(&r, 0, Overlay::Wages, 0, Some(&month));
    assert_eq!(w.legend.title, "Wages in March 1751");
    assert!(w.legend.note.contains("Ashby, \u{00a3}2.50"), "{}", w.legend.note);
    assert_eq!(said(&r, 0, 2, Overlay::Wages, 0, Some(&month)).unwrap(), "Wages: a month's work paid \u{00a3}2.50 at Ashby in March 1751.");
    // the views that go by the year take no notice of a month
    assert_eq!(wash_year(&r, 0, Overlay::Rent, 0, Some(&month)).legend.title, wash_year(&r, 0, Overlay::Rent, 0, None).legend.title);
}

#[test]
fn the_inspector_says_what_the_overlay_shows() {
    let r = strip();
    assert_eq!(said(&r, 0, 0, Overlay::Rent, 0, None), None);
    // 12 months of the record's middle value: 10^(128/255*6 - 3) is about 1.03 a month
    assert_eq!(said(&r, 0, 2, Overlay::Rent, 0, None).unwrap(), "Rent here: about \u{00a3}12 a year.");
    // the record's top step is a floor, not a figure
    assert_eq!(said(&r, 0, 3, Overlay::Rent, 0, None).unwrap(), "Rent here: \u{00a3}12k a year or more.");
    assert_eq!(said(&r, 0, 2, Overlay::Areas, 0, None).unwrap(), "Market: it buys and sells at Ashby, one of 2 squares that do.");
    assert_eq!(said(&r, 0, 1, Overlay::Reach, 0, None).unwrap(), "Way to market: Ashby's market is here.");
    assert_eq!(said(&r, 0, 2, Overlay::Reach, 0, None).unwrap(), "Way to market: Ashby is as far as 6.0 km of flat open country.");
    assert_eq!(said(&r, 0, 3, Overlay::Reach, 0, None).unwrap(), "Way to market: there is no way from here to any market.");
}

#[test]
fn neighbouring_markets_never_share_a_tint() {
    let r = strip();
    let t = area_tints(&r, 0);
    assert_ne!(t[0], t[1]);
    let w = wash_year(&r, 0, Overlay::Areas, 0, None);
    assert_ne!(&tint(&w, 2)[..3], &tint(&w, 3)[..3]);
    // the year is in the title, and is not said again; the markets have no range to choose
    assert_eq!(w.legend.title, "Market areas in 1750");
    assert!(w.legend.note.ends_with("gives way to the next. 2 markets."));
    assert_eq!(w.legend.range, None);
}

#[test]
fn the_traffic_view_says_what_went_how_and_a_square_its_busiest_way() {
    let r = strip_years(3);
    let sc = Scaling::reckon(&r, 2, Overlay::Traffic, 0, Range::Year);
    let w = wash(&r, 2, Overlay::Traffic, 0, None, sc);
    assert_eq!(w.legend.title, "What the ways carried in 1752");
    assert!(w.legend.note.contains("97% by rail, 3% by cart") && w.legend.note.contains("near Ashby"), "{}", w.legend.note);
    let s = said(&r, 2, 2, Overlay::Traffic, 0, None).unwrap();
    assert!(s.starts_with("Traffic: the busiest way from this square, to the east, carried ") && s.contains("by rail in the year; the next, west, ") && s.ends_with("by cart."), "{s}");
    // a record that keeps no traffic says so
    let mut bare = strip_years(3);
    bare.traffic = Default::default();
    let w = wash(&bare, 2, Overlay::Traffic, 0, None, Scaling::reckon(&bare, 2, Overlay::Traffic, 0, Range::Year));
    assert!(w.legend.note.starts_with("This record does not keep what the ways carry") && w.legend.range.is_none());
}

#[test]
fn the_key_steps_through_every_overlay_and_back_to_the_plain_map() {
    let mut ov = MapOverlay::new(None, 0);
    let mut seen = Vec::new();
    for _ in 0..11 {
        ov.step(false);
        seen.push(ov.kind);
    }
    let mut all: Vec<Option<Overlay>> = Overlay::ALL.into_iter().map(Some).collect();
    all.push(None);
    assert_eq!(seen, all);
    // backwards from the plain map is the last of them, and the pill remembers it
    ov.step(true);
    assert_eq!(ov.kind, Some(Overlay::Traffic));
    ov.set(None);
    assert_eq!(ov.last, Overlay::Traffic);
}

#[test]
fn overlays_are_named_on_the_command_line() {
    let r = strip();
    let c = |kind, which, range| Some(Choice { kind, which, range, trend: None });
    assert_eq!(parse("rent", &r), c(Some(Overlay::Rent), None, None));
    assert_eq!(parse("price:Coal", &r), c(Some(Overlay::Price), Some(1), None));
    assert_eq!(parse("made:grain+year", &r), c(Some(Overlay::Made), Some(0), Some(Range::Year)));
    assert_eq!(parse("rent+30", &r), c(Some(Overlay::Rent), None, Some(Range::Thirty)));
    assert_eq!(parse("deposits:coal+all", &r), c(Some(Overlay::Deposits), Some(0), Some(Range::Whole)));
    assert_eq!(parse("deposits:timber", &r), c(Some(Overlay::Deposits), Some(4), None));
    assert_eq!(parse("plain", &r), c(None, None, None));
    assert_eq!(parse("markets", &r), None);
    assert_eq!(parse("rent+ever", &r), None);
    assert!(is_word("price:coal") && is_word("areas") && is_word("made+all") && !is_word("markets") && !is_word("island") && !is_word("rent+ever"));
    // a word that would be passed over is complained of, a good one is not
    assert_eq!(complaint("price:coal", &r), None);
    assert_eq!(complaint("made:coal+year", &r), None);
    assert_eq!(complaint("deposits:ore", &r), None);
    assert_eq!(complaint("wages", &r), None);
    assert!(complaint("rents", &r).unwrap().contains("not a map view"));
    assert_eq!(complaint("price:gin", &r).unwrap(), "\"gin\" is not a good of this run: use one of grain, coal");
    assert!(complaint("deposits:gold", &r).unwrap().starts_with("\"gold\" is not a kind of ground"));
    // a choice sets the view, its good or kind, and its range, and the range is kept for the view
    let mut ov = MapOverlay::new(None, 0);
    ov.choose(&parse("deposits:clay+year", &r).unwrap());
    assert_eq!((ov.kind, ov.deposit, ov.good, ov.range(Overlay::Deposits)), (Some(Overlay::Deposits), 2, 0, Range::Year));
    ov.choose(&parse("rent", &r).unwrap());
    assert_eq!((ov.range(Overlay::Rent), ov.range(Overlay::Deposits)), (Range::Thirty, Range::Year));
}

#[test]
fn every_view_that_colours_by_a_number_takes_the_three_ranges() {
    let r = strip_years(40);
    for kind in Overlay::ALL.into_iter().filter(|k| k.scaled()) {
        for range in Range::ALL {
            let sc = Scaling::reckon(&r, 39, kind, 0, range);
            let w = wash(&r, 39, kind, 0, None, sc);
            // a view the strip has figures for shows its range on its card; growth and residents
            // need kept town ground, which the strip has not, and say so with no scale
            if matches!(kind, Overlay::Growth | Overlay::Residents) {
                assert_eq!(w.legend.range, None, "{kind:?}");
            } else {
                assert_eq!(w.legend.range, Some(range), "{kind:?} {range:?}: {}", w.legend.note);
                assert!(!w.legend.ramp.is_empty(), "{kind:?} {range:?}");
            }
        }
    }
}

#[test]
fn one_wild_town_does_not_flatten_the_thirty_years() {
    // forty years of grain at 1.0 and 1.5; in year 30 Brill's price leaps to 20
    let mut r = strip_years(40);
    let ng = 2;
    r.town_price[(30 * 2 + 1) * ng] = 20.0;
    let ends = |k: usize, range: Range| Scaling::reckon(&r, k, Overlay::Price, 0, range).ends.unwrap();
    // this year alone, the wild year's high end is near the leap
    assert!(ends(30, Range::Year).hi > 15.0);
    // thirty years: the high end goes a fifth of the way toward it in the wild year, and on
    // from there no more than a fifth of what is left in each year after
    let calm = ends(29, Range::Thirty);
    let wild = ends(30, Range::Thirty);
    let to = ranges::window(&(0..=30).map(|j| year_ends(&r, j, Overlay::Price, 0)).collect::<Vec<_>>(), 30).unwrap();
    assert!((wild.hi - (calm.hi + ranges::PACE * (to.hi - calm.hi))).abs() < 1e-4, "{calm:?} {wild:?} {to:?}");
    let mut before = wild;
    for k in 31..40 {
        let now = ends(k, Range::Thirty);
        let years: Vec<Option<Ends>> = (0..=k).map(|j| year_ends(&r, j, Overlay::Price, 0)).collect();
        let to = ranges::window(&years, k).unwrap();
        assert!((now.hi - before.hi).abs() <= ranges::PACE * (to.hi - before.hi).abs() + 1e-4, "year {k}");
        before = now;
    }
    // the year after the leap, Brill's squares are no longer the darkest tint under thirty years
    let w = wash(&r, 31, Overlay::Price, 0, None, Scaling::reckon(&r, 31, Overlay::Price, 0, Range::Thirty));
    assert_ne!(&tint(&w, 3)[..3], &rgb(MADDER[4]));
}

#[test]
fn made_here_shares_a_crop_by_its_ground_and_takes_a_pit_as_it_is() {
    let r = strip_years(2);
    let m = made_map(&r, 1, 0).unwrap();
    assert!(m.by_land && !m.from_workings);
    // Ashby's 30 grain on its two squares: the first all farmland of fertility 200, the
    // second half farmland of 100, so four to one; Brill's 12 on its one square
    assert!((m.cells[1] - 24.0).abs() < 0.05 && (m.cells[2] - 6.0).abs() < 0.05, "{:?}", m.cells);
    assert!((m.cells[3] - 12.0).abs() < 1e-3);
    assert_eq!(m.total, 42.0);
    // coal comes from the working on the last square, with what it raised
    let m = made_map(&r, 1, 1).unwrap();
    assert!(m.from_workings);
    assert_eq!(m.cells, vec![0.0, 0.0, 0.0, 5.0]);
    let w = wash_year(&r, 1, Overlay::Made, 1, None);
    assert_eq!(&tint(&w, 1)[..3], &rgb(BLANK));
    assert_eq!(w.legend.title, "Where coal was made in 1751");
    assert_eq!(said(&r, 1, 3, Overlay::Made, 1, None).unwrap(), "Made here: about 5 of coal in the year, at the pit head.");
    // a good of the works goes over each town's own ground, here its middle
    let mut works = strip_years(2);
    works.holds.sectors.clear();
    let m = made_map(&works, 1, 0).unwrap();
    assert!(!m.by_land);
    assert_eq!(m.cells, vec![0.0, 30.0, 0.0, 12.0]);
    // a good nobody made says so, and a run without books says that
    let mut none = strip_years(2);
    none.holds.made.iter_mut().for_each(|v| *v = 0.0);
    assert_eq!(wash_year(&none, 1, Overlay::Made, 0, None).legend.note, "Nobody made any grain on the island in 1751.");
    assert!(wash_year(&strip(), 0, Overlay::Made, 0, None).legend.note.starts_with("This record does not keep what each town makes"));
}

#[test]
fn the_deposits_show_what_is_left_and_rule_round_what_was_worked() {
    let mut r = strip_years(2);
    // in the second year the hands meet half the coal's first richness on the second square
    r.holds.left = vec![Vec::new(), vec![(1, 0, 128)]];
    let g = ground_map(&r, 1, 0).unwrap();
    assert!((g.cells[1] - 128.0 / 255.0).abs() < 1e-3 && g.worked[1] && !g.worked[2]);
    let s = g.share_left.unwrap();
    assert!((s - (128.0 / 255.0 + 0.2) / 1.2).abs() < 1e-3, "{s}");
    let w = wash(&r, 1, Overlay::Deposits, 0, None, Scaling::reckon(&r, 1, Overlay::Deposits, 0, Range::Whole));
    assert!(w.lines);
    assert_eq!((tint(&w, 1)[3], tint(&w, 2)[3]), (2, 1));
    assert_eq!(w.legend.low, "none");
    assert!(w.legend.note.ends_with("Of the coal the island began with, 58% is left."), "{}", w.legend.note);
    assert_eq!(said(&r, 1, 1, Overlay::Deposits, 0, None).unwrap(), "Coal: 0.50 now, of 1.00 when the island was made; it has been worked.");
    // the whole run's high end is the first year's, before any was worked
    assert!((Scaling::reckon(&r, 1, Overlay::Deposits, 0, Range::Whole).ends.unwrap().hi - year_ends(&r, 0, Overlay::Deposits, 0).unwrap().hi).abs() < 1e-6);
    // a kind of ground the run does not hold says so
    assert_eq!(wash_year(&r, 1, Overlay::Deposits, 1, None).legend.note, "This record does not hold what the ground holds.");
}

#[test]
fn scales_run_from_low_to_high() {
    let even = Scale { lo: 2.0, hi: 6.0, log: false };
    assert_eq!(even.at(1.0), 0.0);
    assert_eq!(even.at(4.0), 0.5);
    assert_eq!(even.at(9.0), 1.0);
    let ratio = Scale { lo: 1.0, hi: 100.0, log: true };
    assert!((ratio.at(10.0) - 0.5).abs() < 1e-6);
    assert_eq!(ratio.at(0.0), 0.0);
    // one town, or every town alike: the middle of the ramp, not a division by nothing
    assert_eq!(Scale { lo: 3.0, hi: 3.0, log: false }.at(3.0), 0.5);
    // a scale by ratio glides in logarithms, and comes back to its figures
    let e = Ends { lo: measure(10.0, true), hi: measure(1000.0, true) };
    let sc = Scale::of(e, true);
    assert!((sc.lo - 10.0).abs() < 1e-3 && (sc.hi - 1000.0).abs() < 1e-1);
}

#[test]
fn figures_are_rounded_and_written_plainly() {
    assert_eq!(round_sig(1234.0), 1200.0);
    assert!((round_sig(0.0456) - 0.046).abs() < 1e-6);
    assert_eq!(coin(1.2), "\u{00a3}1.20");
    assert_eq!(coin(12.34), "\u{00a3}12.3");
    assert_eq!(coin(480.0), "\u{00a3}480");
    assert_eq!(coin(2600.0), "\u{00a3}2,600");
    // a rough sum keeps its two figures and no more
    assert_eq!(about(97.3), "\u{00a3}97");
    assert_eq!(about(0.744), "\u{00a3}0.74");
    assert_eq!(about(4.38), "\u{00a3}4.40");
    assert_eq!(about(1234.0), "\u{00a3}1,200");
    assert_eq!((amount(0.42), amount(2.0), amount(2.5), amount(12.3), amount(3400.0), amount(1.3e6)), ("0.42".into(), "2".into(), "2.5".into(), "12".into(), "3,400".into(), "1.3m".into()));
    assert_eq!(tick_words(Overlay::Price, 2.0), "\u{00a3}2");
    assert_eq!(tick_words(Overlay::Price, 1.2), "\u{00a3}1.20");
}

#[test]
fn the_ramp_ends_on_its_end_stops() {
    assert_eq!(ramp(&SEPIA, 0.0), rgb(SEPIA[0]));
    assert_eq!(ramp(&SEPIA, 1.0), rgb(SEPIA[4]));
    assert_eq!(ramp(&SEPIA, 0.5), rgb(SEPIA[2]));
}

#[test]
fn a_town_that_doubles_in_ten_years_reads_seven_point_two_a_year_and_new_ground_reads_new() {
    let mut r = strip_years(11);
    r.town_urban = vec![3000.0; 11 * 2];
    // Ashby's wage doubles from £2 to £4 in the ten years; Brill's stays at £3
    r.town_wage[10 * 2] = 4.0;
    let m = trend::trend_map(&r, 10, Overlay::Wages, 0, Span::Ten).unwrap();
    let trend::Change::Rate(v) = m.towns[0] else { panic!("{:?}", m.towns) };
    assert_eq!(trend::rate_words(v), "+7.2%");
    assert_eq!(m.towns[1], trend::Change::Rate(0.0));
    // wages go by towns: each square takes its market's rate
    assert!(m.by_town && m.cells[2] == m.towns[0] && m.cells[0] == trend::Change::Nothing);
    // the movers: Ashby rising, and nobody falling (Brill held)
    let (up, down) = trend::movers(&r, &m);
    assert_eq!((up.len(), up[0].name.as_str(), down.len()), (1, "Ashby", 0));
    // a town under the floor at the span's start is left out, whatever it has grown to
    r.town_urban[0] = 500.0;
    assert!(trend::movers(&r, &m).0.is_empty());
    // the scale is even about nought, and never nearer than the least end
    let e = trend::trend_ends(&r, 10, Overlay::Wages, 0, Span::Ten).unwrap();
    assert!(e.lo == -e.hi && e.hi >= trend::LEAST_END, "{e:?}");
    // before the span's first year is held there is no trend, and the card says from when
    assert!(trend::trend_map(&r, 9, Overlay::Wages, 0, Span::Ten).is_none());
    let w = trend_wash(&r, 9, Overlay::Wages, 0, Span::Ten, Scaling::reckon_trend(&r, 9, Overlay::Wages, 0, Range::Year, Some(Span::Ten)));
    assert!(w.legend.note.starts_with("A change over 10 years can be shown from 1760"), "{}", w.legend.note);

    // residents, square by square: the second square is town ground throughout and doubles; the
    // third joins its town in the span and is new, the last is town ground only at the start and is gone
    let (n, ny) = (4, 11);
    r.built = vec![0; ny * n];
    r.res = vec![0; ny * n];
    for k in 0..ny {
        r.built[k * n + 1] = 10;
        r.res[k * n + 1] = if k < 10 { 40 } else { 80 };
    }
    r.built[10 * n + 2] = 100;
    r.res[10 * n + 2] = 20;
    r.built[3] = 10;
    r.res[3] = 30;
    let m = trend::trend_map(&r, 10, Overlay::Residents, 0, Span::Ten).unwrap();
    assert!(!m.by_town);
    assert!(matches!(m.cells[1], trend::Change::Rate(v) if trend::rate_words(v) == "+7.2%"), "{:?}", m.cells);
    assert_eq!((m.cells[2], m.cells[3]), (trend::Change::New, trend::Change::Gone));
    let w = trend_wash(&r, 10, Overlay::Residents, 0, Span::Ten, Scaling::reckon_trend(&r, 10, Overlay::Residents, 0, Range::Thirty, Some(Span::Ten)));
    assert_eq!((&tint(&w, 2)[..3], &tint(&w, 3)[..3]), (&rgb(NEW)[..], &rgb(GONE)[..]));
    assert_eq!(w.legend.title, "Residents: change a year, 1750 to 1760");
    assert_eq!(w.legend.keys.iter().map(|k| k.1.as_str()).collect::<Vec<_>>(), ["new since 1750", "gone since 1750"]);
    assert!(w.legend.note.contains("1 square new since 1750, 1 gone"), "{}", w.legend.note);
    // the still middle of the scale is pale
    assert_eq!(change_tint(trend::Change::Rate(0.0), &Scale { lo: -2.0, hi: 2.0, log: false }), rgb(CALM));
}

#[test]
fn the_trend_is_named_on_the_command_line_and_greyed_where_it_means_nothing() {
    let r = strip();
    assert_eq!(parse("residents+trend10", &r).unwrap().trend, Some(Some(Span::Ten)));
    let c = parse("wages+trendoff+year", &r).unwrap();
    assert_eq!((c.trend, c.range), (Some(None), Some(Range::Year)));
    assert_eq!(parse("rent", &r).unwrap().trend, None);
    assert_eq!(parse("rent+trend7", &r), None);
    assert!(is_word("price:coal+30+trend5") && !is_word("rent+trend"));
    // the switch is one for every view, and kept while a view without a trend is on
    let mut ov = MapOverlay::new(None, 0);
    ov.choose(&parse("residents+trend30", &r).unwrap());
    assert_eq!(ov.trend_shown(), Some(Span::Thirty));
    ov.choose(&parse("areas", &r).unwrap());
    assert_eq!((ov.trend, ov.trend_shown()), (Some(Span::Thirty), None));
    for kind in Overlay::ALL {
        assert_eq!(trend::trends(kind), matches!(kind, Overlay::Residents | Overlay::Wages | Overlay::Rent | Overlay::Price | Overlay::Made), "{kind:?}");
    }
}
