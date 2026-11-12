//! Families (not in the Python): money is conserved and runs are deterministic
//! with families on, the player's choices work through the API, AI families play
//! by the same rules, and the live API reports every year once. With families
//! off the parity tests (tests/parity.rs) cover that nothing changed.
#![allow(non_snake_case)]

use std::collections::BTreeSet;

use isostate_core::families::{self, Strategy};
use isostate_core::*;

fn params(n_families: usize, end_year: i64) -> Params {
    let mut P = default_params();
    P.n_families = n_families;
    P.end_year = Some(end_year);
    P
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1.0)
}

fn report(s: &Sim, k: usize) -> FamilyReport {
    family_report(&s.st, &s.content, &s.params, k).unwrap()
}

#[test]
fn families_conserve_money_and_runs_are_deterministic() {
    // six families for sixty years; run_year checks the money stock every month
    let P = params(6, 1809);
    let a = sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
    let b = sim::run_with(&P, default_content(), false, true).expect("money is conserved every month");
    let st = &a.state;
    let drift = econ::money_total(st) - (st.money_start + st.issued);
    assert!(drift.abs() <= 1e-9 * econ::money_total(st), "money drifted by {drift:e}");
    for (x, y) in a.state.series.data.iter().zip(&b.state.series.data) {
        assert_eq!(bits(x), bits(y), "the same parameters give the same run");
    }
    assert_eq!(a.state.f_owner, b.state.f_owner);
    assert_eq!(a.state.cell_owner, b.state.cell_owner);
    for (fa, fb) in a.state.families.iter().zip(&b.state.families) {
        assert_eq!(fa.cash.to_bits(), fb.cash.to_bits());
        assert_eq!(fa.members.len(), fb.members.len());
    }
    let (works, rent, money) = families::ownership_shares(st, &a.content);
    assert!(works > 0.0 && rent > 0.0 && money > 0.0, "families own something: {works} {rent} {money}");
    assert!(works < 0.5 && rent < 0.5 && money < 0.5, "families do not own most of it: {works} {rent} {money}");
    for (k, f) in st.families.iter().enumerate() {
        let heads = f.members.iter().filter(|m| m.alive && m.post == Post::Head).count();
        assert_eq!(heads, 1, "family {k} has one head");
        let head = f.members.iter().find(|m| m.alive && m.post == Post::Head).unwrap();
        assert_eq!(head.town, f.seat, "the head lives at the seat");
        assert!(f.members.iter().filter(|m| m.alive).count() <= families::FULL_FAMILY + 2);
        for m in f.members.iter().filter(|m| m.alive) {
            match m.post {
                Post::Child => assert!(m.age < 18),
                Post::Manager { group } => assert!(f.known[group.index()], "managers run known trades"),
                _ => {}
            }
        }
    }
    // people: children were born, came of age, trained; rivals learned new trades
    assert!(st.families.iter().any(|f| f.members.iter().any(|m| m.trade.is_some())), "apprentices finished");
    assert!(st.families[1..].iter().any(|f| f.known.iter().filter(|&&x| x).count() > 1), "a rival learned a second trade");
    assert!(a.events.iter().any(|e| e.kind == "family" && e.title.contains("is born")));
}

#[test]
fn the_player_chooses_from_short_lists() {
    let P = params(3, 1770);
    let mut s = sim::start(&P, default_content());
    let C = s.content.clone();
    assert_eq!(s.st.families[0].strategy, Strategy::Player);
    // the first decision: the trade, defaulting to the seat's strongest
    let d = pending(&s.st, 0);
    assert_eq!(d.len(), 1);
    assert_eq!(d[0].options.len(), 6);
    let textiles = d[0].options.iter().position(|o| o.choice == Choice::Trade { group: Group::Textiles }).unwrap();
    decide(&mut s.st, &C, &s.params, 0, d[0].id, textiles, &mut s.events).unwrap();
    assert!(pending(&s.st, 0).is_empty());
    let r = report(&s, 0);
    assert_eq!(r.known, vec![Group::Textiles]);
    assert_eq!(r.first_trade, Group::Textiles);
    assert!(decide(&mut s.st, &C, &s.params, 0, d[0].id, 0, &mut s.events).is_err(), "a decision is answered once");
    sim::run_year(&mut s).unwrap();

    // works are chosen directly only at the seat, only in a known trade
    let seat = s.st.families[0].seat;
    let seat_cell = (0..s.st.N).find(|&c| s.st.catch[c] == seat as i64 && site_options(&s.st, &C, &s.params, 0, c).is_ok()).unwrap();
    let options = site_options(&s.st, &C, &s.params, 0, seat_cell).unwrap();
    let loom = options.iter().find(|o| o.name == "hand_loom" || o.name == "textile_mill").unwrap();
    let iron = options.iter().find(|o| o.group == Some(Group::Iron)).unwrap();
    assert!(iron.allowed.as_ref().unwrap_err().contains("does not know"));
    let away = (0..s.st.N).find(|&c| s.st.catch[c] >= 0 && s.st.catch[c] != seat as i64 && site_options(&s.st, &C, &s.params, 0, c).is_ok()).unwrap();
    let far = site_options(&s.st, &C, &s.params, 0, away).unwrap();
    assert!(far.iter().all(|o| o.allowed.is_err()), "nothing is chosen directly away from the seat");
    let money0 = econ::money_total(&s.st);
    if loom.allowed.is_ok() {
        let fid = found_works(&mut s.st, &C, &s.params, 0, loom.method, seat_cell, s.year, &mut s.events).unwrap();
        assert!(s.st.f_owner[fid] == 0 && s.st.f_reason[fid].kind == "family");
        assert!(rel(econ::money_total(&s.st), money0) < 1e-12);
        let r = report(&s, 0);
        let seat_branch = r.branches.iter().find(|b| b.town == seat && b.group == Group::Textiles).unwrap();
        assert!(seat_branch.works.contains(&fid) && seat_branch.runner == r.head && seat_branch.bonus > 0.0);
    }
    assert!(found_works(&mut s.st, &C, &s.params, 0, iron.method, seat_cell, s.year, &mut s.events).is_err());
    assert!(found_works(&mut s.st, &C, &s.params, 0, loom.method, away, s.year, &mut s.events).unwrap_err().contains("seat"));

    // land is bought directly only around the seat
    let near = (0..s.st.N).find(|&c| s.st.catch[c] == seat as i64 && s.st.cell_owner[c] < 0 && s.st.world.land[c]).unwrap();
    assert!(may_buy_land(&s.st, 0, near).is_ok());
    assert!(may_buy_land(&s.st, 0, away).unwrap_err().contains("only around the seat"));
    assert!(buy_land(&mut s.st, &C, &s.params, 0, away).unwrap_err().contains("only around the seat"));

    // posting: an adult to a branch in another town, from next year
    let r = report(&s, 0);
    let other = (0..s.st.T_max).find(|&t| s.st.town_active[t] && t != seat).unwrap();
    let head = r.head.unwrap();
    assert!(post_member(&mut s.st, 0, head, Posting::Manager { group: Group::Textiles, town: other }).unwrap_err().contains("seat"));
    let adult = r.members.iter().position(|m| m.alive && m.age >= 18 && m.post == Post::Unposted);
    let child = r.members.iter().position(|m| m.alive && m.age < 14);
    if let Some(c) = child {
        assert!(post_member(&mut s.st, 0, c, Posting::Manager { group: Group::Textiles, town: other }).unwrap_err().contains("too young"));
    }
    if let Some(a) = adult {
        assert!(post_member(&mut s.st, 0, a, Posting::Manager { group: Group::Iron, town: other }).unwrap_err().contains("does not know"));
        post_member(&mut s.st, 0, a, Posting::Manager { group: Group::Textiles, town: other }).unwrap();
        assert_eq!(report(&s, 0).members[a].next, Some((other, Post::Manager { group: Group::Textiles })));
        assert!(set_stance(&mut s.st, 0, Group::Textiles, other, Stance::Hold).is_err(), "no branch there yet");
        sim::run_year(&mut s).unwrap();
        let r = report(&s, 0);
        assert_eq!(r.members[a].town, other);
        let b = r.branches.iter().find(|b| b.town == other && b.group == Group::Textiles).expect("a branch where the manager went");
        assert_eq!(b.runner, Some(a));
        set_stance(&mut s.st, 0, Group::Textiles, other, Stance::Hold).unwrap();
        assert_eq!(report(&s, 0).branches.iter().find(|b| b.town == other).unwrap().stance, Stance::Hold);
    }

    // moving the seat takes effect at the year end; the head moves with it
    move_seat(&mut s.st, 0, other).unwrap();
    assert_eq!(report(&s, 0).next_seat, Some(other));
    sim::run_year(&mut s).expect("money is conserved");
    let r = report(&s, 0);
    assert_eq!(r.seat, other);
    assert_eq!(r.members[r.head.unwrap()].town, other);
    // works left behind at the old seat now run under hired managers
    let left: Vec<usize> = r.firms.iter().copied().filter(|&f| s.st.f_town[f] == seat).collect();
    assert!(left.iter().all(|f| r.absentee.contains(f)));

    // a child turning fourteen raises a training decision that defaults to the first
    // trade at the seat while nobody else is in it, and otherwise to a trade the
    // family neither knows nor is learning
    while s.year <= s.end_year {
        sim::run_year(&mut s).unwrap();
        let r = report(&s, 0);
        if let Some(d) = r.pending.iter().find(|d| d.member.is_some()) {
            let j = d.member.unwrap();
            assert!(d.options.iter().any(|o| o.choice == Choice::University));
            let learning = |g: Group| {
                r.members.iter().any(|m| {
                    m.alive && (m.trade == Some(g) || matches!(m.post, Post::Learning { path: Path::Apprentice { group, .. }, .. } if group == g))
                })
            };
            match d.options[d.default].choice {
                Choice::Apprentice { group, town } if !learning(r.first_trade) => assert!(group == r.first_trade && town == r.seat),
                Choice::Apprentice { group, .. } => assert!(!r.known.contains(&group) && !learning(group), "a new trade, not {group:?}"),
                c => panic!("the default is an apprenticeship, not {c:?}"),
            }
            sim::run_year(&mut s).unwrap();
            let m = &report(&s, 0).members[j];
            assert!(matches!(m.post, Post::Learning { .. }), "the default training started");
            assert!(m.training_until.is_some());
            break;
        }
    }
}

#[test]
fn a_passive_family_trains_its_children_in_different_trades() {
    // every decision left to its default for forty years
    let P = params(3, 1789);
    let mut s = sim::start(&P, default_content());
    let mut saved = (0.0, 0.0);
    while s.year <= s.end_year {
        sim::run_year(&mut s).unwrap();
        if s.year - 1 < s.content.start_year + 10 {
            let ly = report(&s, 0).last_year;
            saved.0 += ly.spending;
            saved.1 += ly.earned();
        }
    }
    let r = report(&s, 0);
    let trades: Vec<Group> = r.members.iter().filter_map(|m| m.trade).collect();
    assert!(trades.len() >= 2, "children were trained: {trades:?}");
    let distinct: BTreeSet<Group> = trades.iter().copied().collect();
    assert!(distinct.len() >= 2 && r.known.len() >= 2, "they learned more than the first trade: {trades:?}");
    let mut places = BTreeSet::new();
    for m in r.members.iter().filter(|m| m.alive) {
        if let Post::Manager { group } = m.post {
            assert!(places.insert((group, m.town)), "two managers of one branch: {group:?} at {}", m.town_name);
        }
    }
    // a landed family saves a real share of its rents (cash is not spent down to nothing)
    let rate = 1.0 - saved.0 / saved.1;
    assert!((0.2..0.4).contains(&rate), "saved {:.0}% of its income in its first ten years", 100.0 * rate);
}

#[test]
fn branches_wind_down_one_holding_a_year() {
    let P = params(3, 1765);
    let mut s = sim::start(&P, default_content());
    // the passive player's seat branch grows in its default trade for a few years
    for _ in 0..8 {
        sim::run_year(&mut s).unwrap();
    }
    let r = report(&s, 0);
    let (g, seat) = (r.first_trade, r.seat);
    let held = |s: &Sim| -> (Vec<usize>, Vec<usize>) {
        let r = report(s, 0);
        let works = r.firms.iter().copied().filter(|&f| s.st.f_town[f] == seat).collect();
        let cells = r.cells.iter().copied().filter(|&c| s.st.catch[c] == seat as i64).collect();
        (works, cells)
    };
    let (w0, c0) = held(&s);
    assert!(!w0.is_empty() || !c0.is_empty(), "the passive player's seat branch grew");
    set_stance(&mut s.st, 0, g, seat, Stance::WindDown).unwrap();
    for _ in 0..2 {
        let (w, c) = held(&s);
        sim::run_year(&mut s).expect("money is conserved");
        let (w1, c1) = held(&s);
        let sold_works = w.iter().filter(|f| !w1.contains(f) && s.st.f_alive[**f]).count();
        let sold_cells = c.iter().filter(|x| !c1.contains(x) && s.st.catch[**x] == seat as i64).count();
        assert!(sold_works + sold_cells <= 1, "at most one holding sold a year");
        assert!(w1.len() <= w.len() && c1.len() <= c.len(), "nothing new while winding down");
    }
    let r = report(&s, 0);
    assert!(r.last_year.sales >= 0.0);
}

#[test]
fn ai_families_play_by_the_same_rules() {
    let P = params(6, 1785);
    let mut s = sim::start(&P, default_content());
    let cells0: Vec<usize> = (0..6).map(|k| s.st.cell_owner.iter().filter(|&&o| o == k as i64).count()).collect();
    while s.year <= s.end_year {
        sim::run_year(&mut s).unwrap();
    }
    let st = &s.st;
    let built: Vec<usize> = (0..st.F_count).filter(|&f| st.f_reason[f].kind == "family").collect();
    assert!(built.iter().any(|&f| st.f_owner[f] >= 1), "AI branches found works");
    let mut grew = 0;
    for k in 1..6 {
        let f = &st.families[k];
        assert!(f.decisions.is_empty(), "AI families answer their own decisions");
        let cells = st.cell_owner.iter().filter(|&&o| o == k as i64).count();
        if cells > cells0[k] || built.iter().any(|&w| st.f_owner[w] == k as i64) {
            grew += 1;
        }
    }
    assert!(grew >= 3, "most AI families bought land or founded works ({grew} of 5)");
    assert!(st.families[1..].iter().any(|f| f.members.iter().any(|m| m.alive && matches!(m.post, Post::Manager { .. }))), "AI families post managers");
}

#[test]
fn live_api_reports_every_year_once() {
    let P = params(6, 1755);
    let mut s = sim::start(&P, default_content());
    let stat = live_static(&s.st, &s.content);
    let N = stat.W * stat.H;
    assert!(stat.elev.len() == N && stat.terrain.len() == N && stat.river.len() == N && stat.down.len() == N);
    assert!(stat.fields.len() == 7 && stat.fields.iter().all(|f| f.len() == N));
    assert_eq!(stat.series.len(), s.st.series.names.len());
    assert!(stat.groups.len() == 6 && stat.layers.len() == 9 && stat.methods.len() == s.content.M);
    let mut cursor = LiveCursor::new(&s.content);
    let mut firms_seen = BTreeSet::new();
    let mut towns_seen = BTreeSet::new();
    let mut n_events = 0;
    // what each year took to simulate, and what live_year then took to report it
    let (mut years, mut reports) = (Vec::new(), Vec::new());
    while s.year <= s.end_year {
        let t = std::time::Instant::now();
        sim::run_year(&mut s).unwrap();
        years.push(t.elapsed().as_secs_f64());
        let t = std::time::Instant::now();
        let y = live_year(&mut cursor, &s.st, &s.content, &s.params, &s.events);
        reports.push(t.elapsed().as_secs_f64());
        assert_eq!(y.year, s.year - 1);
        assert_eq!(y.index as i64, y.year - s.content.start_year);
        for l in [&y.layers.land_use, &y.layers.mix_a, &y.layers.mix_s, &y.layers.rent, &y.layers.catch] {
            assert_eq!(l.len(), N);
        }
        assert_eq!(&y.layers.mix_a[..], &s.rec.layer("mix_a")[y.index * N..(y.index + 1) * N], "the same layers as run.bin");
        assert!(y.new_firms.iter().all(|f| firms_seen.insert(f.id)), "each firm is sent once");
        assert!(y.new_towns.iter().all(|t| towns_seen.insert(t.id)));
        assert!(y.firm_K.iter().all(|&(f, _)| s.st.f_alive[f]));
        assert_eq!(y.series.len(), stat.series.len());
        assert_eq!(y.families.len(), 6);
        assert!(y.families.iter().all(|f| f.head.is_some() && !f.branches.is_empty()));
        assert_eq!(y.cell_owner.len(), N);
        n_events += y.events.len();
    }
    assert_eq!(firms_seen.len(), s.st.F_count);
    assert_eq!(n_events, s.events.len(), "each event is sent once");
    // live_year costs less than a quarter of the year it reports (about a hundredth on a quiet
    // machine: 1.1 ms against 120). Set against the year measured beside it, so that a busy machine
    // slows both alike (it used to be held to 100 ms, and took 633 with two dozen builds running),
    // and the middle year of each, so that one year that was interrupted does not decide it
    let middle = |v: &mut Vec<f64>| {
        v.sort_by(f64::total_cmp);
        v[v.len() / 2]
    };
    let (year, report) = (middle(&mut years), middle(&mut reports));
    assert!(report < year / 4.0, "live_year took {:.1} ms to report a year that took {:.1} ms to simulate (the middle of {} years)", report * 1e3, year * 1e3, years.len());
    assert!(s.events.iter().filter(|e| e.family == Some(0)).all(|e| e.weight >= 0.6), "the player's own news weighs at least 0.6");
}

#[test]
fn the_first_decision_waits_for_the_player() {
    // a viewer runs 1750 before the player sees the map
    let P = params(3, 1760);
    let mut s = sim::start(&P, default_content());
    let C = s.content.clone();
    let cells0 = report(&s, 0).cells.len();
    sim::run_year(&mut s).unwrap();
    let d = pending(&s.st, 0);
    assert_eq!(d.len(), 1, "the first-trade decision survives the first year");
    assert_eq!(d[0].due, s.content.start_year + 1);
    let r = report(&s, 0);
    assert!(r.firms.is_empty() && r.cells.len() == cells0, "nothing is founded or bought before it is answered");
    // answered between years, on the state the viewer sees after 1750
    let g = d[0].options.iter().position(|o| o.choice == Choice::Trade { group: Group::Textiles }).unwrap();
    decide(&mut s.st, &C, &s.params, 0, d[0].id, g, &mut s.events).unwrap();
    let r = report(&s, 0);
    assert_eq!(r.known, vec![Group::Textiles]);
    assert!(r.members.iter().all(|m| !matches!(m.post, Post::Learning { path: Path::Apprentice { group, .. }, .. } if group != Group::Textiles)));
    sim::run_year(&mut s).unwrap();
    assert!(pending(&s.st, 0).iter().all(|d| d.member.is_some()), "only training decisions can be pending now");

    // left unanswered, it takes its default at the end of 1751
    let mut t = sim::start(&P, default_content());
    sim::run_year(&mut t).unwrap();
    sim::run_year(&mut t).unwrap();
    assert!(pending(&t.st, 0).iter().all(|d| d.member.is_some()), "the default was taken at the end of 1751");
}

#[test]
fn a_head_the_player_chooses_is_news() {
    // The head dies in 1750 with grown children about him: the eldest heads the
    // family at the year's end, which is news, and the player is asked who
    // should. Choosing another is news too (it used to be made and thrown away).
    let P = params(3, 1760);
    let mut s = sim::start(&P, default_content());
    let C = s.content.clone();
    let fam = &mut s.st.families[0];
    let head = families::head(fam).unwrap();
    for (j, m) in fam.members.iter_mut().enumerate() {
        if j != head {
            (m.age, m.post) = (30 + j as i64, Post::Unposted);
        }
    }
    fam.members[head].alive = false;
    sim::run_year(&mut s).unwrap();
    let d = pending(&s.st, 0).into_iter().find(|d| d.prompt.starts_with("Who heads")).expect("the player is asked who heads the family");
    let fam = &s.st.families[0];
    let eldest = families::head(fam).unwrap();
    assert!(d.options.len() > 1 && d.options[0].choice == Choice::Head { member: eldest });
    let Choice::Head { member: other } = d.options[1].choice else { panic!("{:?}", d.options[1]) };
    let (was, now) = (fam.members[eldest].first.clone(), fam.members[other].first.clone());
    let told = s.events.len();
    // the eldest again is no change and no news
    let mut same = s.clone();
    decide(&mut same.st, &C, &same.params, 0, d.id, 0, &mut same.events).unwrap();
    assert_eq!(same.events.len(), told);
    decide(&mut s.st, &C, &s.params, 0, d.id, 1, &mut s.events).unwrap();
    assert_eq!(families::head(&s.st.families[0]), Some(other));
    assert_eq!(s.events.len(), told + 1, "the new head is news");
    let e = s.events.last().unwrap();
    assert!(e.title.starts_with(&format!("{now} ")) && e.title.ends_with(" heads the family") && e.text.contains(&format!("in place of {was}")), "{e:?}");
    assert!(e.family == Some(0) && e.kind == "family" && e.year == 1750.95, "{e:?}");
}
