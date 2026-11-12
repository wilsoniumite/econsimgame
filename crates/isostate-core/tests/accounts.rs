//! The books (not in the Python; docs/rules/accounts.md): they change nothing
//! in the run they watch, they balance, they agree with what the observer and
//! the families keep for themselves, and they are the same books every time.
//! With the switch off the golden test (tests/golden.rs) covers the port.
#![allow(non_snake_case)]

use isostate_core::accounts::{self, GOODS_FIELDS, KG, KL, KM, KT, KW, METHOD_FIELDS, TOWN_FIELDS, WORKS_FIELDS};
use isostate_core::observer::annual;
use isostate_core::*;

fn params(rules: Rules, seed: u64, families: usize, end_year: i64, books: bool) -> Params {
    let mut P = rules.params();
    P.seed = seed;
    P.n_families = families;
    P.end_year = Some(end_year);
    P.accounts.enabled = books;
    P
}

fn run(rules: Rules, P: &Params) -> RunResult {
    // run_with checks every month that money only changed by what was issued
    sim::run_with(P, rules.content(), false, true).expect("money is conserved every month")
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn col(name: &str) -> usize {
    GOODS_FIELDS.iter().position(|&n| n == name).unwrap()
}

fn wcol(name: &str) -> usize {
    WORKS_FIELDS.iter().position(|&n| n == name).unwrap()
}

fn tcol(name: &str) -> usize {
    TOWN_FIELDS.iter().position(|&n| n == name).unwrap()
}

fn mcol(name: &str) -> usize {
    METHOD_FIELDS.iter().position(|&n| n == name).unwrap()
}

/// Names of the arrays in a packed run, and its header.
fn header(blob: &[u8]) -> serde_json::Value {
    let n = u32::from_le_bytes(blob[..4].try_into().unwrap()) as usize;
    serde_json::from_slice(&blob[4..4 + n]).unwrap()
}

#[test]
fn off_under_the_port_and_on_in_the_game() {
    assert!(!default_params().accounts.enabled && game_params().accounts.enabled);
    // a port run keeps no books and says nothing of them anywhere
    let P = params(Rules::Port, 7, 0, 1752, false);
    let r = run(Rules::Port, &P);
    let b = &r.state.acct;
    assert!(!b.on && b.years.is_empty() && b.hist.n == 0 && b.hist.goods.capacity() == 0 && b.last_months.is_empty() && b.seconds == 0.0);
    assert!(r.state.series.names.iter().all(|n| !n.starts_with("acct_")));
    assert!(accounts::town_books(&r.state, &r.content, 0, None).is_none());
    assert!(accounts::works_books(&r.state, &r.content, 0).is_none());
    let h = header(&record::pack(&r.state, &r.content, &r.params, &r.events, &r.rec, "x", false));
    assert!(h.get("accounts").is_none());
    assert!(h["arrays"].as_array().unwrap().iter().all(|a| !a["name"].as_str().unwrap().starts_with("acct_")));
    assert!(!serde_json::to_string(&r.params).unwrap().contains("accounts"), "params.json of a port run does not mention the books");
}

#[test]
fn the_books_change_nothing() {
    // three islands, with and without families, under the port's rules and the
    // game's: every series, every piece of news and the state the run ends in
    // are bit for bit what they are with the books off
    for (rules, seed, families, end) in [(Rules::Port, 3, 6, 1775), (Rules::Port, 11, 0, 1765), (Rules::Game, 7, 6, 1772), (Rules::Game, 5, 0, 1762)] {
        let off = run(rules, &params(rules, seed, families, end, false));
        let on = run(rules, &params(rules, seed, families, end, true));
        let (a, b) = (&off.state, &on.state);
        for (k, name) in a.series.names.iter().enumerate() {
            assert_eq!(bits(&a.series.data[k]), bits(b.series.get(name)), "series {name} moved, seed {seed}");
        }
        assert_eq!(b.series.names.len(), a.series.names.len() + 8, "the books add their eight series after everyone's");
        assert_eq!(serde_json::to_string(&off.events).unwrap(), serde_json::to_string(&on.events).unwrap());
        for (what, x, y) in [
            ("people", &a.pop, &b.pop),
            ("households' cash", &a.hh_cash, &b.hh_cash),
            ("landlords' cash", &a.lord_cash, &b.lord_cash),
            ("merchants' cash", &a.merch_cash, &b.merch_cash),
            ("works' cash", &a.f_cash, &b.f_cash),
            ("works' capacity", &a.f_K, &b.f_K),
            ("prices", &a.price.data, &b.price.data),
            ("stocks", &a.inv.data, &b.inv.data),
            ("wages", &a.wage, &b.wage),
            ("land shares", &a.lam.data, &b.lam.data),
            ("forest cover", &a.cover, &b.cover),
        ] {
            assert_eq!(bits(x), bits(y), "{what} moved, seed {seed}");
        }
        let cash = |st: &State| st.families.iter().map(|f| f.cash.to_bits()).collect::<Vec<_>>();
        assert_eq!(cash(a), cash(b));
        assert_eq!((a.F_count, a.issued.to_bits(), a.money_now.to_bits()), (b.F_count, b.issued.to_bits(), b.money_now.to_bits()));
        assert_eq!(off.rec.layers, on.rec.layers);
        assert!(b.acct.on && b.acct.years.len() == (end - 1749) as usize);
    }
}

/// Every year of a run: goods, incomes and spending all balance.
fn balances(rules: Rules, seed: u64, families: usize, end: i64) {
    let r = run(rules, &params(rules, seed, families, end, true));
    let (st, C) = (&r.state, &r.content);
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    let (T, G) = (st.T_max, C.G);
    for (k, y) in st.acct.years.iter().enumerate() {
        // made + received + moved + oversold = bought + sent + spoiled + added to stock,
        // for every town and good, as the books added it up in double precision
        assert!(y.balance_goods < 1e-9, "{}: {} of {} unplaced in town {}", y.year, y.balance_goods_units, C.good_name[y.balance_goods_at.1], y.balance_goods_at.0);
        // every month, what was paid as income is what households, landlords,
        // families and the fund received
        assert!(y.balance_income < 1e-9, "{}: incomes off by {:e}", y.year, y.balance_income);
        // and over the year
        let rents = y.rent_farm + y.rent_pits + y.rent_sites + y.rent_town;
        let paid = y.wage_bill + y.div_households + y.div_families + y.merchants_div + rents + y.fund_div + y.issue;
        let got = y.to_households + y.to_landlords + y.to_fund + y.to_families.iter().map(|f| f.dividends + f.rents).sum::<f64>();
        assert!((paid - got).abs() < 1e-9 * paid, "{}: {paid} paid as income and {got} received", y.year);
        // what the island earned is what it spent on goods, house rent and building
        assert!(y.balance_spending < 1e-9, "{}: income {} against spending {}", y.year, y.income, y.spending);
        assert!((y.share_work + y.share_works + y.share_merchants + y.share_land - 1.0).abs() < 1e-9);
        // (the shares themselves are not held to any range: in a year a town is founded
        // the port's land can pay more in wages than its produce fetches, and its
        // rent, the remainder, is then below nothing; docs/rules/accounts.md)
        assert!(y.share_work > 0.0 && y.share_work.is_finite() && y.share_land.is_finite());
        assert!(y.pay_share > 0.3 && y.pay_share <= 1.0);
        assert!(y.works_years > 0.0 && y.ownership_years.is_finite());
        assert!(y.oversold_share >= 0.0 && y.oversold_share < 1e-3, "{}: {} of all that was bought was not in store", y.year, y.oversold_share);
        // the same balance from the tables as they are kept (single precision)
        let rec = st.acct.year(k).unwrap();
        for t in 0..T {
            for g in 0..G {
                let v = |name: &str| rec.goods[(t * G + g) * KG + col(name)] as f64;
                let bought = v("households") + v("landlords") + v("families") + v("works_inputs") + v("works_fuel") + v("land") + v("building") + v("projects");
                let through = v("stock_open") + v("made") + v("received");
                let left = v("made") + v("received") + v("moved") + v("oversold") - bought - v("sent") - v("spoiled") - (v("stock") - v("stock_open"));
                assert!(left.abs() <= 1e-5 * through + 1e-6, "{} town {t} {}: {left} unplaced of {through}", y.year, C.good_name[g]);
                assert!(bought <= v("asked") * (1.0 + 1e-5) + 1e-6, "nobody buys more than was asked for");
                assert!(v("price_low") <= v("price") && v("price") <= v("price_high"));
            }
        }
        // every flow between towns is in the towns' own columns
        for t in 0..T {
            for g in 0..G {
                let sent: f64 = rec.trade.iter().filter(|f| f.from as usize == t && f.good as usize == g).map(|f| f.qty as f64).sum();
                let recv: f64 = rec.trade.iter().filter(|f| f.to as usize == t && f.good as usize == g).map(|f| f.qty as f64).sum();
                let v = |name: &str| rec.goods[(t * G + g) * KG + col(name)] as f64;
                assert!((sent - v("sent")).abs() <= 1e-5 * sent + 1e-6 && (recv - v("received")).abs() <= 1e-5 * recv + 1e-6);
            }
        }
        // stock that changed between two months went with a district from one town
        // to another, so over the island `moved` comes to nothing: nothing is made
        // or lost under that name
        for g in 0..G {
            let v = |t: usize, name: &str| rec.goods[(t * G + g) * KG + col(name)] as f64;
            let moved: f64 = (0..T).map(|t| v(t, "moved")).sum();
            let held: f64 = (0..T).map(|t| v(t, "stock_open") + v(t, "made") + v(t, "received")).sum();
            assert!(moved.abs() <= 1e-5 * held + 1e-6, "{} {}: {moved} appeared between months", y.year, C.good_name[g]);
        }
        // the towns' tables add up to the island's account
        let near = |a: f64, b: f64| (a - b).abs() <= 1e-5 * b.abs() + 1e-3;
        let town = |name: &str| (0..T).map(|t| rec.towns[t * KT + tcol(name)] as f64).sum::<f64>();
        let kinds = ["pay_works", "pay_land", "pay_projects", "cart_works", "cart_land", "cart_trade", "managers", "div_works", "div_merchants", "fund", "issue"];
        let received: f64 = kinds.iter().map(|n| town(n)).sum();
        assert!(near(received, y.to_households), "{}: the towns' households received {received}, the island's {}", y.year, y.to_households);
        assert!(near(town("rent_land"), y.rent_farm + y.rent_pits) && near(town("rent_sites"), y.rent_sites) && near(town("rent_town"), y.rent_town));
        assert!(near(town("rent_families"), y.to_families.iter().map(|f| f.rents).sum()));
        assert!(near(town("div_families"), y.to_families.iter().map(|f| f.dividends).sum()));
        let land_rent: f64 = rec.land.chunks(KL).map(|l| l[KL - 1] as f64).sum();
        assert!(near(land_rent, y.rent_farm + y.rent_pits), "{}: the sectors' rents", y.year);
        assert!(near(y.rent_by_sector.iter().sum(), y.rent_farm + y.rent_pits));
        // the pits' rent is that of the four sectors that dig: quarries and clay
        // pits as well as the two the content marks as mines
        let digs: f64 = (0..C.NS).filter(|&s| mines::is_extractive(&C.sector_name[s])).map(|s| y.rent_by_sector[s]).sum();
        assert!((y.rent_pits - digs).abs() <= 1e-9 * digs.abs().max(1.0), "{}: {} to the pits, {digs} by sector", y.year, y.rent_pits);
        assert!(y.rent_by_sector[C.sector_name.iter().position(|n| n == "quarry").unwrap()] != 0.0);
        // what was spent on each need is what was spent on goods
        let S = C.S;
        let need = |who: usize| (0..T * S).map(|k| rec.needs[k * 2 + who] as f64).sum::<f64>();
        assert!(near(need(0), y.spend_households), "{}: {} on households' needs, {} on their goods", y.year, need(0), y.spend_households);
        assert!(near(need(1), y.spend_landlords + y.spend_families));
        assert!(near(y.need_households.iter().sum(), y.spend_households));
        // the works' rows add up to their methods' and to the island's
        let works = |name: &str| rec.works.chunks(KW).map(|w| w[wcol(name)] as f64).sum::<f64>();
        let method = |name: &str| rec.methods.chunks(KM).map(|m| m[mcol(name)] as f64).sum::<f64>();
        for name in ["out", "value", "wages", "inputs", "fuel", "cart_in", "cart_out", "rent", "managers", "profit", "dividends"] {
            assert!(near(works(name), method(name)), "{} {name}: {} over the works, {} over the methods", y.year, works(name), method(name));
        }
        assert!(near(works("dividends"), y.div_households + y.div_families));
        assert!(near(works("wages"), y.pay_works) && near(works("invest"), y.invest));
    }
}

#[test]
fn the_books_balance_under_the_port() {
    balances(Rules::Port, 3, 6, 1789);
}

#[test]
fn the_books_balance_under_the_game() {
    balances(Rules::Game, 7, 6, 1784);
}

#[test]
fn the_books_keep_up_with_the_mines_and_the_canals() {
    // the game as it is played, to 1800 on seed 7, where canals are dug
    // (tests/canals.rs) and workings are sunk: the sinkers' pay, a working's
    // royalty split among the cells of its lease, and navvies paid two shillings
    // on every shilling of materials all pass through the books
    const END: i64 = 1800;
    let r = run(Rules::Game, &params(Rules::Game, 7, 6, END, true));
    let (st, C) = (&r.state, &r.content);
    assert!(r.params.mines.enabled && r.params.canals.enabled && r.params.canals.wages > 0.3);
    for y in &st.acct.years {
        assert!(y.balance_goods < 1e-9 && y.balance_income < 1e-9 && y.balance_spending < 1e-9, "{}: {} {} {}", y.year, y.balance_goods, y.balance_income, y.balance_spending);
    }
    let (T, NS) = (st.T_max, C.NS);
    // the towns' navvies are the canals' own count of them, year by year
    let mut years_digging = 0;
    for cy in &st.cn.years {
        let rec = st.acct.year((cy.year - 1750) as usize).unwrap();
        let navvies: f64 = (0..T).map(|t| rec.towns[t * KT + tcol("navvies")] as f64).sum();
        assert!((navvies - cy.navvies).abs() <= 1e-4 * cy.navvies + 1e-3, "{}: {navvies} navvies in the books, {} in the canals' own", cy.year, cy.navvies);
        years_digging += (cy.navvies > 50.0) as usize;
    }
    assert!(years_digging > 0, "no canal was dug by {END}");
    // a railway's navvies are paid three tenths of its materials and a canal's
    // twice them: every year's pay lies between, and a year of canals alone is at the top
    let canal = r.params.canals.wages;
    let shares: Vec<f64> = st.acct.years.iter().filter(|y| y.pay_projects > 0.0).map(|y| y.pay_projects / (y.projects - y.pay_projects)).collect();
    assert!(shares.iter().all(|&s| s > 0.3 - 1e-9 && s < canal + 1e-9), "{shares:?}");
    assert!(shares.iter().any(|&s| (s - canal).abs() < 1e-9), "no year of canals alone: {shares:?}");
    // the mines keep each working's hands (the sinkers among them), output and
    // royalty over the last complete year; sector by sector they are the books'
    assert!(st.mine_opened.iter().any(|&y| y as i64 == END - 1), "no working was being sunk in {END}");
    let rec = st.acct.year((END - 1750) as usize).unwrap();
    for (row, &s) in st.mine_sectors.iter().enumerate() {
        let own = |m: &mat::Mat| (0..st.N).map(|c| m[(row, c)]).sum::<f64>();
        let books = |name: &str| (0..T).map(|t| rec.land[(t * NS + s) * KL + accounts::LAND_FIELDS.iter().position(|&n| n == name).unwrap()] as f64).sum::<f64>();
        for (name, m) in [("hands", &st.mine_hands), ("out", &st.mine_out), ("rent", &st.mine_royalty)] {
            assert!((books(name) - own(m)).abs() <= 1e-5 * own(m).abs() + 1e-3, "{} {name}: {} in the books, {} in the mines' own", C.sector_name[s], books(name), own(m));
        }
    }
    // and the families' rents, royalties among them, are what the families say they had
    let last = st.acct.years.last().unwrap();
    for (fam, got) in st.families.iter().zip(&last.to_families) {
        assert!((got.rents - fam.last_year.rents).abs() <= 1e-9 * fam.last_year.rents.abs().max(1.0), "the {} family: {} in the books, {} in its own", fam.surname, got.rents, fam.last_year.rents);
    }
}

#[test]
fn the_books_are_ready_for_the_island_made_by_water() {
    // land (docs/rules/land.md) is off in the game until the viewer can draw
    // it. With it thrown, thirty years with six families: the books balance,
    // and the run is the run it is without them.
    let mut P = params(Rules::Game, 3, 6, 1779, true);
    P.land = landform::LandParams::game();
    let on = run(Rules::Game, &P);
    assert!(on.state.world.landform.is_some(), "the island was made by the land mechanism");
    assert_eq!(on.state.acct.years.len(), 30);
    for y in &on.state.acct.years {
        assert!(y.balance_goods < 1e-9 && y.balance_income < 1e-9 && y.balance_spending < 1e-9, "{}: {} {} {}", y.year, y.balance_goods, y.balance_income, y.balance_spending);
    }
    P.accounts.enabled = false;
    let off = run(Rules::Game, &P);
    for (k, name) in off.state.series.names.iter().enumerate() {
        assert_eq!(bits(&off.state.series.data[k]), bits(on.state.series.get(name)), "series {name} moved");
    }
    assert_eq!(serde_json::to_string(&off.events).unwrap(), serde_json::to_string(&on.events).unwrap());
}

#[test]
fn the_books_balance_under_the_commons() {
    // rents go to the common fund and out again as a dividend to every household
    let mut P = params(Rules::Port, 2, 0, 1760, true);
    P.rent_regime = RentRegime::Commons;
    let r = run(Rules::Port, &P);
    for y in &r.state.acct.years {
        assert!(y.balance_goods < 1e-9 && y.balance_income < 1e-9 && y.balance_spending < 1e-9, "{}", y.year);
        assert!(y.to_landlords == 0.0 && y.to_fund > 0.0 && y.fund_div > 0.0);
    }
}

#[test]
fn the_books_follow_what_a_player_does_between_years() {
    // a works founded and land bought between two years, as the viewer's player
    // does: the works has books from its first year, with its owner, the land's
    // rent reaches the family in the books, and everything still balances
    let P = params(Rules::Game, 3, 6, 1772, true);
    let mut s = sim::start(&P, Rules::Game.content());
    let C = s.content.clone();
    let d = pending(&s.st, 0);
    decide(&mut s.st, &C, 0, d[0].id, 0).unwrap();
    // (the year it happened in, the works, the family's rents the year before)
    let mut done: Option<(i64, usize, f64)> = None;
    while s.year <= s.end_year {
        sim::run_year(&mut s).expect("money is conserved every month");
        if done.is_some() {
            continue;
        }
        let seat = s.st.families[0].seat;
        let mut founded = None;
        'site: for c in 0..s.st.N {
            if s.st.catch[c] != seat as i64 {
                continue;
            }
            let Ok(options) = site_options(&s.st, &C, &s.params, 0, c) else { continue };
            for o in options.iter().filter(|o| o.allowed.is_ok()) {
                if let Ok(f) = found_works(&mut s.st, &C, &s.params, 0, o.method, c, s.year, &mut s.events) {
                    founded = Some(f);
                    break 'site;
                }
            }
        }
        let Some(f) = founded else { continue };
        let mut bought = 0;
        for c in 0..s.st.N {
            if bought < 3 && s.st.catch[c] == seat as i64 && may_buy_land(&s.st, 0, c).is_ok() && buy_land(&mut s.st, &C, &s.params, 0, c).is_ok() {
                bought += 1;
            }
        }
        assert!(bought > 0, "the player can buy some land around its seat");
        done = Some((s.year, f, s.st.acct.years.last().unwrap().to_families[0].rents));
    }
    let (year, f, rents_before) = done.expect("the player could found some works at its seat in twenty years");
    let st = &s.st;
    for y in &st.acct.years {
        assert!(y.balance_goods < 1e-9 && y.balance_income < 1e-9 && y.balance_spending < 1e-9, "{}: {} {} {}", y.year, y.balance_goods, y.balance_income, y.balance_spending);
    }
    let wb = accounts::works_books(st, &C, f).expect("the new works' books");
    assert_eq!(wb.years.first().map(|y| (y.year, y.owner)), Some((year, Some(0))), "it has books from its first year, in the player's name");
    assert!(wb.years.iter().any(|y| y.invest > 0.0), "and they show it being built");
    let k = (year - 1750) as usize;
    assert!(st.acct.years[k].to_families[0].rents > rents_before, "the land's rent reaches the family in the books");
}

#[test]
fn a_loaded_game_has_its_books_again() {
    // a save is the player's orders; loading plays the years over, books and all
    let mut g = save::Game::new(3, 6, 0);
    let d = pending(&g.sim.st, 0);
    g.give(save::Order::decide(&g.sim, d[0].id, 0).unwrap()).unwrap();
    for _ in 0..3 {
        g.year().unwrap();
    }
    let seat = g.sim.st.families[0].seat as i64;
    let cell = (0..g.sim.st.N).find(|&c| g.sim.st.catch[c] == seat && may_buy_land(&g.sim.st, 0, c).is_ok()).expect("some land to buy at the seat");
    g.give(save::Order::BuyLand { cell }).expect("the land is bought");
    g.year().unwrap();
    let again = save::load(&g.save(0), |_| {}).unwrap();
    assert!(!again.drifted && again.refused.is_empty());
    let (was, now) = (&g.sim.st.acct, &again.game.sim.st.acct);
    assert_eq!(was.years.len(), 4);
    assert_eq!(was.years, now.years);
    assert_eq!(was.hist.goods.iter().map(|v| v.to_bits()).collect::<Vec<_>>(), now.hist.goods.iter().map(|v| v.to_bits()).collect::<Vec<_>>());
}

/// Everything the books publish of two games, and every series, is the same to the last bit.
fn same_books(a: &sim::Sim, b: &sim::Sim, when: &str) {
    let f32bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!((a.year, a.t), (b.year, b.t), "{when}: where the games stand");
    let (x, y) = (&a.st.acct, &b.st.acct);
    // (as text: a share of nothing is NaN, which is not equal to itself)
    assert_eq!(format!("{:?}", x.years), format!("{:?}", y.years), "{when}: the island's account");
    let (h, k) = (&x.hist, &y.hist);
    assert_eq!(h.n, k.n, "{when}: years put away");
    for (name, p, q) in [
        ("goods", &h.goods, &k.goods),
        ("land", &h.land, &k.land),
        ("methods", &h.methods, &k.methods),
        ("towns", &h.towns, &k.towns),
        ("needs", &h.needs, &k.needs),
        ("works", &h.works, &k.works),
        ("the last year's months", &x.last_months, &y.last_months),
        ("what the works used last year", &x.last_works_in, &y.last_works_in),
    ] {
        assert_eq!(f32bits(p), f32bits(q), "{when}: {name}");
    }
    assert_eq!((&h.trade_off, &h.use_off, &h.works_off, &h.works_id), (&k.trade_off, &k.use_off, &k.works_off, &k.works_id), "{when}");
    assert_eq!(format!("{:?}", (&h.trade, &h.uses)), format!("{:?}", (&k.trade, &k.uses)), "{when}: trade and what the works used");
    assert_eq!(a.st.series.names, b.st.series.names);
    for (i, name) in a.st.series.names.iter().enumerate() {
        assert_eq!(bits(&a.st.series.data[i]), bits(&b.st.series.data[i]), "{when}: the series {name}");
    }
    for t in 0..a.st.town_name.len() {
        assert_eq!(format!("{:?}", accounts::town_books(&a.st, &a.content, t, None)), format!("{:?}", accounts::town_books(&b.st, &b.content, t, None)), "{when}: town {t}'s books");
    }
    assert_eq!(save::check(a), save::check(b), "{when}");
}

#[test]
fn a_game_left_in_the_middle_of_a_year_goes_on_with_the_same_books() {
    // A game played a month at a time and saved in June, with land bought at
    // the last turn, and a move of the seat and more land waiting for the
    // year's end. Loaded, it has the books it had; and stepped on by the
    // month, through the year's end where what waited is carried out and well
    // into the year after the next, its books are those of the game that was
    // never left: the sums the books were running in June came back too.
    let mut g = save::Game::new(3, 6, 0);
    let d = pending(&g.sim.st, 0);
    g.give(save::Order::decide(&g.sim, d[0].id, 0).unwrap()).unwrap();
    for _ in 0..2 {
        g.year().unwrap();
        g.turn();
    }
    let seat = g.sim.st.families[0].seat;
    let land = |g: &save::Game| (0..g.sim.st.N).find(|&c| g.sim.st.catch[c] == seat as i64 && may_buy_land(&g.sim.st, 0, c).is_ok()).expect("some land to buy at the seat");
    let cell = land(&g);
    g.give(save::Order::BuyLand { cell }).expect("the land is bought");
    for _ in 0..5 {
        assert!(!g.month().unwrap());
    }
    let other = (0..g.sim.st.T_max).find(|&t| t != seat && g.sim.st.town_active[t]).expect("another town");
    assert_eq!(g.give(save::Order::MoveSeat { town: other }), Ok(save::Done::Waits));
    assert_eq!(g.give(save::Order::BuyLand { cell: land(&g) }), Ok(save::Done::Waits));
    let saved = save::Save::from_text(&g.save(0).to_text()).unwrap();
    assert_eq!((saved.year, saved.month, saved.orders.len(), saved.waiting.len()), (1751, 5, 2, 2));

    let again = save::load(&saved, |_| {}).unwrap();
    assert!(!again.drifted && again.refused.is_empty());
    let mut h = again.game;
    same_books(&g.sim, &h.sim, "in June 1752");
    // of the year in progress the books say nothing yet, in either: the last
    // year they have is 1751, and their series have no figure for a month of 1752
    for s in [&g.sim, &h.sim] {
        assert_eq!((s.st.acct.years.len(), s.st.acct.hist.n, s.t), (2, 2, 29));
        assert_eq!(accounts::town_books(&s.st, &s.content, 0, None).unwrap().year, 1751);
        assert!(accounts::year_series(&s.st, 1752).is_empty() && accounts::year_series(&s.st, 1751).len() == 8);
        for name in s.st.series.names.iter().filter(|n| accounts::own_series(n)) {
            let months = s.st.series.get(name);
            assert!(months[..24].iter().all(|v| v.is_finite()) && months[24..].iter().all(|v| v.is_nan()), "{name}: {:?}", &months[..36]);
        }
    }

    // on by the month: the rest of 1752, all of 1753, and three months of 1754
    let mut turns = 0;
    for _ in 0..7 + 12 + 3 {
        let (a, b) = (g.month().unwrap(), h.month().unwrap());
        assert_eq!(a, b);
        if a {
            turns += 1;
            let (done, done_again) = (g.turn(), h.turn());
            assert_eq!(format!("{done:?}"), format!("{done_again:?}"), "what became of the orders that waited");
            assert_eq!(done.len(), if turns == 1 { 2 } else { 0 });
            same_books(&g.sim, &h.sim, "at a year's turn");
        }
    }
    same_books(&g.sim, &h.sim, "in April 1754");
    assert_eq!((g.sim.st.acct.years.len(), g.month, g.sim.st.families[0].seat), (4, 3, other));
    assert_eq!(g.orders, h.orders);
    for y in &g.sim.st.acct.years {
        assert!(y.balance_goods < 1e-9 && y.balance_income < 1e-9 && y.balance_spending < 1e-9, "{}", y.year);
    }
}

/// The whole of the books as text: what they publish, the sums they are
/// running for the year in progress, and the month's scratch, to the last
/// digit (a float written with `{:?}` reads back as the same float). All but
/// the wall time they took, which is the last thing in them.
fn whole_books(s: &sim::Sim) -> String {
    let text = format!("{:?}", s.st.acct);
    let cut = text.rfind("seconds: ").expect("the books' clock is the last thing in them");
    text[..cut].to_string()
}

/// Two games stand at the same point with the same books, the year in
/// progress and all, the same series and the same fingerprint.
fn same_whole(a: &sim::Sim, b: &sim::Sim, when: &str) {
    assert_eq!((a.year, a.t), (b.year, b.t), "{when}: where the games stand");
    let (x, y) = (whole_books(a), whole_books(b));
    if x != y {
        let at = x.bytes().zip(y.bytes()).position(|(p, q)| p != q).unwrap_or(x.len().min(y.len()));
        let from = x[..at].rfind(|c: char| c.is_alphabetic()).map_or(0, |i| i.saturating_sub(40));
        panic!("{when}: the books differ, {} characters in:\n  {}\n  {}", at, &x[from..(at + 60).min(x.len())], &y[from..(at + 60).min(y.len())]);
    }
    for (i, name) in a.st.series.names.iter().enumerate() {
        assert_eq!(bits(&a.st.series.data[i]), bits(&b.st.series.data[i]), "{when}: the series {name}");
    }
    assert_eq!(save::check(a), save::check(b), "{when}");
    same_books(a, b, when);
}

#[test]
fn the_books_are_the_same_books_however_the_game_was_reached() {
    // One game, reached five ways: played through by the month; saved at a
    // year's turn and loaded; saved in May and loaded; saved in June with
    // orders waiting and loaded; and loaded, saved and loaded again. At three
    // points (in June 1752 with two orders waiting, at the turn after it where
    // they are carried out, and in April 1754) everything in the books is
    // compared, not only what they publish: the sums they are running for the
    // year in progress and the months they have kept of it are in the text too.
    //
    // The player's orders, by the calendar: a decision answered before the
    // first year, land bought at the turn of 1752, and in June 1752 a move of
    // the seat and more land, which wait. Each is given where it falls, unless
    // the game came from a save that has it already.
    fn orders(g: &mut save::Game) {
        let seat = g.sim.st.families[0].seat;
        let land = |g: &save::Game| (0..g.sim.st.N).find(|&c| g.sim.st.catch[c] == seat as i64 && may_buy_land(&g.sim.st, 0, c).is_ok()).expect("some land to buy at the seat");
        match (g.last_year(), g.month) {
            (1749, 0) if g.orders.is_empty() => {
                let d = pending(&g.sim.st, 0);
                g.give(save::Order::decide(&g.sim, d[0].id, 0).unwrap()).unwrap();
            }
            (1751, 0) if !g.orders.iter().any(|o| o.year == 1751) => {
                let cell = land(g);
                g.give(save::Order::BuyLand { cell }).expect("the land is bought");
            }
            (1751, 5) if g.waiting.is_empty() => {
                let other = (0..g.sim.st.T_max).find(|&t| t != seat && g.sim.st.town_active[t]).expect("another town");
                assert_eq!(g.give(save::Order::MoveSeat { town: other }), Ok(save::Done::Waits));
                let cell = land(g);
                assert_eq!(g.give(save::Order::BuyLand { cell }), Ok(save::Done::Waits));
            }
            _ => {}
        }
    }
    // play on to `months` months into the year after `year`, giving the orders on the way and there
    fn reach(g: &mut save::Game, year: i64, months: usize) {
        loop {
            orders(g);
            if (g.last_year(), g.month) >= (year, months) {
                break;
            }
            if g.month().unwrap() {
                g.turn();
            }
        }
    }
    // through the file's text, as a game on disk goes
    fn again(g: &save::Game) -> save::Game {
        let loaded = save::load(&save::Save::from_text(&g.save(0).to_text()).unwrap(), |_| {}).unwrap();
        assert!(!loaded.drifted && loaded.refused.is_empty(), "{:?}", loaded.refused);
        loaded.game
    }

    let mut played = save::Game::new(3, 6, 0);
    reach(&mut played, 1750, 0);
    let mut at_turn = again(&played);
    reach(&mut played, 1750, 4);
    let mut in_may = again(&played);
    reach(&mut played, 1751, 5);
    assert_eq!((played.orders.len(), played.waiting.len(), played.sim.st.acct.years.len()), (2, 2, 2));
    let mut in_june = again(&played);
    reach(&mut at_turn, 1751, 5);
    reach(&mut in_may, 1751, 5);
    let mut twice = again(&again(&in_may));
    assert_eq!((in_june.waiting.len(), twice.waiting.len()), (2, 2));
    // the year in progress is in what is compared: five months of it are in the books' text
    let open = whole_books(&played.sim);
    assert!(open.contains("month: 5,"), "the books are five months into their year");
    // (one more month moves the text and nothing the books publish: the text sees what the tables do not)
    let mut ahead = played.sim.clone();
    sim::run_month(&mut ahead, 5).unwrap();
    assert_ne!(whole_books(&ahead), open);
    assert_eq!(format!("{:?}", (&ahead.st.acct.years, &ahead.st.acct.hist, &ahead.st.acct.last_months)), format!("{:?}", (&played.sim.st.acct.years, &played.sim.st.acct.hist, &played.sim.st.acct.last_months)));

    let mut others = [("saved at a turn", &mut at_turn), ("saved in May", &mut in_may), ("saved in June", &mut in_june), ("loaded twice", &mut twice)];
    for (year, months, when) in [(1751, 5, "in June 1752"), (1752, 0, "at the turn of 1753"), (1753, 3, "in April 1754")] {
        reach(&mut played, year, months);
        for (how, g) in others.iter_mut() {
            reach(g, year, months);
            same_whole(&played.sim, &g.sim, &format!("{when}, {how}"));
            assert_eq!(played.orders, g.orders, "{when}, {how}: the orders carried out");
        }
    }
    // (the seat moved at the year's end, so the land that waited beside the old one was refused at the turn)
    assert_eq!((played.orders.len(), played.waiting.len(), played.sim.st.acct.years.len()), (3, 0, 4), "what waited was dealt with, and four years are closed");
    assert_ne!(whole_books(&played.sim), open, "and the books moved on");

    // And with no orders, the game's own steps against the simulation's: a
    // game played by the month (`Game::month`) and a run stepped a year at a
    // time (`sim::run_year`) and then by the month, into May of the third
    // year and on through its end.
    let mut by_month = save::Game::new(3, 6, 0);
    let mut by_year = sim::start(&save::params(Rules::Game, 3, 6), Rules::Game.content());
    for _ in 0..2 * 12 + 4 {
        if by_month.month().unwrap() {
            by_month.turn();
        }
    }
    sim::run_year(&mut by_year).unwrap();
    sim::run_year(&mut by_year).unwrap();
    for month in 0..4 {
        sim::run_month(&mut by_year, month).unwrap();
    }
    same_whole(&by_month.sim, &by_year, "in May 1752, by the month and by the year");
    for month in 4..12 {
        assert_eq!(by_month.month().unwrap(), month == 11);
        sim::run_month(&mut by_year, month).unwrap();
    }
    sim::end_year(&mut by_year);
    same_whole(&by_month.sim, &by_year, "at the turn of 1753, by the month and by the year");
}

/// Two months as a viewer would hold them: the same month, wage, price and
/// cash to the last bit, and every series but the books' own (whose figure for
/// a month of a closed year is the year's, written when it closed; for a month
/// of an open year both are NaN, and that is compared too).
fn same_month(sent: &LiveMonth, kept: &LiveMonth, own: &[usize], when: &str) {
    let f32bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    assert_eq!((sent.year, sent.month, sent.index), (kept.year, kept.month, kept.index), "{when}");
    assert_eq!(f32bits(&sent.wage), f32bits(&kept.wage), "{when}: the wages, {}-{}", kept.year, kept.month);
    assert_eq!(f32bits(&sent.price), f32bits(&kept.price), "{when}: the prices, {}-{}", kept.year, kept.month);
    assert_eq!(bits(&sent.cash), bits(&kept.cash), "{when}: the families' cash, {}-{}", kept.year, kept.month);
    let open = own.iter().all(|&i| kept.series[i].is_nan());
    for i in (0..sent.series.len()).filter(|i| open || !own.contains(i)) {
        assert_eq!(sent.series[i].to_bits(), kept.series[i].to_bits(), "{when}: series {i}, {}-{}", kept.year, kept.month);
    }
}

#[test]
fn a_game_gone_on_with_has_the_months_it_did_not_watch() {
    // The game's thread sends every month as it is played (`live_month`), and
    // a viewer keeps each town's prices and wages by the month from them for
    // its price and wage maps. A game gone on with from a save plays its months
    // over before there is a viewer to send them to; the books have kept the
    // last complete year's and the open year's, and `live::months_kept` hands
    // them out. Here they are the months a played game sent, to the last bit:
    // after every month of three years, from the game as it is played; and from
    // the same game saved in June and loaded, at a turn and loaded, and loaded
    // twice. December too: a game sends it once the year has ended, and it is
    // told as it ended, not as the next January opens.
    let mut g = save::Game::new(3, 6, 0);
    let (C, P) = (g.sim.content.clone(), g.sim.params.clone());
    let stat = live::live_static(&g.sim.st, &C);
    let own: Vec<usize> = (0..stat.series.len()).filter(|&i| accounts::own_series(&stat.series[i].name)).collect();
    let mut cursor = live::LiveCursor::new(&C);
    let mut sent: Vec<LiveMonth> = Vec::new();
    let again = |g: &save::Game| {
        let loaded = save::load(&save::Save::from_text(&g.save(0).to_text()).unwrap(), |_| {}).unwrap();
        assert!(!loaded.drifted && loaded.refused.is_empty(), "{:?}", loaded.refused);
        loaded.game
    };
    let d = pending(&g.sim.st, 0);
    g.give(save::Order::decide(&g.sim, d[0].id, 0).unwrap()).unwrap();
    let mut at_turn = None;
    // Decembers whose close the year's end moved, in prices or wages, and in the families' cash
    let (mut moved, mut spent) = (0, 0);
    while g.sim.t < 2 * 12 + 5 {
        // land bought at the turn of 1752, as a player's purchase moves the family's cash between years
        if (g.last_year(), g.month) == (1751, 0) {
            at_turn = Some(again(&g));
            let seat = g.sim.st.families[0].seat as i64;
            let cell = (0..g.sim.st.N).find(|&c| g.sim.st.catch[c] == seat && may_buy_land(&g.sim.st, 0, c).is_ok()).expect("some land to buy at the seat");
            g.give(save::Order::BuyLand { cell }).expect("the land is bought");
        }
        let turned = g.month().unwrap();
        let m = live::live_month(&mut cursor, &g.sim.st, &C, &P, &g.sim.events, g.sim.t - 1);
        // the month just played is the books' newest, and it is the month sent
        let kept = live::months_kept(&g.sim.st, &C);
        assert_eq!(kept.len(), if g.sim.t <= 12 { g.sim.t } else { 12 + g.sim.t % 12 });
        same_month(&m, kept.last().unwrap(), &own, "as it is played");
        if turned {
            // what the island stands at once the year has ended: the next January's opening
            let st = &g.sim.st;
            let opens: Vec<f32> = st.price.data.iter().map(|&p| p as f32).collect();
            let wages: Vec<f32> = (0..st.T_max).map(|i| if st.town_active[i] { st.wage[i] as f32 } else { 0.0 }).collect();
            moved += (opens != m.price || wages != m.wage) as usize;
            spent += st.families.iter().zip(&m.cash).any(|(f, &c)| f.cash != c) as usize;
        }
        sent.push(m);
        if turned {
            let _ = live::live_year(&mut cursor, &g.sim.st, &C, &P, &g.sim.events);
            g.turn();
        }
    }
    assert_eq!((g.last_year(), g.month), (1751, 5));
    // (both Decembers would have been told otherwise as January opened: so the books' close is what was sent)
    assert_eq!((moved, spent), (2, 2), "Decembers whose prices or wages, and whose cash, the year's end moved");
    // a game that is loaded in June has 1751's twelve months and 1752's five, and so has the game that was played
    let in_june = again(&g);
    let twice = again(&in_june);
    let kept = live::months_kept(&in_june.sim.st, &C);
    assert_eq!(kept.iter().map(|m| m.index).collect::<Vec<_>>(), (12..29).collect::<Vec<_>>());
    for (how, other) in [("played", &g), ("loaded twice", &twice)] {
        assert_eq!(format!("{:?}", live::months_kept(&other.sim.st, &C)), format!("{kept:?}"), "{how}");
    }
    for m in kept.iter().filter(|m| m.month != 11) {
        same_month(&sent[m.index], m, &own, "loaded in June");
    }
    // one loaded at the turn of 1752, before the land was bought, has 1751's twelve
    let kept = live::months_kept(&at_turn.unwrap().sim.st, &C);
    assert_eq!(kept.iter().map(|m| m.index).collect::<Vec<_>>(), (12..24).collect::<Vec<_>>());
    for m in kept.iter().filter(|m| m.month != 11) {
        same_month(&sent[m.index], m, &own, "loaded at a turn");
    }
    // without the books there is nothing to hand out
    let mut s = sim::start(&params(Rules::Game, 3, 6, 1751, false), Rules::Game.content());
    sim::run_month(&mut s, 0).unwrap();
    assert!(live::months_kept(&s.st, &s.content).is_empty());
}

#[test]
fn a_copy_of_the_island_shares_the_years_put_away() {
    // An order given in the middle of a year is tried on a copy of the island
    // (`save::Game::give`), and the interface asks for two more. The years the
    // books have put away (33 MB by 1900) are not copied with it: the copy
    // and the game hold the same ones, and whichever closes a year first
    // while the other still stands takes its own.
    use std::sync::Arc;
    let mut g = save::Game::new(3, 6, 0);
    for _ in 0..2 * 12 + 5 {
        if g.month().unwrap() {
            g.turn();
        }
    }
    // (the tables by town are reserved for the whole run; the works' grow by doubling, and may move)
    let room = |s: &sim::Sim| (s.st.acct.hist.goods.as_ptr(), s.st.acct.hist.goods.capacity(), s.st.acct.hist.methods.as_ptr());
    let before = room(&g.sim);
    assert!(before.1 >= 151 * g.sim.st.T_max * g.sim.content.G * KG, "room for every year of the run");

    // the copies an order makes come and go, and the game's history is where it was
    let seat = g.sim.st.families[0].seat;
    let other = (0..g.sim.st.T_max).find(|&t| t != seat && g.sim.st.town_active[t]).expect("another town");
    assert_eq!(g.give(save::Order::MoveSeat { town: other }), Ok(save::Done::Waits));
    assert!(g.foreseen().is_some());
    assert_eq!(room(&g.sim), before);

    // a copy that is kept: it shares the years put away, and has the same books
    let mut copy = g.clone();
    assert!(Arc::ptr_eq(&copy.sim.st.acct.hist, &g.sim.st.acct.hist));
    same_whole(&g.sim, &copy.sim, "a copy");
    let old = whole_books(&g.sim);
    // the copy closes the year first, and takes its own history to write in: the game's is untouched
    copy.year().unwrap();
    assert!(!Arc::ptr_eq(&copy.sim.st.acct.hist, &g.sim.st.acct.hist));
    assert_eq!((copy.sim.st.acct.hist.n, g.sim.st.acct.hist.n), (3, 2));
    assert_eq!(whole_books(&g.sim), old, "what the copy did is not in the game's books");
    assert_eq!(room(&g.sim), before);
    // the game follows, in the history it always had, with no more room asked for; and the two agree
    g.year().unwrap();
    assert_eq!(room(&g.sim), before, "a year is put away where room was kept for it");
    same_whole(&g.sim, &copy.sim, "the game and its copy, a year on");
    // and a game that never was copied says the same
    let mut alone = save::Game::new(3, 6, 0);
    for _ in 0..2 * 12 + 5 {
        if alone.month().unwrap() {
            alone.turn();
        }
    }
    alone.give(save::Order::MoveSeat { town: other }).unwrap();
    alone.year().unwrap();
    same_whole(&g.sim, &alone.sim, "a game that was copied and one that was not");
}

#[test]
fn the_books_do_not_change_what_a_save_is_checked_by() {
    // the fingerprint a save carries is of the island's history, which the
    // books only read: with them or without, a year and three months in, it is
    // the same, so a game saved before there were books is not said to have drifted
    let stand = |books: bool| {
        let mut s = sim::start(&params(Rules::Game, 7, 6, 1752, books), Rules::Game.content());
        sim::run_year(&mut s).unwrap();
        for month in 0..3 {
            sim::run_month(&mut s, month).unwrap();
        }
        s
    };
    let (off, on) = (stand(false), stand(true));
    assert_eq!(on.st.series.names.len(), off.st.series.names.len() + 8);
    assert!(on.st.series.get("acct_share_work")[0].is_finite(), "the books' series are written");
    assert_eq!(save::check(&on), save::check(&off));
}

#[test]
fn a_viewer_that_takes_the_months_as_they_come_is_told_the_years_figures() {
    // As the game's thread does it: a month is played and sent (`live_month`),
    // and after December the year (`live_year`). The books' series have no
    // figure for a month until its year is closed, so the months go out
    // without one, and the year's message brings the year's figure for all
    // twelve. A viewer that writes it in holds exactly the series the
    // simulation holds, which is what a loaded game's viewer reads.
    let mut g = save::Game::new(3, 6, 0);
    let (C, P) = (g.sim.content.clone(), g.sim.params.clone());
    let stat = live::live_static(&g.sim.st, &C);
    let own: Vec<usize> = (0..stat.series.len()).filter(|&i| accounts::own_series(&stat.series[i].name)).collect();
    assert_eq!(own.len(), 8);
    let mut cursor = live::LiveCursor::new(&C);
    let mut held = vec![vec![f64::NAN; 36]; stat.series.len()];
    // the last year's month-end prices as they were sent, `[month][town * G + good]`
    let mut sent_prices: Vec<Vec<f32>> = Vec::new();
    for k in 0..3 {
        sent_prices.clear();
        loop {
            let turned = g.month().unwrap();
            let m = live::live_month(&mut cursor, &g.sim.st, &C, &P, &g.sim.events, g.sim.t - 1);
            assert_eq!((m.index / 12, m.month), (k, sent_prices.len()));
            for (i, v) in m.series.iter().enumerate() {
                held[i][m.index] = *v;
            }
            sent_prices.push(m.price.clone());
            if turned {
                break;
            }
            // a month of a year still open: every other series has its figure, the books' have none
            assert!(own.iter().all(|&i| m.series[i].is_nan()), "{}-{}", m.year, m.month);
            assert!((0..m.series.len()).filter(|i| !own.contains(i)).any(|i| m.series[i].is_finite()));
        }
        let y = live::live_year(&mut cursor, &g.sim.st, &C, &P, &g.sim.events);
        let acc = y.accounts.as_ref().expect("the island's account");
        assert_eq!(y.year_series.iter().map(|&(i, _)| i).collect::<Vec<_>>(), own);
        let figure = |name: &str| y.year_series.iter().find(|&&(i, _)| stat.series[i].name == name).unwrap().1;
        assert_eq!((figure("acct_share_work").to_bits(), figure("acct_ownership_years").to_bits()), (acc.share_work.to_bits(), acc.ownership_years.to_bits()));
        // held as it was sent, the year would be eleven months of nothing: the year's message mends it
        assert!(held[own[0]][k * 12..k * 12 + 11].iter().all(|v| v.is_nan()));
        for &(i, v) in &y.year_series {
            held[i][k * 12..(k + 1) * 12].fill(v);
        }
        g.turn();
    }
    for (i, s) in stat.series.iter().enumerate() {
        assert_eq!(bits(&held[i]), bits(&g.sim.st.series.data[i][..36]), "{}", s.name);
    }

    // The months the books keep of the last complete year and the months a
    // viewer was sent say the same price, the one each month ended at, for
    // every town that stood all year. (The books' is single precision, as the
    // message's is.) December too: a game sends it once the year has ended
    // (`save::Game::month` ends it), when districts that changed market have
    // taken their prices with them (`peopling::move_with_cells`), and it is
    // told from the books, at the price December's trade ended at.
    let st = &g.sim.st;
    assert_eq!(sent_prices.len(), 12);
    for t in (0..st.town_name.len()).filter(|&t| st.town_founded[t] < 1752) {
        let months = accounts::town_books(st, &C, t, None).unwrap().months.expect("the last year's months");
        for (mo, sent) in sent_prices.iter().enumerate() {
            for good in 0..C.G {
                assert_eq!((months[mo][good][0] as f32).to_bits(), sent[t * C.G + good].to_bits(), "town {t}, month {mo}, {}", C.good_name[good]);
            }
        }
    }
}

#[test]
fn a_run_of_one_year_and_towns_not_yet_founded() {
    let r = run(Rules::Game, &params(Rules::Game, 4, 0, 1750, true));
    let (st, C) = (&r.state, &r.content);
    assert_eq!((st.acct.years.len(), st.acct.hist.n), (1, 1));
    let y = &st.acct.years[0];
    assert!(y.balance_goods < 1e-9 && y.balance_income < 1e-9 && y.balance_spending < 1e-9);
    assert!(y.to_families.is_empty() && y.div_families == 0.0 && y.works_value_families == 0.0);
    // a slot with no town in it yet has empty books, with the prices it stands at
    let empty = (0..st.T_max).find(|&t| !st.town_active[t]).expect("some slot is still empty");
    let rec = st.acct.year(0).unwrap();
    let row = &rec.towns[empty * KT..(empty + 1) * KT];
    assert!(row.iter().all(|&v| v == 0.0), "an empty slot has figures: {row:?}");
    for g in 0..C.G {
        let v = |name: &str| rec.goods[(empty * C.G + g) * KG + col(name)];
        assert!(v("made") == 0.0 && v("stock") == 0.0 && v("price").is_finite() && v("price_low") == v("price_high"));
    }
    assert!(accounts::town_books(st, C, empty, None).is_none(), "and the viewer is not handed them");
    // every number kept is a number
    let h = &st.acct.hist;
    for (name, table) in [("goods", &h.goods), ("land", &h.land), ("methods", &h.methods), ("towns", &h.towns), ("needs", &h.needs), ("works", &h.works)] {
        assert!(table.iter().all(|v| v.is_finite()), "the {name} table holds something that is not a number");
    }
}

#[test]
fn the_books_agree_with_the_observer_and_the_families() {
    let r = run(Rules::Port, &params(Rules::Port, 3, 6, 1779, true));
    let (st, C) = (&r.state, &r.content);
    let y0 = C.start_year;
    let (T, G, M) = (st.T_max, C.G, C.M);
    for (k, y) in st.acct.years.iter().enumerate() {
        let rec = st.acct.year(k).unwrap();
        // what the towns made is what the observer's output series say, the
        // timber of cleared woodland aside (the observer does not see it)
        for g in 0..G {
            let made: f64 = (0..T).map(|t| (rec.goods[(t * G + g) * KG + col("made")] - rec.goods[(t * G + g) * KG + col("cleared")]) as f64).sum();
            let series = 12.0 * annual(st.series.get(&format!("out_{}", C.good_name[g])), y0, y.year);
            assert!((made - series).abs() <= 1e-5 * series + 1e-6, "{} {}: {made} in the books, {series} in the series", y.year, C.good_name[g]);
        }
        for m in 0..M {
            let made: f64 = (0..T).map(|t| rec.methods[(t * M + m) * accounts::KM] as f64).sum();
            let series = 12.0 * annual(st.series.get(&format!("mout_{}", C.method_name[m])), y0, y.year);
            assert!((made - series).abs() <= 1e-5 * series + 1e-6, "{} {}: {made} against {series}", y.year, C.method_name[m]);
        }
        // spending is the observer's output in money
        let gdp = 12.0 * annual(st.series.get("gdp_nom"), y0, y.year);
        assert!((y.spending - gdp).abs() < 1e-9 * gdp, "{}: {} spent, output {gdp}", y.year, y.spending);
        // the headline series hold the year's figure in all its months
        assert!((annual(st.series.get("acct_share_work"), y0, y.year) - y.share_work).abs() < 1e-12);
        assert_eq!(st.series.get("acct_share_land")[k * 12 + 5].to_bits(), y.share_land.to_bits());
        assert!((annual(st.series.get("acct_ownership_years"), y0, y.year) - y.ownership_years).abs() < 1e-12);
    }
    // the families' own books of the last year say what ours say they received
    let last = st.acct.years.last().unwrap();
    assert_eq!(last.to_families.len(), 6);
    for (fam, got) in st.families.iter().zip(&last.to_families) {
        let own = fam.last_year.earned();
        assert!((got.dividends + got.rents - own).abs() <= 1e-9 * own.abs().max(1.0), "the {} family: {} in the books, {own} in its own", fam.surname, got.dividends + got.rents);
        assert!((got.rents - fam.last_year.rents).abs() <= 1e-9 * fam.last_year.rents.abs().max(1.0));
    }
    assert!(last.works_value_families > 0.0 && last.works_value_families < last.works_value);
    assert!(last.land_value_families > 0.0 && last.land_value_families < last.land_value);
    let m = &last.money;
    let money = m.households + m.landlords + m.merchants + m.works + m.families + m.projects + m.fund;
    assert!((money - econ::money_total(st)).abs() < 1e-9 * money, "the purses hold all the money there is");
}

#[test]
fn a_town_and_a_works_can_be_asked_for() {
    let r = run(Rules::Port, &params(Rules::Port, 7, 6, 1764, true));
    let (st, C) = (&r.state, &r.content);
    let iron = C.gi("iron");
    // a town's books for a past year and for the last
    let old = accounts::town_books(st, C, 0, Some(1755)).expect("town 0 in 1755");
    let last = accounts::town_books(st, C, 0, None).expect("town 0 in the last year");
    assert_eq!((old.year, last.year, last.town), (1755, 1764, 0));
    assert_eq!((old.goods.len(), old.land.len(), old.needs.len()), (C.G, C.NS, C.S));
    assert!(old.months.is_none() && last.months.as_ref().is_some_and(|m| m.len() == 12 && m[0].len() == C.G));
    assert!(accounts::town_books(st, C, 0, Some(1765)).is_none() && accounts::town_books(st, C, 0, Some(1749)).is_none());
    assert!(accounts::town_books(st, C, st.T_max, None).is_none());
    let g = &last.goods[iron];
    assert!(g.made > 0.0 && g.bought > 0.0 && g.unmet >= 0.0 && g.anchor.is_none());
    assert!(g.unplaced.abs() < 1e-4 * (g.stock_open + g.made + g.received));
    // the last year's months add up to the year
    let months = last.months.as_ref().unwrap();
    let made: f64 = months.iter().map(|m| m[iron][1]).sum();
    assert!((made - g.made).abs() < 1e-4 * g.made);
    assert!((months[11][iron][3] - (g.stock - g.moved)).abs() < 1e-4 * g.stock.max(1.0), "December's stock is the year's, less what came with a district at the year's end");
    // the methods that made the iron account for all of it, and their sums add up
    let by_methods: f64 = last.methods.iter().filter(|m| m.good == iron).map(|m| m.out).sum();
    assert!((by_methods - g.made).abs() < 1e-4 * g.made);
    for m in &last.methods {
        let spent = m.inputs + m.fuel + m.cart_in + m.wages + m.rent + m.managers + m.profit;
        let revenue = m.value * (1.0 - r.params.merchant_spread) - m.cart_out;
        assert!((spent - revenue).abs() < 1e-4 * revenue.abs().max(1.0), "{}: {spent} against {revenue}", m.name);
    }
    // what the methods used, good by good, is what the town's works bought
    for good in &last.goods {
        let used: f64 = last.methods.iter().flat_map(|m| &m.uses).filter(|u| u.good == good.good).map(|u| u.used).sum();
        let burnt: f64 = last.methods.iter().flat_map(|m| &m.uses).filter(|u| u.good == good.good).map(|u| u.burnt).sum();
        assert!((used - good.works_inputs).abs() <= 1e-4 * used + 1e-6 && (burnt - good.works_fuel).abs() <= 1e-4 * burnt + 1e-6, "{}", good.name);
    }
    assert!(last.methods.iter().any(|m| m.uses.iter().any(|u| u.burnt > 0.0)), "some works burns fuel");
    for l in last.land.iter().filter(|l| l.out > 0.0) {
        let spent = l.wages + l.carting + l.inputs + l.rent;
        assert!((spent - l.value * (1.0 - r.params.merchant_spread)).abs() < 1e-4 * l.value);
    }
    let fig = |name: &str| last.figures.iter().find(|(n, _)| n == name).unwrap().1;
    assert!(fig("pop") > 0.0 && fig("wage") > 0.0 && fig("pay_land") > 0.0 && fig("spend_households") > 0.0);
    assert!(last.needs.iter().map(|n| n.households).sum::<f64>() > 0.99 * fig("spend_households"));

    // a works: every year it stood, and its sums add up
    let rec = st.acct.year(st.acct.hist.n - 1).unwrap();
    let busiest = (0..rec.works_id.len()).max_by(|&a, &b| rec.works[a * KW + wcol("inputs")].total_cmp(&rec.works[b * KW + wcol("inputs")])).unwrap();
    let f = rec.works_id[busiest] as usize;
    let wb = accounts::works_books(st, C, f).expect("the works' books");
    assert_eq!(wb.name, st.f_name[f]);
    assert!(!wb.years.is_empty() && wb.years.last().unwrap().year == 1764);
    assert!(wb.years.iter().all(|y| y.year >= st.f_born[f].max(1750)));
    for y in &wb.years {
        let left = y.revenue - y.inputs - y.fuel - y.cart_in - y.wages - y.rent - y.managers - y.profit;
        assert!(left.abs() < 1e-4 * y.revenue.abs().max(1.0), "{}: {left} of revenue {} unplaced", y.year, y.revenue);
        assert!(y.util >= 0.0 && y.util <= 1.0 + 1e-6 && y.months <= 12.0);
    }
    assert!(!wb.inputs.is_empty() && wb.inputs.iter().all(|i| i.cost > 0.0));
    assert!(accounts::works_books(st, C, st.F_count).is_none());
    // with the works' part switched off the towns' books are kept and no works'
    let mut P = params(Rules::Port, 7, 0, 1752, true);
    P.accounts.works = false;
    let r = run(Rules::Port, &P);
    assert!(r.state.acct.hist.works_id.is_empty() && r.state.acct.hist.n == 3 && !r.state.acct.year(2).unwrap().goods.is_empty());
    assert!(accounts::works_books(&r.state, &r.content, 0).is_some_and(|w| w.years.is_empty()));
}

#[test]
fn the_books_repeat_and_go_into_the_run_file_and_the_live_view() {
    let P = params(Rules::Game, 3, 6, 1762, true);
    let (a, b) = (run(Rules::Game, &P), run(Rules::Game, &P));
    assert_eq!(a.state.acct.years, b.state.acct.years);
    let f32bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<_>>();
    let (x, y) = (&a.state.acct.hist, &b.state.acct.hist);
    assert_eq!(f32bits(&x.goods), f32bits(&y.goods));
    assert_eq!(f32bits(&x.methods), f32bits(&y.methods));
    assert_eq!(f32bits(&x.towns), f32bits(&y.towns));
    assert_eq!(f32bits(&x.works), f32bits(&y.works));
    assert_eq!((&x.works_id, &x.trade, &x.uses, &x.works_off), (&y.works_id, &y.trade, &y.uses, &y.works_off));
    assert_eq!(x.n, 13);
    let blob = record::pack(&a.state, &a.content, &a.params, &a.events, &a.rec, "a", false);
    assert!(blob == record::pack(&b.state, &b.content, &b.params, &b.events, &b.rec, "a", false), "identical runs pack to different bytes");

    // the run file: the tables with their shapes, their columns and the island's years
    // (of the town slots only those in use are written: the rest would be zeros)
    let h = header(&blob);
    let (n, T, G) = (13, a.state.town_name.len(), a.content.G);
    assert!(T < a.state.T_max && h["accounts"]["towns"] == T);
    for table in accounts::tables(&a.state, false) {
        match table {
            accounts::Table::F32(name, shape, data) if name == "acct_towns" => {
                assert_eq!(shape, vec![13, T, KT]);
                for k in 0..13 {
                    assert_eq!(f32bits(&data[k * T * KT..(k + 1) * T * KT]), f32bits(&a.state.acct.year(k).unwrap().towns[..T * KT]), "year {k}");
                }
            }
            _ => {}
        }
    }
    let shape = |name: &str| -> Vec<u64> {
        let arr = h["arrays"].as_array().unwrap().iter().find(|x| x["name"] == name).unwrap_or_else(|| panic!("no array {name}"));
        arr["shape"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect()
    };
    assert_eq!(shape("acct_goods"), vec![n, T as u64, G as u64, KG as u64]);
    assert_eq!(shape("acct_land"), vec![n, T as u64, a.content.NS as u64, accounts::KL as u64]);
    assert_eq!(shape("acct_methods"), vec![n, T as u64, a.content.M as u64, accounts::KM as u64]);
    assert_eq!(shape("acct_towns"), vec![n, T as u64, accounts::KT as u64]);
    assert_eq!(shape("acct_needs"), vec![n, T as u64, a.content.S as u64, 2]);
    assert_eq!(shape("acct_trade_off"), vec![n + 1]);
    assert_eq!(shape("acct_trade_key")[1], 3);
    assert_eq!(shape("acct_use_off"), vec![n + 1]);
    assert_eq!(shape("acct_use_key")[0], shape("acct_use_val")[0]);
    assert_eq!(shape("acct_works_off"), vec![n + 1]);
    assert_eq!(shape("acct_works")[1], KW as u64);
    assert_eq!(shape("acct_works")[0], shape("acct_works_id")[0]);
    let books = &h["accounts"];
    assert_eq!(books["goods_fields"].as_array().unwrap().len(), KG);
    assert_eq!(books["island"].as_array().unwrap().len(), n as usize);
    assert!(books["island"][0]["share_work"].as_f64().unwrap() > 0.0);
    // under the game's rules the merchants have an anchor, and the books keep it
    let anchor = a.state.acct.year(5).unwrap().goods[col("anchor")];
    assert!(anchor > 0.0);
    // a lite pack keeps the towns' tables and drops the works'
    let lite = header(&record::pack(&a.state, &a.content, &a.params, &a.events, &a.rec, "a", true));
    let names: Vec<&str> = lite["arrays"].as_array().unwrap().iter().map(|x| x["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"acct_goods") && !names.contains(&"acct_works"));

    // the live view: each year's message carries that year's island account
    let mut s = sim::start(&P, Rules::Game.content());
    let mut cursor = live::LiveCursor::new(&s.content);
    for year in 1750..=1752 {
        sim::run_year(&mut s).unwrap();
        let y = live::live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        let acc = y.accounts.expect("the island's account");
        assert_eq!(acc.year, year);
        assert_eq!(acc, a.state.acct.years[(year - 1750) as usize]);
    }
}
