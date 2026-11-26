//! The saved game end to end, without a window: a game is played through the
//! simulation thread as the interface plays it (orders over the channel, a
//! month at a time), the thread's own save is loaded as `--load` loads it, and
//! everything the viewer holds of the two is compared.

use super::*;
use crate::family::FamilyRow;
use isostate_core::families::Post;

/// One game as the app holds it, with the thread's answers taken in the way
/// `game::pump` takes them in.
struct Table {
    r: Replay,
    live: Live,
    view: FamilyView,
    /// each town's prices and wages by the month, for the price and wage maps
    months: Months,
    /// orders the simulation carried out, by kind
    carried: std::collections::BTreeMap<&'static str, usize>,
    refused: usize,
    /// what the thread last said of an order
    said: String,
    /// what became, at the last year's end, of the orders that waited for it
    turned: Vec<(bool, String)>,
}

impl Table {
    fn open(o: &Opening) -> Table {
        let (r, mut live, view) = start(o).expect("the game starts");
        // (`game::pump` takes the months the books kept on its first call)
        let months = live.months_kept.take().expect("the months the books kept, for the maps");
        Table { r, live, view, months, carried: Default::default(), refused: 0, said: String::new(), turned: Vec::new() }
    }

    fn wait(&self) -> Msg {
        loop {
            match self.live.try_recv() {
                Some(Msg::Unsaved(Some(why))) => panic!("the game could not be saved: {why}"),
                Some(Msg::Unsaved(None)) => {}
                Some(m) => return m,
                None => std::thread::sleep(std::time::Duration::from_millis(1)),
            }
        }
    }

    /// Ask for the next month, as `game::drive` does, and take in what comes
    /// as `game::pump` does. True when the year turned with it.
    fn month(&mut self) -> bool {
        self.live.step();
        loop {
            match self.wait() {
                Msg::Month(m) => {
                    self.months.add(&m);
                    apply_month(&mut self.r, &m);
                    crate::game::month_cash(&mut self.view, &m.cash);
                    self.live.month_came();
                    if self.live.months < 12 {
                        return false;
                    }
                }
                Msg::Year(end) => {
                    end.apply(&mut self.r);
                    crate::game::year_families(&mut self.view, &end.year.families, end.year.year as i32);
                    if let Some(rep) = &end.family {
                        crate::game::player_family(&mut self.view, rep);
                    }
                    self.view.prospects = end.prospects.clone();
                    self.live.year_came();
                    self.live.standing = end.standing.clone();
                    self.turned = end.carried.clone();
                    return true;
                }
                Msg::Failed(e) => panic!("the month failed: {e}"),
                _ => panic!("a month was asked for"),
            }
        }
    }

    /// Play on to the year's turn.
    fn step(&mut self) {
        while !self.month() {}
    }

    fn order(&mut self, kind: &'static str, c: Cmd) -> bool {
        self.live.send(c);
        match self.wait() {
            Msg::Done { ok, family, text, standing } => {
                self.said = text;
                self.live.standing = standing;
                if let Some(rep) = family {
                    crate::game::player_family(&mut self.view, &rep);
                }
                if ok {
                    *self.carried.entry(kind).or_default() += 1;
                } else {
                    self.refused += 1;
                }
                ok
            }
            _ => panic!("an order was given"),
        }
    }

    /// Take the last edit back, and forget what came after it as
    /// `back::taken_back` does; or the thread's refusal.
    fn take_back(&mut self) -> Result<TakenBack, String> {
        self.live.send(Cmd::TakeBack);
        match self.wait() {
            Msg::TakenBack(to) => {
                crate::back::forget(&mut self.r, &to);
                self.months.forget_from(to.t);
                crate::game::families_back(&mut self.view, &to.families, to.family.as_ref(), to.year as i32, to.filled);
                self.view.prospects = to.prospects.clone();
                self.live.taken_back(to.filled.clamp(1, self.r.ny) - 1, to.months, to.standing.clone());
                Ok(*to)
            }
            Msg::Done { ok: false, text, standing, .. } => {
                self.live.standing = standing;
                Err(text)
            }
            _ => panic!("a take back was asked for"),
        }
    }

    fn site(&self, cell: usize) -> SiteInfo {
        self.live.send(Cmd::Site(cell));
        match self.wait() {
            Msg::Site(s) => s,
            _ => panic!("a square was asked about"),
        }
    }

    /// What the simulation knows of a thing, asked as the sandbox's inspector asks it.
    fn ask(&self, thing: isostate_core::inspect::Thing) -> Result<isostate_core::inspect::Told, String> {
        self.live.send(Cmd::Ask(thing));
        match self.wait() {
            Msg::Told { thing: about, told } => {
                assert_eq!(about, thing, "the answer is about what was asked");
                told
            }
            _ => panic!("a thing was asked about"),
        }
    }

    /// The rules as they stand, asked as the sandbox's Rules tab asks.
    fn rules(&self) -> RulesNow {
        self.live.send(Cmd::Rules);
        match self.wait() {
            Msg::Rules(now) => *now,
            _ => panic!("the rules were asked for"),
        }
    }

    fn me(&self) -> FamilyRow {
        self.view.player().expect("there is a player").clone()
    }

    fn year(&self) -> i32 {
        self.r.y0 + self.r.filled as i32 - 1
    }

    /// A scripted player's orders for the year on the table: what is waiting
    /// is answered (not always with the default), land is bought and works
    /// built by the seat, people are posted, a branch is told to hold or grow,
    /// and once the seat is moved.
    fn play_year(&mut self) {
        let year = self.year();
        let me = self.me();
        for d in &me.rep.pending {
            let option = if year % 2 == 0 { d.default } else { d.options.len() - 1 };
            self.order("decide", Cmd::Decide { id: d.id, option });
        }
        let me = self.me();
        let home = self.r.header.towns[me.seat].node;
        let w = self.r.w;
        let around: Vec<usize> = [1, w, w + 1, 2, 2 * w, w + 2, 2 * w + 1].iter().flat_map(|&d| [home + d, home.wrapping_sub(d)]).filter(|&c| c < self.r.n).collect();
        if year % 3 == 1 {
            for &c in &around {
                if self.order("buy_land", Cmd::BuyLand(c)) {
                    break;
                }
            }
        }
        if year % 4 == 2 {
            'cells: for &c in &around {
                let site = self.site(c);
                for o in site.options.unwrap_or_default() {
                    if let (Ok(sums), Ok(())) = (&o.sums, &o.allowed) {
                        if sums.capital_cost < 0.5 * me.cash as f64 && self.order("found", Cmd::Found { method: o.method, cell: c }) {
                            break 'cells;
                        }
                    }
                }
            }
        }
        let k = self.r.filled - 1;
        let other = (0..self.r.n_towns).find(|&t| t != me.seat && self.r.town_active[k * self.r.n_towns + t] != 0);
        if year % 5 == 3 {
            let idle = me.rep.members.iter().find(|m| m.alive && m.age >= 18 && m.post == Post::Unposted);
            if let (Some(m), Some(town), Some(&group)) = (idle, other, me.rep.known.first()) {
                self.order("post", Cmd::Post { member: m.index, posting: Posting::Manager { group, town } });
            }
        }
        if year % 4 == 0 {
            if let Some(b) = me.rep.branches.first() {
                let stance = if b.stance == Stance::Grow { Stance::Hold } else { Stance::Grow };
                self.order("stance", Cmd::Stance { group: b.group, town: b.town, stance });
            }
        }
        if year == 1762 {
            if let Some(town) = other {
                self.order("move_seat", Cmd::MoveSeat(town));
            }
        }
        // one the simulation refuses: it must not reach the save
        let sea = (0..self.r.n).find(|&c| self.r.terrain[c] == 0).unwrap();
        assert!(!self.order("refused", Cmd::BuyLand(sea)));
    }
}

fn bits(v: &[f32]) -> Vec<u32> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// Both viewers hold months `ts` for the price and wage maps, with the same
/// prices and wages to the last bit.
fn same_months(a: &Table, b: &Table, ts: impl Iterator<Item = usize>, when: &str) {
    let (x, y) = (&a.months, &b.months);
    assert_eq!((x.nt, x.ng), (y.nt, y.ng), "{when}: towns and goods in a month");
    let (nt, ng) = (x.nt, x.ng);
    for t in ts {
        let (Some(i), Some(j)) = (x.row(t), y.row(t)) else { panic!("{when}: month {t} is not held by both ({} from {}, {} from {})", x.wage.len() / nt.max(1), x.first, y.wage.len() / nt.max(1), y.first) };
        assert_eq!(bits(&x.wage[i * nt..(i + 1) * nt]), bits(&y.wage[j * nt..(j + 1) * nt]), "{when}: the wages of month {t}");
        assert_eq!(bits(&x.price[i * nt * ng..(i + 1) * nt * ng]), bits(&y.price[j * nt * ng..(j + 1) * nt * ng]), "{when}: the prices of month {t}");
    }
}

/// Everything the viewer holds of two games is the same, to the last digit.
fn same(a: &Table, b: &Table, when: &str) {
    let (ra, rb) = (&a.r, &b.r);
    assert_eq!(ra.filled, rb.filled, "{when}: years played");
    for (name, x, y) in [
        ("land use", &ra.use_, &rb.use_),
        ("intensity", &ra.inten, &rb.inten),
        ("roads and rails", &ra.trans, &rb.trans),
        ("catchments", &ra.catch, &rb.catch),
        ("first land class", &ra.mix_a, &rb.mix_a),
        ("second land class", &ra.mix_b, &rb.mix_b),
        ("their shares", &ra.mix_s, &rb.mix_s),
        ("towns standing", &ra.town_active, &rb.town_active),
    ] {
        assert!(x == y, "{when}: {name} differs");
    }
    assert!(ra.owner == rb.owner, "{when}: who owns each square differs");
    // traffic (the core's docs/rules/traffic.md): what each way carried, and the trade between towns
    assert!(ra.traffic.goods == rb.traffic.goods && ra.traffic.mode == rb.traffic.mode, "{when}: what the ways carried differs");
    assert!(ra.traffic.pairs == rb.traffic.pairs, "{when}: the trade between towns differs");
    // series by series first, so that a failure says which, when and by how much
    assert_eq!((&ra.series_names, ra.series_len), (&rb.series_names, rb.series_len), "{when}: which series there are");
    let mut differ = Vec::new();
    for (i, name) in ra.series_names.iter().enumerate() {
        let months = i * ra.series_len..(i + 1) * ra.series_len;
        let (x, y) = (&ra.series[months.clone()], &rb.series[months]);
        let at: Vec<usize> = (0..x.len()).filter(|&t| x[t].to_bits() != y[t].to_bits()).collect();
        if let (Some(&first), Some(&last)) = (at.first(), at.last()) {
            let date = |t: usize| format!("{}-{:02}", ra.y0 + (t / 12) as i32, t % 12 + 1);
            differ.push(format!("{name}: {} months, {} to {}; in {} it is {} against {}", at.len(), date(first), date(last), date(first), x[first], y[first]));
        }
    }
    assert!(differ.is_empty(), "{when}: the island's series: {} of {} differ\n  {}", differ.len(), ra.series_names.len(), differ.join("\n  "));
    assert_eq!(bits(&ra.series), bits(&rb.series), "{when}: the island's series");
    assert_eq!(bits(&ra.town_pop), bits(&rb.town_pop), "{when}: town populations");
    assert_eq!(bits(&ra.town_urban), bits(&rb.town_urban), "{when}: townspeople");
    assert_eq!(bits(&ra.firm_k), bits(&rb.firm_k), "{when}: the works' capacities");
    let firms = |r: &Replay| r.header.firms.iter().map(|f| format!("{} {} {} {} {} {} {:?}", f.name, f.method, f.cell, f.town, f.born, f.died, f.owner)).collect::<Vec<_>>();
    assert_eq!(firms(ra), firms(rb), "{when}: the works");
    let towns = |r: &Replay| r.header.towns.iter().map(|t| format!("{} {} {}", t.name, t.node, t.founded)).collect::<Vec<_>>();
    assert_eq!(towns(ra), towns(rb), "{when}: the towns");
    let news = |r: &Replay| r.header.events.iter().map(|e| format!("{} {} {} {} {}", e.year, e.kind, e.weight, e.title, e.text)).collect::<Vec<_>>();
    assert_eq!(news(ra), news(rb), "{when}: the news");

    assert_eq!(a.view.year, b.view.year, "{when}");
    assert_eq!(a.view.player, b.view.player, "{when}");
    assert_eq!(a.view.families.len(), b.view.families.len(), "{when}");
    for (fa, fb) in a.view.families.iter().zip(&b.view.families) {
        // serde_json writes the shortest digits that read back to the same number
        assert_eq!(serde_json::to_string(&fa.rep).unwrap(), serde_json::to_string(&fb.rep).unwrap(), "{when}: the {} family's report", fa.name);
        assert_eq!(bits(&fa.history), bits(&fb.history), "{when}: the {} family's worth over the years", fa.name);
        assert_eq!(bits(&[fa.cash, fa.net_worth, fa.income]), bits(&[fb.cash, fb.net_worth, fb.income]), "{when}");
        assert_eq!((fa.works, fa.land, fa.seat), (fb.works, fb.land, fb.seat), "{when}");
    }
    assert_eq!(format!("{:?}", a.view.prospects), format!("{:?}", b.view.prospects), "{when}: what each trade would earn");
}

#[test]
fn a_game_saved_by_the_thread_comes_back_the_same() {
    let dir = saves::test_dir("live-test");
    let file = dir.join("game.json");
    let name = file.to_string_lossy().into_owned();

    // thirty years (1750 to 1779), with the demo's orders after the first and a script's after every one
    let mut a = Table::open(&Opening { seed: 7, families: 6, demo: true, fresh: true, shots: true, save: Some(name.clone()), ..default() });
    assert_eq!(a.year(), 1750);
    let demo = saves::read(&file).unwrap().orders.len();
    assert!(demo >= 2, "the demo's works and land are in the save from the start");
    assert_eq!(a.live.ordered.len(), 1, "the demo's works is on order");
    a.play_year();
    for _ in 0..29 {
        a.step();
        a.play_year();
    }
    assert_eq!(a.year(), 1779);
    eprintln!("carried out through the thread: {:?}; refused: {}", a.carried, a.refused);
    for kind in ["decide", "buy_land", "found", "post", "stance", "move_seat"] {
        assert!(a.carried.get(kind).copied().unwrap_or(0) > 0, "the script gave no {kind} order that was carried out: {:?}", a.carried);
    }

    // the thread kept the save up to the last order: every order carried out, and no other
    let saved = saves::read(&file).unwrap();
    let scripted: usize = a.carried.values().sum();
    assert_eq!(saved.orders.len(), demo + scripted, "the demo's orders and the script's");
    assert_eq!(saved.year, 1779);
    assert_eq!((saved.seed, saved.families), (7, 6));
    assert!(saves::read(&saves::earlier(&file, 1)).unwrap().year <= 1779);
    assert!(saves::earlier(&file, saves::KEPT).exists() && !saves::earlier(&file, saves::KEPT + 1).exists());

    // loaded as --load loads it; nothing is written, so the first game's file stays as it is
    let mut b = Table::open(&Opening { load: Some(name.clone()), shots: true, ..default() });
    assert_eq!(b.live.resumed, Some(Resumed { year: 1779, month: 0, orders: saved.orders.len(), refused: 0, drifted: false, passed_over: 0, by_hand: 0 }));
    assert!(a.live.kept && !b.live.kept, "the first is saved where it was told to be; the second only takes pictures");
    same(&a, &b, "after loading");
    // the last year's months for the maps, as the books kept them
    assert_eq!(b.months.first, 29 * 12);
    same_months(&a, &b, 29 * 12..30 * 12, "after loading");
    let on_order: Vec<usize> = saved.orders.iter().filter(|g| g.year == 1779).filter_map(|g| if let Order::Found { cell, .. } = &g.order { Some(*cell) } else { None }).collect();
    assert_eq!(b.live.ordered, on_order, "works ordered in the last year keep their pegs");

    // and they go on the same, through more orders and three more years
    a.carried.clear();
    for t in [&mut a, &mut b] {
        t.play_year();
        for _ in 0..3 {
            t.step();
            t.play_year();
        }
    }
    same(&a, &b, "three years on");
    same_months(&a, &b, 30 * 12..33 * 12, "three years on");
    assert!(!b.carried.is_empty(), "orders were given after loading");
    assert_eq!(a.carried, b.carried, "the same orders were carried out in both");
    assert_eq!(saves::read(&file).unwrap().year, 1782);

    // leaving the game waits for the thread, so the file is whole
    drop(a);
    drop(b);
    assert!(!dir.join("game.writing.json").exists());

    // years played on before the window opens: the card says the year the game opens in
    let d = Table::open(&Opening { load: Some(name), advance: 2, shots: true, ..default() });
    assert_eq!((d.year(), d.live.resumed.as_ref().unwrap().year), (1784, 1784));
    drop(d);

    // a place named to save in that holds something else is refused before
    // anything is played, and the something is left as it was
    let notes = dir.join("notes.txt");
    std::fs::write(&notes, "not a game").unwrap();
    let e = start(&Opening { seed: 7, families: 6, fresh: true, save: Some(notes.to_string_lossy().into_owned()), ..default() }).err().unwrap();
    assert!(e.contains("not a saved game"), "{e}");
    assert_eq!(std::fs::read_to_string(&notes).unwrap(), "not a game");
    assert!(start(&Opening { load: Some(String::new()), ..default() }).err().unwrap().contains("--load needs"));

    // a save the rules no longer agree with: one order names a method that is
    // gone, and one that changed the island is missing. It loads, to the same
    // year, and the player is owed the count and the warning.
    let mut odd = saved.clone();
    let bought = odd.orders.iter().position(|g| matches!(g.order, Order::BuyLand { .. })).unwrap();
    odd.orders.remove(bought);
    odd.orders.insert(0, isostate_core::save::Given { year: 1750, month: 0, order: Order::Found { method: "a_works_of_the_future".into(), cell: 100 } });
    let odd_file = dir.join("odd.json");
    std::fs::write(&odd_file, odd.to_text()).unwrap();
    let c = Table::open(&Opening { load: Some(odd_file.to_string_lossy().into_owned()), shots: true, ..default() });
    assert_eq!(c.year(), 1779);
    let back = c.live.resumed.clone().unwrap();
    assert!(back.refused >= 1 && back.drifted, "{back:?}");
    let note = crate::game::resumed_note(&back).unwrap();
    assert!(note.contains("could not be given again"), "{note}");
    assert!(note.ends_with(" The save as it was is kept."), "{note}");
    drop(c);
    // gone on with and kept in its own place: the file as it was, with the order that could not be given
    // again in it, is set aside whole before the game without that order is saved over it
    let name = odd_file.to_string_lossy().into_owned();
    drop(Table::open(&Opening { load: Some(name.clone()), save: Some(name), ..default() }));
    assert_eq!(std::fs::read_to_string(saves::previous(&odd_file)).unwrap(), odd.to_text(), "the save with every order in it is gone");
    let kept = saves::read(&odd_file).unwrap();
    assert!(kept.orders.len() < odd.orders.len() && kept.started == odd.started, "{} orders for {}", kept.orders.len(), odd.orders.len());
    let _ = std::fs::remove_dir_all(&dir);
}

/// The cheapest square by the seat that the family could buy.
fn a_square_to_buy(t: &Table) -> usize {
    let me = t.me();
    let k = t.r.filled - 1;
    // the squares that trade in the seat's market and are not the family's, a few dozen of them
    (0..t.r.n)
        .filter(|&c| t.r.catch[k * t.r.n + c] as usize == me.seat + 1 && t.r.terrain[c] > 0 && t.r.owner[k * t.r.n + c] != PLAYER as i8)
        .take(60)
        .map(|c| t.site(c))
        .filter(|s| s.owner != Some(PLAYER) && s.land_allowed.is_ok() && s.land_price < 0.5 * me.cash as f64)
        .min_by(|a, b| a.land_price.total_cmp(&b.land_price))
        .expect("a square the family could buy")
        .cell
}

#[test]
fn a_game_left_in_the_middle_of_a_year_is_gone_on_with_there() {
    let dir = saves::test_dir("midyear-test");
    let file = dir.join("game.json");
    let name = file.to_string_lossy().into_owned();

    // four years, the first trade chosen at the first turn; then into the fifth, to the end of May
    let mut a = Table::open(&Opening { seed: 7, families: 6, fresh: true, shots: true, save: Some(name.clone()), ..default() });
    a.play_year();
    for _ in 0..3 {
        a.step();
        a.play_year();
    }
    assert_eq!((a.year(), a.live.months, a.live.at_turn()), (1753, 0, true));
    let before = saves::read(&file).unwrap();
    for m in 1..=5 {
        assert!(!a.month());
        assert_eq!((a.live.months, a.live.at_turn()), (m, false));
    }
    assert_eq!(a.year(), 1753, "the year in progress is not a year played");
    // the months came with their own figures: the price of grain is not the same every month of 1754
    let grain = replay::monthly(&a.r, "price_grain", 4 * 12, 12);
    assert!(grain[..5].iter().all(|v| v.is_finite()) && grain[5..].iter().all(|v| v.is_nan()), "{grain:?}");
    assert!(grain[..5].windows(2).any(|w| w[0] != w[1]), "{grain:?}");

    // orders given in June: land and a move of the seat. They wait, and the thread says until when
    let square = a_square_to_buy(&a);
    let me = a.me();
    let other = (0..a.r.n_towns).find(|&t| t != me.seat && a.r.town_active[3 * a.r.n_towns + t] != 0).unwrap();
    assert!(a.order("buy_land", Cmd::BuyLand(square)));
    assert!(a.said.contains("at the year's turn, in January 1755"), "{}", a.said);
    assert!(a.order("stance", Cmd::Stance { group: me.rep.branches[0].group, town: me.rep.branches[0].town, stance: Stance::Hold }));
    assert!(a.said.contains("from the end of 1754"), "{}", a.said);
    assert!(a.order("move_seat", Cmd::MoveSeat(other)));
    assert!(a.order("move_seat", Cmd::MoveSeat(me.seat)), "the move is thought better of");
    assert_eq!(a.live.standing.waiting.len(), 4);
    // the last of them is taken back; the square is not the family's yet, and its price is promised
    assert!(a.order("cancel", Cmd::Cancel(3)));
    assert!(!a.order("cancel", Cmd::Cancel(3)));
    let stands = a.live.standing.clone();
    assert_eq!(stands.waiting.len(), 3);
    assert!(stands.promised > 0.0 && stands.pegs.is_empty());
    let me = a.me();
    assert!(!me.rep.cells.contains(&square), "what the family owns is what it owns now");
    assert_eq!(me.rep.next_seat, Some(other), "what it has been told shows as told");
    assert_eq!(me.rep.branches[0].stance, Stance::Hold);
    assert_eq!(crate::live::order_words(&stands.waiting[2], &a.r, Some(&me.rep)), format!("Move the seat to {}", a.r.header.towns[other].name));

    // the save was written with the last order: the month, and what waits
    let saved = saves::read(&file).unwrap();
    assert_eq!((saved.format, saved.year, saved.month, saved.waiting.len()), (iso::save::FORMAT, 1753, 5, 3));
    assert_eq!(saved.orders, before.orders, "nothing has been carried out since the turn");
    // two more months, and the game is left: the thread saves them on its way out
    assert!(!a.month() && !a.month());

    // gone on with: the same month, the same orders waiting, the same island
    let mut b = Table::open(&Opening { load: Some(name.clone()), shots: true, ..default() });
    // (the first game is still open: its months since the last order are not in the file yet)
    assert_eq!(b.live.resumed.as_ref().map(|r| (r.year, r.month, r.date(false))), Some((1753, 5, "June 1754".to_string())));
    assert_eq!((b.live.months, b.live.asked), (5, 5));
    assert_eq!(b.live.standing, stands);
    // the price and wage maps have the months the loaded game played before it was opened, 1753's and
    // 1754's (the books kept them), as the game that was watched has them; December as it ended in both
    assert_eq!(b.months.first, 3 * 12);
    same_months(&a, &b, 3 * 12..4 * 12 + 5, "loaded in June 1754");
    assert!(!b.month() && !b.month());
    same_months(&a, &b, 4 * 12..4 * 12 + 7, "seven months into 1754");
    // (what each trade would earn is worked out when a year arrives or a game is opened: for the one in January, for the other in June)
    b.view.prospects = a.view.prospects.clone();
    same(&a, &b, "seven months into 1754");

    // the year's end: the branch holds and the seat moves with the books; then the land is bought.
    // The seat has gone from beside the square, so the land is refused, and the family is told why
    for t in [&mut a, &mut b] {
        t.step();
        assert_eq!(t.year(), 1754);
        assert_eq!(t.turned.len(), 3, "{:?}", t.turned);
        assert!(t.turned[0].0 && t.turned[1].0 && !t.turned[2].0, "{:?}", t.turned);
        assert!(t.turned[2].1.contains("not done, because land is bought directly only around the seat"), "{:?}", t.turned);
        assert!(t.live.standing.waiting.is_empty() && t.live.standing.promised == 0.0);
        assert_eq!(t.me().seat, other);
    }
    same(&a, &b, "at the turn of 1755");
    // and the maps by the month, across the turn the orders waited for: 1753's and 1754's, December as it ended
    same_months(&a, &b, 3 * 12..5 * 12, "at the turn of 1755");
    drop(b);

    // left with the clock in the middle of a year and nothing ordered since the turn: the months are kept all the same
    assert!(!a.month() && !a.month());
    assert_eq!(saves::read(&file).unwrap().month, 0, "a month is not saved as it is played");
    // the clock stops: the interface asks for the months to be kept, and they are
    a.live.send(Cmd::Keep);
    let kept = saves::waited_for(120.0, || saves::read(&file).is_ok_and(|s| (s.year, s.month) == (1754, 2)));
    assert!(kept, "{}", saves::gave_up(120.0, "the months were to be saved when the clock stopped"));
    // and one more month, saved by the thread on its way out
    assert!(!a.month());
    drop(a);
    let left = saves::read(&file).unwrap();
    assert_eq!((left.year, left.month, left.waiting.len()), (1754, 3, 0));
    let c = Table::open(&Opening { load: Some(name), shots: true, ..default() });
    assert_eq!(c.live.resumed.as_ref().unwrap().date(false), "April 1755");
    // a game that is over has no January after its last year
    assert_eq!(Resumed { year: 1900, month: 0, orders: 0, refused: 0, drifted: false, passed_over: 0, by_hand: 0 }.date(true), "1900");
    assert!(!c.live.resumed.as_ref().unwrap().drifted);
    drop(c);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_edit_by_hand_goes_through_the_thread_and_comes_back_from_the_save() {
    // The sandbox (docs/designs/sandbox.md, section 6.1): an edit is sent to the
    // game's thread as the tray sends it, is answered with the line it wrote in
    // the island's news, and is kept in the save like any order. Played under
    // the game's rules with six families, nobody at the first.
    let dir = saves::test_dir("by-hand-test");
    let file = dir.join("game.json");
    let name = file.to_string_lossy().into_owned();
    let mut a = Table::open(&Opening { seed: 7, families: 6, fresh: true, shots: true, sandbox: true, save: Some(name.clone()), ..default() });
    a.step();
    a.step();
    for _ in 0..4 {
        assert!(!a.month());
    }
    // April 1753: the rest of the year's crop on the whole island is cut by three tenths
    assert!(a.order("by_hand", Cmd::ByHand(isostate_core::edits::Edit::Harvest { town: None, factor: 0.7 })), "{}", a.said);
    assert!(a.said.starts_with("By hand: the rest of 1753's harvest on the whole island"), "{}", a.said);
    // and the households' money of the first town doubled
    let town = (0..a.r.n_towns).find(|&t| a.r.town_active[(a.r.filled - 1) * a.r.n_towns + t] != 0).unwrap();
    assert!(a.order("by_hand", Cmd::ByHand(isostate_core::edits::Edit::Money { purse: isostate_core::edits::Purse::Households, town: Some(town), factor: 2.0 })));
    assert!(a.said.starts_with(&format!("By hand: households' money in {}", a.r.header.towns[town].name)), "{}", a.said);
    // a factor out of bounds is refused, and nothing is kept of it
    assert!(!a.order("by_hand", Cmd::ByHand(isostate_core::edits::Edit::People { town, factor: 50.0 })));
    // the edits are kept at once, dated in the month they were made
    let saved = saves::read(&file).unwrap();
    let edits: Vec<&isostate_core::save::Given> = saved.orders.iter().filter(|g| matches!(g.order, Order::ByHand { .. })).collect();
    assert_eq!(edits.len(), 2);
    assert!(edits.iter().all(|g| (g.year, g.month) == (1752, 4)), "{:?}", edits.iter().map(|g| (g.year, g.month)).collect::<Vec<_>>());
    // a year on, the save holds both and loads with no drift, the same island
    a.step();
    a.step();
    assert_eq!(a.year(), 1754);
    // (the thread writes the year's save after it has sent the year)
    let kept = saves::waited_for(120.0, || saves::read(&file).is_ok_and(|s| s.year == 1754));
    assert!(kept, "{}", saves::gave_up(120.0, "the year was to be saved"));
    let saved = saves::read(&file).unwrap();
    assert_eq!(saved.orders.iter().filter(|g| matches!(g.order, Order::ByHand { .. })).count(), 2);
    let b = Table::open(&Opening { load: Some(name), shots: true, ..default() });
    let back = b.live.resumed.as_ref().expect("the game is gone on with");
    assert!(!back.drifted && back.refused == 0, "{back:?}");
    same(&a, &b, "a year after two edits by hand");
    drop(a);
    drop(b);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_thread_tells_of_a_thing_what_a_game_played_to_the_same_month_tells() {
    // The sandbox's inspector (docs/designs/sandbox.md, 4.5 and 6.5): a
    // question goes to the game's thread between months and comes back as
    // `inspect::tell` makes it. On island 7 with six families, a sandbox
    // played to June 1760 tells of each of its towns what a game played to
    // June 1760 without a thread tells; a works is asked for by its slot in
    // the record's list; and an edit is seen in the next answer. Both games
    // are played under the game's rules as they stand, side by side, so the
    // case leans on no one island's history and sets no rule by name.
    use isostate_core::edits::Edit;
    use isostate_core::inspect::{Thing, Told};
    let mut a = Table::open(&Opening { seed: 7, families: 6, fresh: true, shots: true, sandbox: true, advance: 9, ..default() });
    assert_eq!(a.year(), 1759);
    for _ in 0..5 {
        assert!(!a.month());
    }
    let rules = isostate_core::params::Rules::Game;
    let mut b = isostate_core::save::Game::under_to(rules, 7, 6, 0, isostate_core::save::content_last_year(rules)).unwrap();
    // (a town's card dates its ideas by what is watched at each year's end, as the thread watches it)
    let mut learnt = isostate_core::inspect::Learnt::default();
    for _ in 0..10 {
        b.year().unwrap();
        learnt.year_end(&b.sim);
        b.turn();
    }
    for _ in 0..5 {
        assert!(!b.month().unwrap());
    }
    let k = a.r.filled - 1;
    let towns: Vec<usize> = (0..a.r.n_towns).filter(|&t| a.r.town_active[k * a.r.n_towns + t] != 0).collect();
    assert!(towns.len() > 5, "{} towns", towns.len());
    for &t in &towns {
        let told = a.ask(Thing::Town(t)).expect("a town that stands is told of");
        assert_eq!(told.as_of, "June 1760");
        assert_eq!(Ok(told), isostate_core::inspect::tell_with(&b.sim, Thing::Town(t), Some(&learnt)), "{}", a.r.header.towns[t].name);
    }
    // five works that stand, each asked for by its place in the record's list
    let mut works = 0;
    for f in 0..a.r.header.firms.len() {
        if let Ok(told) = a.ask(Thing::Works(f)) {
            assert_eq!(told.title, a.r.header.firms[f].name);
            works += 1;
            if works == 5 {
                break;
            }
        }
    }
    assert_eq!(works, 5);
    // the most populous town's people halved by hand: the next answer has them at half
    let row = |told: &Told, group: &str, name: &str| told.groups.iter().find(|g| g.name == group).and_then(|g| g.rows.iter().find(|r| r.0 == name)).map(|r| r.1.clone()).unwrap_or_else(|| panic!("no row {name} in {group}"));
    let figure = |s: String| s.replace(',', "").parse::<f64>().unwrap_or_else(|_| panic!("not a figure: {s}"));
    let town = *towns.iter().max_by_key(|&&t| a.r.town_pop[k * a.r.n_towns + t] as u64).unwrap();
    let before = a.ask(Thing::Town(town)).unwrap();
    assert_eq!(row(&a.ask(Thing::Island).unwrap(), "By hand", "edits carried out"), "none yet");
    assert!(a.order("by_hand", Cmd::ByHand(Edit::People { town, factor: 0.5 })), "{}", a.said);
    let after = a.ask(Thing::Town(town)).unwrap();
    let (was, now) = (figure(row(&before, "People", "people in its market area")), figure(row(&after, "People", "people in its market area")));
    assert!((now - 0.5 * was).abs() <= 1.0, "{was} halved is {now}");
    // (the purses are as they were: those who are left inherit)
    for purse in ["the households' money", "the landlords' money", "the merchants' money"] {
        assert_eq!(row(&before, "Money and prices", purse), row(&after, "Money and prices", purse));
    }
    assert_eq!(row(&a.ask(Thing::Island).unwrap(), "By hand", "edits carried out"), "1");
}

#[test]
fn the_thread_tells_the_rules_and_a_rule_set_by_hand_is_in_its_next_answer() {
    // The sandbox's Rules tab (docs/designs/sandbox.md, 4.6 and 6.6): the
    // rules are asked for over the channel and come back walked
    // (`isostate_core::rulebook`), with an entry for every pinned name; after
    // money.growth is set to nothing by hand, the next answer has it at
    // nothing, differing from the game's. On island 7 with six families under
    // the game's rules as they stand: it asks for no history beyond the rule
    // of money's issue being there to set (`money.growth`, whose game value
    // it reads from the game's rules rather than writing down).
    use isostate_core::edits::Edit;
    use isostate_core::rulebook::PINNED;
    let mut a = Table::open(&Opening { seed: 7, families: 6, fresh: true, shots: true, sandbox: true, advance: 3, ..default() });
    assert!(!a.month());
    let before = a.rules();
    for name in PINNED {
        assert!(before.entries.iter().any(|e| e.name == name), "{name} is pinned and not in the answer");
    }
    assert!(!before.unfound.is_empty());
    let growth = |now: &RulesNow| now.entries.iter().find(|e| e.name == "money.growth").cloned().unwrap();
    let game = isostate_core::params::game_params().money.growth;
    assert_eq!(growth(&before).value.as_f64(), Some(game));
    assert!(a.order("by_hand", Cmd::ByHand(Edit::Rule { name: "money.growth".into(), value: serde_json::json!(0) })), "{}", a.said);
    assert!(a.said.starts_with("By hand"), "{}", a.said);
    let after = growth(&a.rules());
    assert_eq!(after.value.as_f64(), Some(0.0));
    assert_eq!(after.game.as_f64(), Some(game));
    assert_ne!(after.value.as_f64(), after.game.as_f64(), "money.growth differs from the game's");
    // (and the walk offers it back: As set, the game's figure, is not refused)
    assert!(after.offers.iter().any(|o| o.word == "As set" && o.refused.is_none()));
    // a month on it still stands
    assert!(!a.month());
    assert_eq!(growth(&a.rules()).value.as_f64(), Some(0.0));
}

#[test]
fn a_works_by_hand_is_offered_with_the_square_waits_with_its_stake_and_is_taken_back() {
    // The sandbox's Works tool (docs/designs/sandbox.md, 4.4 and 6.7, stage 7b).
    // On island 7 in January 1791, the answer about a farm square of
    // the first town's district near it holds the hand's list
    // (`edits::works_offered`) with a method that may be built there; given
    // five months on, it is answered "Ordered by hand", stands among what
    // waits for the turn and pegs its square; `Cmd::Cancel(0)` takes it away.
    // Given again, it is carried out at the turn, told in the line it wrote in
    // the news, and its square stays pegged until the year's picture has it.
    // Played under the game's rules as they stand, six families, nobody at the
    // first: it asks of the island's history only that the island's first town
    // stands and that some square near it takes a works, and it looks for that
    // square rather than naming one. (It was written to Melbourne on the old
    // island; since the game's rules took the new land, docs/rules/land.md, "The
    // switch", island 7's towns have other names, and the first town is taken.)
    use isostate_core::edits::Edit;
    let mut a = Table::open(&Opening { seed: 7, families: 6, fresh: true, shots: true, sandbox: true, advance: 40, ..default() });
    assert_eq!(a.year(), 1790);
    let (k, w, n) = (a.r.filled - 1, a.r.w as i64, a.r.n);
    let town = 0;
    let name = a.r.header.towns[town].name.clone();
    let node = a.r.header.towns[town].node as i64;
    // the squares of its district round the town, the nearest first
    let mut near: Vec<usize> = (-5..=5i64)
        .flat_map(|dz| (-5..=5i64).map(move |dx| node + dz * w + dx))
        .filter(|&c| c >= 0 && (c as usize) < n && c != node)
        .map(|c| c as usize)
        .filter(|&c| a.r.catch[k * n + c] as usize == town + 1)
        .collect();
    near.sort_by_key(|&c| ((c as i64 % w - node % w).pow(2) + (c as i64 / w - node / w).pow(2), c));
    let (cell, method, cost) = near
        .iter()
        .find_map(|&c| {
            let hand = a.site(c).hand.ok()?;
            let best = hand.into_iter().filter(|o| o.allowed.is_ok()).filter_map(|o| o.sums.ok().map(|s| (o.name, s.capital_cost, s.expected_return))).max_by(|x, y| x.2.total_cmp(&y.2))?;
            Some((c, best.0, best.1))
        })
        .unwrap_or_else(|| panic!("a square near {name} takes a works"));
    assert!(cost > 0.0);
    // five months on, it is ordered, and waits with its stake
    for _ in 0..5 {
        assert!(!a.month());
    }
    let build = Edit::BuildWorks { method: method.clone(), cell };
    assert!(a.order("by_hand", Cmd::ByHand(build.clone())), "{}", a.said);
    assert!(a.said.starts_with("Ordered by hand: a works ("), "{}", a.said);
    assert!(a.said.contains(&format!("in {name}'s district")) && a.said.ends_with("at the year's turn, in January 1792."), "{}", a.said);
    assert_eq!(a.live.standing.waiting, vec![Order::ByHand { edit: build.clone() }]);
    assert!(a.live.standing.pegs.contains(&cell), "{:?}", a.live.standing.pegs);
    // the Hand tab's row: its words, with no "By hand" before them
    let row = crate::hand_tab::waiting_words(&a.live.standing.waiting[0], &a.r);
    assert!(row.starts_with("a works (") && row.contains(&format!("at {}, {}", cell % a.r.w, cell / a.r.w)), "{row}");
    // taken back: nothing waits, and the stake is gone
    assert!(a.order("cancel", Cmd::Cancel(0)), "{}", a.said);
    assert!(a.live.standing.waiting.is_empty() && !a.live.standing.pegs.contains(&cell), "{:?}", a.live.standing);
    // given again, it is carried out at the turn, in the line it wrote in the news
    assert!(a.order("by_hand", Cmd::ByHand(build)), "{}", a.said);
    a.step();
    assert_eq!(a.year(), 1791);
    let line = a.turned.iter().find(|t| t.1.starts_with("By hand")).unwrap_or_else(|| panic!("{:?}", a.turned));
    assert!(line.0 && line.1.contains("built for"), "{line:?}");
    assert!(a.live.standing.waiting.is_empty() && a.live.standing.pegs.contains(&cell), "{:?}", a.live.standing);
}

#[test]
fn a_towns_ideas_come_with_its_card_opening_it_changes_nothing_and_discover_is_the_idea_tools_edit() {
    // A town's ideas on its card (town_ideas.rs). On island 7 in January
    // 1791, played under the game's rules as they stand, six families, nobody
    // at the first: every town's card, asked three times as the inspector asks
    // it, has its ideas, dated by what the thread has watched, and none of it
    // is an edit (the island's card still says "none yet"). Then the edit the
    // card's Discover it here makes for an idea found nowhere in the first
    // town is the Idea tool's, is carried out at once in the line it writes in
    // the news, and the card has the idea known there, found by hand. It leans
    // on no one island's history beyond some idea of the tree being found
    // nowhere in 1791, and looks for one rather than naming it.
    use isostate_core::inspect::{Stands, Thing};
    let mut a = Table::open(&Opening { seed: 7, families: 6, fresh: true, shots: true, sandbox: true, advance: 40, ..default() });
    assert_eq!(a.year(), 1790);
    let towns: Vec<usize> = (0..a.r.n_towns.min(a.r.header.towns.len())).filter(|&t| a.r.town_active[(a.r.filled - 1) * a.r.n_towns + t] != 0).collect();
    for _ in 0..3 {
        for &t in &towns {
            let told = a.ask(Thing::Town(t)).unwrap();
            let ideas = told.ideas.as_ref().expect("a town's card has its ideas");
            assert_eq!((ideas.town, ideas.from, ideas.to), (t, 1750, 1791));
            // every idea known here is dated, and only one found nowhere may be found here by hand
            for i in ideas.all() {
                assert!(i.stands != Stands::Here || i.year.is_some(), "{}: {}", told.title, i.name);
                assert!(!i.discover || i.stands == Stands::Nowhere, "{}: {}", told.title, i.name);
            }
        }
    }
    let island = a.ask(Thing::Island).unwrap();
    let edits = island.groups.iter().find(|g| g.name == "By hand").and_then(|g| g.rows.iter().find(|r| r.0 == "edits carried out")).map(|r| r.1.clone());
    assert_eq!(edits.as_deref(), Some("none yet"), "opening the cards made no edit");
    // Discover it here, on the first town's first idea found nowhere
    let t = towns[0];
    let ideas = a.ask(Thing::Town(t)).unwrap().ideas.unwrap();
    let idea = ideas.all().find(|i| i.discover).expect("some idea of the tree is found nowhere in 1791").clone();
    let edit = crate::sandbox::discover_edit(idea.name.clone(), t);
    assert_eq!(edit, crate::sandbox::idea_tool_edit(&idea.name, t), "the card's edit is the Idea tool's");
    assert!(a.order("by_hand", Cmd::ByHand(edit)), "{}", a.said);
    assert_eq!(a.said, format!("By hand: {} found in {} in 1791", idea.label, a.r.header.towns[t].name));
    let now = a.ask(Thing::Town(t)).unwrap().ideas.unwrap();
    let found = now.all().find(|i| i.name == idea.name).unwrap();
    assert_eq!((found.stands, found.year, found.discover), (Stands::Here, Some(1791), false));
    assert!(found.how.ends_with("by hand"), "{}", found.how);
    // and the same idea is now known elsewhere, not found nowhere, on every other town's card
    for &o in towns.iter().filter(|&&o| o != t) {
        let there = a.ask(Thing::Town(o)).unwrap().ideas.unwrap();
        assert_ne!(there.all().find(|i| i.name == idea.name).unwrap().stands, Stands::Nowhere);
    }
}

#[test]
fn a_game_begun_to_1910_plays_past_1900_through_the_thread_and_its_save_says_so() {
    // A game past 1900 (docs/rules/longrun.md, "A game past 1900"): begun with
    // a last year of 1910, played to 1899 before it opens and on through the
    // thread, as the interface plays it, into 1901; its save says it is played
    // to 1910, and loads again to the same island and the same last year; and
    // the thread says no more when 1910 is played. Under the game's rules as
    // they stand, three families: it leans on no one island's history.
    let dir = saves::test_dir("to-1910-test");
    let file = dir.join("game.json");
    let name = file.to_string_lossy().into_owned();
    let mut a = Table::open(&Opening { seed: 7, families: 3, fresh: true, shots: true, advance: 149, to: Some(1910), save: Some(name.clone()), ..default() });
    assert_eq!((a.year(), a.r.ny, a.r.y0 + a.r.ny as i32 - 1), (1899, 161, 1910));
    a.step();
    a.step();
    assert_eq!(a.year(), 1901);
    // (the thread writes the year's save after it has sent the year)
    let kept = saves::waited_for(120.0, || saves::read(&file).is_ok_and(|s| s.year == 1901));
    assert!(kept, "{}", saves::gave_up(120.0, "the year was to be saved"));
    let saved = saves::read(&file).unwrap();
    assert_eq!((saved.year, saved.to, saved.last_year_of_game()), (1901, Some(1910), 1910));
    let b = Table::open(&Opening { load: Some(name), shots: true, ..default() });
    let back = b.live.resumed.as_ref().expect("the game is gone on with");
    assert!(!back.drifted && back.refused == 0, "{back:?}");
    assert_eq!((b.year(), b.r.ny), (1901, 161));
    same(&a, &b, "a game to 1910, two years past 1900");
    drop(b);
    // played to its last year, and no further
    while a.year() < 1910 {
        a.step();
    }
    a.live.step();
    match a.wait() {
        Msg::Failed(why) => assert_eq!(why, "1910 was the last year."),
        _ => panic!("a month past the game's last year was played"),
    }
    drop(a);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_game_that_is_played_holds_the_books_figures_of_its_years() {
    // The books' series (the shares of income, ownership in years of wages)
    // have one figure for a year, known when the year's books close: after
    // January to November have gone to the interface. The year brings it, and
    // a game played through the thread holds it in all twelve months of every
    // year played, as the simulation does; of the year in progress it holds
    // nothing yet. Three years and three months, with no orders given.
    let mut a = Table::open(&Opening { seed: 7, families: 6, fresh: true, shots: true, ..default() });
    a.step();
    a.step();
    for _ in 0..3 {
        assert!(!a.month());
    }
    assert_eq!((a.year(), a.live.months), (1752, 3));
    // the same island played by the simulation alone
    let mut g = isostate_core::save::Game::new(7, 6, 0);
    for _ in 0..3 {
        g.year().unwrap();
        g.turn();
    }
    for _ in 0..3 {
        g.month().unwrap();
    }
    let st = &g.sim.st;
    // (the reckoning of pay, counted among the books' own, has no figure before 1900: docs/rules/pay.md)
    let own: Vec<&String> = a.r.series_names.iter().filter(|n| isostate_core::accounts::own_series(n) && !n.starts_with("pay_")).collect();
    // the books' eight, and the gauges' read off them (docs/rules/gauges.md), which come the same way,
    // and the reckoning of the people's ages (docs/rules/ages.md)
    assert_eq!(own.len(), 8 + isostate_core::gauges::SERIES.len() + isostate_core::ages::N_SERIES);
    for name in own {
        let held = replay::monthly(&a.r, name, 0, 48);
        let there = st.series.get(name);
        for t in 0..36 {
            assert!(held[t].is_finite(), "{name} has no figure in month {t}");
            assert_eq!(held[t].to_bits(), (there[t] as f32).to_bits(), "{name} in month {t}");
        }
        assert!(held[36..].iter().all(|v| v.is_nan()) && there[36..48].iter().all(|v| v.is_nan()), "{name}: the year in progress has no figure yet");
        // (a gauge can read the same two years running, new work at nothing before anything is found,
        // and so can the reckoned years of school, two on an island too poor to buy more)
        assert!(held[..12].windows(2).all(|w| w[0] == w[1]) && (held[11] != held[12] || name.starts_with("gauge_") || name.starts_with("ages_")), "{name}: a year's figure, the same in its twelve months");
    }
    // which is the island's account of the year, and what a chart by the year shows
    let shares: Vec<f32> = st.acct.years.iter().map(|y| y.share_work as f32).collect();
    assert_eq!(shares.len(), 3);
    for (k, share) in shares.iter().enumerate() {
        assert_eq!(replay::month(&a.r, "acct_share_work", k * 12 + 4).map(f32::to_bits), Some(share.to_bits()));
        assert!((replay::yearly(&a.r, "acct_share_work")[k] - share).abs() < 1e-6);
    }
    // every other series has the months of the year in progress as they came
    let pop = replay::monthly(&a.r, "pop", 36, 12);
    assert!(pop[..3].iter().all(|v| v.is_finite()) && pop[3..].iter().all(|v| v.is_nan()), "{pop:?}");
}

#[test]
fn the_card_says_what_could_not_be_brought_back() {
    let back = |refused, drifted| Resumed { year: 1779, month: 0, orders: 41 - refused, refused, drifted, passed_over: 0, by_hand: 0 };
    assert_eq!(crate::game::resumed_note(&back(0, false)), None);
    assert_eq!(
        crate::game::resumed_note(&back(0, true)).unwrap(),
        "The island's rules have changed since this game was saved, so the years have played out a little differently."
    );
    assert_eq!(
        crate::game::resumed_note(&back(3, true)).unwrap(),
        "The island's rules have changed since this game was saved. Three of your 41 orders could not be given again, so some things stand differently. The save as it was is kept."
    );
    assert!(crate::game::resumed_note(&Resumed { year: 1751, month: 0, orders: 0, refused: 1, drifted: false, passed_over: 0, by_hand: 0 }).unwrap().contains(" Your one order could not"));
    assert!(crate::game::resumed_note(&Resumed { year: 1751, month: 0, orders: 0, refused: 4, drifted: true, passed_over: 0, by_hand: 0 }).unwrap().contains(" All four of your orders could not"));
    // an earlier save than the last, with or without the rules having changed
    assert_eq!(
        crate::game::resumed_note(&Resumed { passed_over: 1, ..back(0, false) }).unwrap(),
        "The last save could not be read, so this is an earlier one, from 1779."
    );
    // what New game leaves behind: a game with orders in it, set aside; one without, nothing
    assert_eq!(crate::game::new_game_hint(1750, true), "A new game begins again in 1750 and leaves this one saved beside it.");
    assert_eq!(crate::game::new_game_hint(1750, false), "A new game begins again in 1750. No order was given in this one, so it is not kept beside the new game: the same island makes it again.");
    let both = crate::game::resumed_note(&Resumed { passed_over: 2, ..back(0, true) }).unwrap();
    assert!(both.starts_with("The last save could not be read, so this is an earlier one, from 1779. The island's rules have changed"), "{both}");
    // a sandbox's short card: where the session was left and what was done by hand in it, and what Begin again leaves
    let session = |month, by_hand| Resumed { year: 1790, month, orders: by_hand, refused: 0, drifted: false, passed_over: 0, by_hand };
    assert_eq!(crate::game::session_left_words(&session(2, 2), false), "As you left it in March 1791, with two edits by hand.");
    assert_eq!(crate::game::session_left_words(&session(0, 1), false), "As you left it in January 1791, with one edit by hand.");
    assert_eq!(crate::game::session_left_words(&session(0, 0), false), "As you left it in January 1791, with nothing done by hand.");
    assert_eq!(crate::game::session_left_words(&session(5, 23), false), "As you left it in June 1791, with 23 edits by hand.");
    assert_eq!(crate::game::begin_again_hint(7, 1750, true), "Begin again makes island 7 anew in 1750 and keeps this session beside it.");
    assert_eq!(
        crate::game::begin_again_hint(7, 1750, false),
        "Begin again makes island 7 anew in 1750. Nothing was done by hand in this session, so it is not kept beside the new one: the same island makes it again."
    );
}

/// `ISOSTATE_SAVES` names one folder for the whole program, so the tests that
/// set it take turns: each holds this while it does.
static SAVES_FOLDER: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn saves_folder_held() -> std::sync::MutexGuard<'static, ()> {
    // (a test that failed while holding it has let it go all the same)
    SAVES_FOLDER.lock().unwrap_or_else(|held| held.into_inner())
}

#[test]
fn a_sandbox_is_kept_in_its_session_and_a_plain_start_goes_on_with_it() {
    // The sandbox's sessions folder (docs/designs/sandbox.md, 6.8, 8a). Island 7 with six families
    // under the game's rules as they stand; the case holds what is kept and gone on with, not what
    // the island does, so it leans on no one island's history and sets no rule by name.
    use isostate_core::edits::{Edit, Purse};
    use isostate_core::inspect::{Thing, Told};
    let _held = saves_folder_held();
    let dir = saves::test_dir("session-test");
    unsafe { std::env::set_var("ISOSTATE_SAVES", &dir) };
    let file = saves::session(7, 6);
    assert_eq!(file, dir.join("sandbox").join("island-7-families-6.json"));
    let sandbox = |fresh| Opening { seed: 7, families: 6, sandbox: true, fresh, ..default() };
    assert!(sandbox(false).plain() && !sandbox(true).plain());
    assert_eq!((sandbox(false).keeping(), sandbox(true).keeping()), (Keeping::Session, Keeping::Session));
    let edits_carried_out = |t: &Table| {
        let told: Told = t.ask(Thing::Island).expect("the island is told of");
        told.groups.iter().find(|g| g.name == "By hand").and_then(|g| g.rows.iter().find(|r| r.0 == "edits carried out")).map(|r| r.1.clone()).expect("the ledger's row")
    };
    let by_hand = |s: &isostate_core::save::Save| s.orders.iter().filter(|g| matches!(g.order, Order::ByHand { .. })).count();

    // a new session, kept in its session: its largest town's people halved, a year played, and let go
    let mut a = Table::open(&sandbox(true));
    assert!(a.live.resumed.is_none() && a.live.kept);
    let k = a.r.filled - 1;
    let town = (0..a.r.n_towns).filter(|&t| a.r.town_active[k * a.r.n_towns + t] != 0).max_by_key(|&t| a.r.town_pop[k * a.r.n_towns + t] as u64).unwrap();
    assert!(a.order("by_hand", Cmd::ByHand(Edit::People { town, factor: 0.5 })), "{}", a.said);
    a.step();
    let year = a.year();
    drop(a);
    let saved = saves::read(&file).unwrap();
    assert_eq!((saved.year, saved.month, by_hand(&saved)), (year as i64, 0, 1));
    // and nothing in the family's place, nor among the trials
    assert!(!saves::autosave(7, 6).exists() && !dir.join("trials").exists());

    // a plain sandbox start goes on with it, the edit in its log
    let mut b = Table::open(&sandbox(false));
    let back = b.live.resumed.clone().expect("the session is gone on with");
    assert_eq!((back.year, back.month, back.by_hand, back.refused, back.drifted), (year, 0, 1, 0, false));
    assert!(b.live.kept);
    assert_eq!(edits_carried_out(&b), "1");
    // stopped five months into the year with an edit made that month: kept, and gone on with in that month
    for _ in 0..5 {
        assert!(!b.month());
    }
    assert!(b.order("by_hand", Cmd::ByHand(Edit::Money { purse: Purse::Households, town: None, factor: 2.0 })), "{}", b.said);
    drop(b);
    let c = Table::open(&sandbox(false));
    let back = c.live.resumed.clone().expect("the session is gone on with");
    assert_eq!((back.year, back.month, back.by_hand), (year, 5, 2));
    assert_eq!(edits_carried_out(&c), "2");
    drop(c);

    // a --new sandbox start makes the island anew and is saved at once, so that it is the session a plain
    // start goes on with from now: the session that was there is set aside whole
    let mut d = Table::open(&sandbox(true));
    assert!(d.live.resumed.is_none());
    let aside = saves::read(&saves::previous(&file)).expect("the session before is set aside");
    assert_eq!((aside.year, aside.month, by_hand(&aside)), (year as i64, 5, 2));
    let now = saves::read(&file).unwrap();
    assert!(now.orders.is_empty() && (now.year, now.month) == (1750, 0), "{} orders, {} {}", now.orders.len(), now.year, now.month);
    // and kept as it is played
    d.month();
    drop(d);
    assert_eq!(saves::read(&file).unwrap().month, 1);
    assert_eq!(saves::read(&saves::previous(&file)).unwrap(), aside, "set aside once");
    // a plain start now goes on with the new session, which nobody did anything in
    let e = Table::open(&sandbox(false));
    assert_eq!(e.live.resumed.as_ref().map(|r| (r.month, r.by_hand)), Some((1, 0)));
    drop(e);

    // a sandbox nobody is at is kept nowhere, unless --keep asks for its session in a folder named for it
    assert_eq!(Opening { sandbox: true, unattended: true, ..default() }.keeping(), Keeping::Nowhere);
    assert_eq!(Opening { sandbox: true, shots: true, unattended: true, ..default() }.keeping(), Keeping::Nowhere);
    let check = Opening { sandbox: true, unattended: true, keep: true, ..default() };
    assert_eq!(check.keeping(), Keeping::Session);
    assert!(check.keep_refused(false).unwrap().starts_with("--keep with nobody at the game"));
    assert_eq!(check.keep_refused(true), None);
    // and a place that was named is the place
    assert_eq!(Opening { sandbox: true, save: Some("x.json".into()), ..default() }.keeping(), Keeping::Named("x.json".into()));

    unsafe { std::env::remove_var("ISOSTATE_SAVES") };
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn only_a_plain_start_goes_on_with_the_autosave() {
    // ISOSTATE_SAVES is this test's while it holds the folder: the other tests name their files
    let _held = saves_folder_held();
    let dir = saves::test_dir("autosave-test");
    unsafe { std::env::set_var("ISOSTATE_SAVES", &dir) };
    let file = saves::autosave(7, 3);
    assert_eq!(file, dir.join("island-7-families-3.json"));

    // a new game writes nothing until something has happened in it
    let mut a = Table::open(&Opening { seed: 7, families: 3, ..default() });
    assert!(a.live.resumed.is_none());
    assert!(!file.exists(), "nothing is saved before the first order or year");
    a.play_year();
    a.step();
    a.step();
    let orders = saves::read(&file).unwrap().orders.len();
    assert!(orders >= 1);
    drop(a);

    // a plain start goes on with it
    let b = Table::open(&Opening { seed: 7, families: 3, ..default() });
    assert_eq!(b.live.resumed, Some(Resumed { year: 1752, month: 0, orders, refused: 0, drifted: false, passed_over: 0, by_hand: 0 }));
    assert!(b.live.kept);
    assert_eq!(b.year(), 1752);
    drop(b);

    // screenshots, --new or --seed, --advance, --demo and --save each get a new game
    assert!(Opening { seed: 7, families: 3, ..default() }.plain());
    for o in [
        Opening { shots: true, ..default() },
        Opening { fresh: true, ..default() },
        Opening { advance: 1, ..default() },
        Opening { demo: true, ..default() },
        Opening { save: Some("x.json".into()), ..default() },
    ] {
        assert!(!o.plain(), "{o:?}");
    }
    // and a run that only takes pictures leaves the autosave alone
    let played = std::fs::read(&file).unwrap();
    let autosaves = |dir: &std::path::Path| {
        let mut v: Vec<String> = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
        v.sort();
        v
    };
    let beside_it = autosaves(&dir);
    assert_eq!(beside_it, ["island-7-families-3.1.json", "island-7-families-3.2.json", "island-7-families-3.json", "island-7-families-3.json.lock"]);
    assert_eq!(Opening { seed: 7, families: 3, shots: true, ..default() }.keeping(), Keeping::Nowhere);
    let mut t = Table::open(&Opening { seed: 7, families: 3, shots: true, ..default() });
    assert!(t.live.resumed.is_none() && !t.live.kept);
    t.play_year();
    t.step();
    drop(t);
    assert_eq!(std::fs::read(&file).unwrap(), played);

    // a script goes on with the autosave where ISOSTATE_SAVES names the folder, and writes nothing there
    let o = Opening { seed: 7, families: 3, unattended: true, ..default() };
    assert!(o.plain() && o.keeping() == Keeping::Nowhere);
    let mut t = Table::open(&o);
    assert_eq!(t.live.resumed.as_ref().map(|r| r.year), Some(1752));
    assert!(!t.live.kept);
    t.play_year();
    t.step();
    drop(t);
    assert_eq!(std::fs::read(&file).unwrap(), played);
    assert_eq!(autosaves(&dir), beside_it);

    // a start that is not plain play is a trial, kept apart: four of them one after
    // another, each played on and given orders, and the island's own game is as it was.
    // (Each used to be autosaved, and the fourth pushed the played game out of the copies kept.)
    let trial = saves::trial(7, 3);
    assert_eq!(trial, dir.join("trials").join("island-7-families-3.json"));
    assert_eq!(Opening { advance: 1, ..default() }.keeping(), Keeping::Trial("--advance"));
    assert_eq!(Opening { demo: true, ..default() }.keeping(), Keeping::Trial("--demo"));
    assert_eq!(Opening { advance: 1, demo: true, load: Some("x.json".into()), fresh: true, ..default() }.keeping(), Keeping::Trial("--advance"));
    assert_eq!(Opening { advance: 1, save: Some("x.json".into()), ..default() }.keeping(), Keeping::Named("x.json".into()));
    assert_eq!(Opening { advance: 1, unattended: true, ..default() }.keeping(), Keeping::Nowhere);
    assert_eq!(Opening { fresh: true, ..default() }.keeping(), Keeping::Autosave);
    // told to by name (--keep), any start is kept as the island's own; a named file comes first
    assert_eq!(Opening { advance: 1, unattended: true, shots: true, keep: true, ..default() }.keeping(), Keeping::Autosave);
    assert_eq!(Opening { keep: true, save: Some("x.json".into()), ..default() }.keeping(), Keeping::Named("x.json".into()));
    // but with nobody at the game, only in a folder that was named for it: never among the player's own
    let check = Opening { unattended: true, keep: true, ..default() };
    assert!(check.keep_refused(false).unwrap().starts_with("--keep with nobody at the game"));
    assert_eq!(check.keep_refused(true), None);
    assert!(Opening { shots: true, keep: true, ..default() }.keep_refused(false).is_some());
    assert_eq!(Opening { advance: 5, keep: true, ..default() }.keep_refused(false), None, "somebody is at it, and asked");
    assert_eq!(Opening { unattended: true, ..default() }.keep_refused(false), None);
    assert_eq!(Opening { unattended: true, keep: true, save: Some("x.json".into()), ..default() }.keep_refused(false), None);
    for n in 1..=4 {
        let mut t = Table::open(&Opening { seed: 7, families: 3, advance: n, demo: n == 4, ..default() });
        assert!(t.live.resumed.is_none() && t.live.kept, "a trial is kept, in a place of its own");
        assert_eq!(t.year(), 1750 + n as i32);
        t.play_year();
        t.step();
        drop(t);
        assert_eq!(saves::read(&trial).unwrap().year, 1751 + n as i64);
        assert_eq!(std::fs::read(&file).unwrap(), played, "trial {n} wrote where the played game is kept");
        assert_eq!(autosaves(&dir), [beside_it.clone(), vec!["trials".to_string()]].concat(), "trial {n} left something among the played games");
    }
    // each trial with orders in it set the one before aside, among the trials
    assert!(saves::read(&saves::previous(&trial)).is_ok_and(|g| saves::holds_orders(&g)));
    // a trial is gone on with by naming it, and so becomes the island's own game: the game
    // that was there is set aside whole
    let t = Table::open(&Opening { load: Some(trial.to_string_lossy().into_owned()), ..default() });
    assert_eq!(t.live.resumed.as_ref().map(|r| r.year), Some(1755));
    drop(t);
    assert_eq!(std::fs::read(&file).unwrap(), played, "nothing is written until something happens in the game");

    // a second copy of the game on the island: refused before anything is played, whatever
    // it was asked for, and the first copy's game is as it was. "Before anything is played" is
    // held against the starts just below, which make the island and play its first year: each
    // refusal (the quickest of three tries) takes less than a quarter of the quickest of them,
    // all timed in the same run so that a busy machine slows both alike. (It used to be held to a
    // second, which a busy machine could run out.)
    let mut first = Table::open(&Opening { seed: 7, families: 3, ..default() });
    let mut refusals = Vec::new();
    for o in [Opening { seed: 7, families: 3, ..default() }, Opening { seed: 7, families: 3, fresh: true, ..default() }, Opening { load: Some(file.to_string_lossy().into_owned()), ..default() }] {
        let mut quickest = f32::INFINITY;
        for _ in 0..3 {
            let t0 = std::time::Instant::now();
            let e = start(&o).err().expect("a second copy started");
            quickest = quickest.min(t0.elapsed().as_secs_f32());
            assert!(e.starts_with("another copy of the game is open and is keeping its game in ") && e.contains("island-7-families-3.json: play on in that window"), "{e}");
        }
        refusals.push(quickest);
    }
    assert_eq!(std::fs::read(&file).unwrap(), played);
    // what is not kept there is not in its way: pictures, a script, a trial, another island's game
    let mut starts = Vec::new();
    for o in [
        Opening { seed: 7, families: 3, shots: true, ..default() },
        Opening { seed: 7, families: 3, unattended: true, ..default() },
        Opening { seed: 7, families: 3, advance: 1, ..default() },
        Opening { seed: 7, families: 2, ..default() },
    ] {
        let t0 = std::time::Instant::now();
        let t = Table::open(&o);
        starts.push(t0.elapsed().as_secs_f32());
        drop(t);
    }
    let slowest = refusals.iter().copied().fold(0.0, f32::max);
    let quickest = starts.iter().copied().fold(f32::INFINITY, f32::min);
    assert!(slowest < quickest / 4.0, "a second copy was refused only after {:.0} ms, and a start that makes the island and plays its first year took {:.0} ms", slowest * 1e3, quickest * 1e3);
    // the first copy plays on and is saved as ever
    first.step();
    // (the thread hands the year over and then writes it)
    let written = saves::waited_for(120.0, || saves::read(&file).is_ok_and(|s| s.year == 1753));
    assert!(written, "{}", saves::gave_up(120.0, "the first copy's year was to be saved"));
    // a copy started by one that is closing (New game) waits for it to let go
    let closing = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(400));
        drop(first);
    });
    let next = Table::open(&Opening { seed: 7, families: 3, restarted: true, ..default() });
    closing.join().unwrap();
    assert_eq!(next.live.resumed.as_ref().map(|r| r.year), Some(1753));
    drop(next);
    // back to the game of 1752, for what follows
    std::fs::write(&file, &played).unwrap();

    // another number of families is another game, with its own file
    let c = Table::open(&Opening { seed: 7, families: 2, ..default() });
    assert!(c.live.resumed.is_none());
    drop(c);

    // a new game on the island sets the old one aside when it is first saved
    let mut d = Table::open(&Opening { seed: 7, families: 3, fresh: true, ..default() });
    assert_eq!(saves::read(&file).unwrap().year, 1752, "starting a new game does not touch the old one");
    d.step();
    drop(d);
    assert_eq!(saves::read(&file).unwrap().year, 1751);
    assert_eq!(saves::read(&saves::previous(&file)).unwrap().year, 1752);
    assert_eq!(saves::read(&saves::previous(&file)).unwrap().orders.len(), orders);

    // a latest save cut short: the one before it is gone on with, and the card is owed a word about it
    let mut e = Table::open(&Opening { seed: 7, families: 3, ..default() });
    assert_eq!(e.year(), 1751);
    e.step();
    drop(e);
    assert_eq!(saves::read(&file).unwrap().year, 1752);
    let whole = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, &whole[..whole.len() / 2]).unwrap();
    let f = Table::open(&Opening { seed: 7, families: 3, ..default() });
    let back = f.live.resumed.clone().unwrap();
    assert_eq!((back.year, back.passed_over, back.refused, back.drifted), (1751, 1, 0, false));
    assert_eq!(f.year(), 1751);
    assert!(crate::game::resumed_note(&back).unwrap().contains("could not be read"));
    assert_eq!(f.live.unread, None, "there was a save to go on with");
    drop(f);

    // it is gone on with from there and played two years, with orders; and its latest save is spoiled again
    let mut on = Table::open(&Opening { seed: 7, families: 3, ..default() });
    for _ in 0..2 {
        on.play_year();
        on.step();
    }
    drop(on);
    let first_half = whole[..whole.len() / 2].to_string();
    assert_eq!(std::fs::read_to_string(saves::unread(&file)).unwrap(), first_half);
    let whole = std::fs::read_to_string(&file).unwrap();
    std::fs::write(&file, &whole[..whole.len() / 2]).unwrap();
    // New game from the card that offers the year before: the spoiled latest is set aside as unread, and
    // the game that was offered whole as the game before, where the new game's years cannot push it out
    let earlier_one = saves::read(&saves::earlier(&file, 1)).unwrap();
    assert!(saves::holds_orders(&earlier_one) && earlier_one.year == 1752, "{earlier_one:?}");
    let mut g = Table::open(&Opening { seed: 7, families: 3, fresh: true, ..default() });
    assert_eq!(g.live.unread.as_deref(), Some("island-7-families-3.unread.json"), "a new game over a file that cannot be read says so");
    for _ in 0..5 {
        g.play_year();
        g.step();
    }
    drop(g);
    assert_eq!(saves::read(&saves::previous(&file)).unwrap(), earlier_one, "the played game is not beside the new one");
    assert_eq!(std::fs::read_to_string(saves::unread(&file)).unwrap(), whole[..whole.len() / 2]);

    // nothing left that can be read: a plain start is a new game, and its card is owed a word about the file
    for n in 0..=saves::KEPT {
        std::fs::write(if n == 0 { file.clone() } else { saves::earlier(&file, n) }, "{\"format\": 9}").unwrap();
    }
    let mut h = Table::open(&Opening { seed: 7, families: 3, ..default() });
    assert!(h.live.resumed.is_none());
    assert_eq!(h.live.unread.as_deref(), Some("island-7-families-3.unread.json"));
    assert!(crate::game::unread_note("island-7-families-3.unread.json").starts_with("The saved game that was here could not be read, so this is a new one. That file is not written over: it is set aside as island-7-families-3.unread.json "));
    h.play_year();
    h.step();
    drop(h);
    assert_eq!(std::fs::read_to_string(saves::unread(&file)).unwrap(), "{\"format\": 9}");
    assert_eq!(std::fs::read_to_string(saves::unread_before(&file, 2)).unwrap(), whole[..whole.len() / 2]);
    assert_eq!(std::fs::read_to_string(saves::unread_before(&file, 3)).unwrap(), first_half);
    assert_eq!(saves::read(&saves::previous(&file)).unwrap(), earlier_one);
    // and a new game where nothing was, or where a game that can be read is, says nothing of the kind
    let i = Table::open(&Opening { seed: 7, families: 3, fresh: true, ..default() });
    assert_eq!(i.live.unread, None);
    drop(i);

    unsafe { std::env::remove_var("ISOSTATE_SAVES") };
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn a_month_leaves_what_scenery_is_built_from_as_it_was() {
    // (winter, task D) `rebuild_year` takes the years the run holds for the state of the run
    // that scenery is built from: a year built again from the same years is the camera's
    // alone, and only its houses go on screen again. That holds while a month writes
    // nothing a year's scenery reads, and a year's turn always adds a year
    let mut t = Table::open(&Opening { seed: 7, families: 6, fresh: true, shots: true, ..default() });
    let of = |r: &Replay| (r.filled, r.header.firms.len(), bits(&r.firm_k), bits(&r.town_pop), r.mix_a.clone(), r.mix_b.clone(), r.mix_s.clone());
    let before = of(&t.r);
    let mut months = 0;
    while !t.month() {
        months += 1;
        assert!(of(&t.r) == before, "month {months} changed the years, the works or the land the scenery is built from");
    }
    assert!(months >= 11, "a year of months went by ({months})");
    assert_eq!(t.r.filled, before.0 + 1, "a year's turn adds a year to the run");
}

#[test]
fn a_new_game_is_on_the_new_land_and_old_land_starts_one_on_the_old_island() {
    // Since the game's rules took the new land (docs/rules/land.md, "The switch"), a new game is
    // on it; `--old-land` starts one on the port's island. Either is a new game, kept where any
    // new game is: with somebody at it the island's own, begun some years in a trial.
    for o in [
        Opening { seed: 7, families: 2, fresh: true, old_land: true, ..default() },
        Opening { seed: 7, families: 2, fresh: true, old_land: true, advance: 3, ..default() },
    ] {
        assert!(!o.plain(), "a game on the old island is a new game: {o:?}");
        assert!(matches!(o.keeping(), Keeping::Autosave | Keeping::Trial(_)));
    }

    // A new game has the land in an `Arc` of its own, which a year arriving does not copy, and
    // the thread's save says which land it is on.
    let dir = saves::test_dir("new-land-test");
    let file = dir.join("game.json");
    let name = file.to_string_lossy().into_owned();
    let mut a = Table::open(&Opening { seed: 7, families: 2, fresh: true, shots: true, save: Some(name.clone()), ..default() });
    let land = a.r.land.clone().expect("a new game is on the new land");
    assert_eq!((land.w, land.h, land.fw, land.fh), (a.r.w, a.r.h, a.r.w * land.fine, a.r.h * land.fine));
    assert_eq!(land.height.len(), land.fw * land.fh);
    assert!(land.lines.len() > 100 && land.lakes.len() >= 2, "{} lines and {} lakes", land.lines.len(), land.lakes.len());
    assert!(land.lines.iter().all(|l| l.pts.len() >= 2 && l.pts.len() == l.width.len() && l.pts.len() == l.level.len() && l.pts.len() == l.flow.len()));
    a.play_year();
    a.step();
    let copy = a.r.clone();
    assert!(std::sync::Arc::ptr_eq(&land, a.r.land.as_ref().unwrap()) && std::sync::Arc::ptr_eq(&land, copy.land.as_ref().unwrap()), "a copy of the run copies a pointer to the land");
    assert!(a.live.kept);
    drop(a);
    let text = std::fs::read_to_string(&file).unwrap();
    assert!(text.contains("\"land\": true") && saves::read(&file).unwrap().land, "the save says the game is on the new land: {text}");

    // gone on with as --load goes on with it, it is on the same land
    let b = Table::open(&Opening { load: Some(name.clone()), save: Some(name), ..default() });
    assert!(b.live.kept);
    let again = b.r.land.clone().expect("a game gone on with from a save on the new land has its land");
    assert_eq!(bits(&again.height), bits(&land.height));
    assert_eq!(b.r.terrain, copy.terrain);
    drop(b);

    // and a game begun with --old-land has none, is another island, and its save says so
    let old = dir.join("old.json");
    let old_name = old.to_string_lossy().into_owned();
    let c = Table::open(&Opening { seed: 7, families: 2, fresh: true, shots: true, old_land: true, save: Some(old_name.clone()), ..default() });
    assert!(c.r.land.is_none());
    assert_ne!(c.r.terrain, copy.terrain, "the old island is another island");
    let old_terrain = c.r.terrain.clone();
    drop(c);
    let text = std::fs::read_to_string(&old).unwrap();
    assert!(text.contains("\"land\": false") && !saves::read(&old).unwrap().land, "the save says the game is on the old island: {text}");
    let d = Table::open(&Opening { load: Some(old_name.clone()), save: Some(old_name), ..default() });
    assert!(d.r.land.is_none() && d.r.terrain == old_terrain, "gone on with, it is on the old island again");
    drop(d);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_work_charts_hold_what_the_core_kept() {
    // The Markets panel's Work half (work_charts.rs) holds what the core kept, year by
    // year: island 7, six families, a game to 2050 begun as a sandbox is (`Game::under_to`
    // under the game's rules), its years taken in by a `Board` as a game gone on with from
    // a save takes them in, to the end of the year after the hoist is found. In every
    // year, each of the viewer's own rows and each series the charts read holds the core's
    // figure from the game's own state, to the bit in every month, and the charts' year
    // figures (an f32 mean of the months) to a millionth; hauling by machine is above
    // nothing in the last year. Played under the game's rules with the books, the gauges,
    // the owners and the machines for tasks (and their land) on, which it says by name; and
    // on the old island, which the case was written to: since the game's rules took the new
    // land (docs/rules/land.md, "The switch") island 7 is another island, on which nothing is
    // hauled by machine in the year after the hoist's find (1909).
    use crate::work_charts::{charts, MACHINE_ROWS, OWNED_ROWS};
    let t0 = std::time::Instant::now();
    let mut game = Game::on_old_land_to(iso::params::Rules::Game, 7, 6, 0, 2050).unwrap();
    let p = &game.sim.params;
    assert!(p.accounts.enabled && p.gauges.enabled && p.owners.enabled && p.tasks.enabled && p.tasks.land, "the case is played with the books, gauges, owners and tasks on");
    let hoist = game.sim.content.tasks.uses.iter().position(|u| u.name == "hoist").expect("the hoist is a machine for a task");
    let mut board = Board::new(&game.sim);
    let ratio = |a: f64, b: f64| if b.abs() > 1e-12 { a / b } else { f64::NAN };
    let mut until = None;
    let mut last_haul;
    loop {
        game.year().unwrap();
        board.year(&game.sim);
        game.turn();
        let (st, y0) = (&game.sim.st, game.sim.content.start_year);
        let y = game.last_year();
        let k = (y - y0) as usize;
        let r = &board.r;
        // every month of the year holds the core's figure, as an f32, to the bit
        let row = |name: &str, want: f64| {
            let i = r.series_names.iter().position(|s| s == name).unwrap_or_else(|| panic!("{y}: no row {name}"));
            for t in k * 12..(k + 1) * 12 {
                let x = r.series[i * r.series_len + t];
                assert_eq!(x.to_bits(), (want as f32).to_bits(), "{y}: {name} in month {t}: {x} against the core's {want}");
            }
        };
        let t = st.tasks.years.iter().find(|x| x.year == y).expect("the year's machines");
        let by_machine = |j: usize| ratio(t.held[j] + t.land_held.iter().map(|s| s[j]).sum::<f64>(), t.work[j] + t.land_work.iter().map(|s| s[j]).sum::<f64>());
        for (j, name) in MACHINE_ROWS.iter().enumerate() {
            row(name, by_machine(j));
        }
        last_haul = by_machine(0);
        let o = st.owners.years.iter().find(|x| x.year == y).expect("the year's owners");
        for (i, name) in OWNED_ROWS.iter().enumerate() {
            row(name, ratio(o.book[i], o.book.iter().sum()));
        }
        let a = st.acct.years.iter().find(|x| x.year == y).expect("the year's books");
        let g = st.gauges.years.iter().find(|x| x.year == y).expect("the year's gauges");
        let read = [
            ("acct_share_work", a.share_work),
            ("gauge_work_net", g.work_net),
            ("gauge_spend_work", g.spend_work),
            ("acct_share_works", a.share_works),
            ("acct_share_land", a.share_land),
            ("acct_share_merchants", a.share_merchants),
            ("gauge_fork_made", g.fork_made),
            ("gauge_fork_raw", g.fork_raw),
            ("gauge_fork_all", g.fork_all),
        ];
        for (name, v) in read {
            row(name, v);
        }
        // the chained and the linked wage are monthly series: each month as the core holds it
        for name in ["real_wage_chained", "real_wage_linked"] {
            let s = st.series.index[name];
            let i = r.series_names.iter().position(|n| n == name).unwrap();
            for t in k * 12..(k + 1) * 12 {
                assert_eq!(r.series[i * r.series_len + t].to_bits(), (st.series.data[s][t] as f32).to_bits(), "{y}: {name} in month {t}");
            }
        }
        // and the charts' figures for the year are the core's: what a wage buys is the linked
        // wage since 20 November 2026 (docs/rules/measures.md)
        let plans = charts(r, k);
        let s = st.series.index["real_wage_linked"];
        let wage = iso::observer::annual(&st.series.data[s], y0, y) / iso::observer::annual(&st.series.data[s], y0, y0);
        let near = |x: f32, want: f64, tol: f64, what: &str| assert!((x.is_nan() && want.is_nan()) || (x as f64 - want).abs() <= tol * want.abs().max(1.0), "{y}: {what}: the chart has {x}, the core {want}");
        near(plans[0].lines[0].vals[k], wage, 1e-5, "what a wage buys");
        for (p, names) in [(1, [0, 1, 2]), (3, [3, 4, 5]), (5, [6, 7, 8])] {
            for (l, &j) in names.iter().enumerate() {
                near(plans[p].lines[l].vals[k], read[j].1, 1e-6, read[j].0);
            }
        }
        for j in 0..4 {
            near(plans[2].lines[j].vals[k], by_machine(j), 1e-6, MACHINE_ROWS[j]);
        }
        if until.is_none() && st.tasks.found[hoist] >= 0 {
            until = Some(st.tasks.found[hoist] + 1);
            eprintln!("the hoist was found in {}", st.tasks.found[hoist]);
        }
        if until == Some(y) || game.over() {
            break;
        }
    }
    assert!(until.is_some(), "the hoist was never found");
    assert!(last_haul > 0.0, "hauling by machine in {}: {last_haul}", game.last_year());
    eprintln!("the work charts held the core's figures to the end of {}, hauling by machine {:.4} then; {:.1} s", game.last_year(), last_haul, t0.elapsed().as_secs_f32());
}

#[test]
fn made_here_draws_every_good_that_is_made_and_the_ground_is_held_year_by_year() {
    // The map views of where a good is made and of what the ground holds read
    // what the game's thread hands over each year (`holds_year`). Island 7 is
    // played eighty years under the game's rules, which keep the books and the
    // mines; it leans on no history, only on the books and the workings being kept.
    use crate::overlay::{self, Overlay};
    let mut game = isostate_core::save::Game::new(7, 6, 0);
    let p = &game.sim.params;
    assert!(p.accounts.enabled && p.mines.enabled, "the case is played with the books and the mines on");
    let mut board = Board::new(&game.sim);
    for _ in 0..80 {
        game.year().unwrap();
        board.year(&game.sim);
        game.turn();
    }
    let (r, st, ng) = (&board.r, &game.sim.st, game.sim.content.G);
    let k = (game.last_year() - game.sim.content.start_year) as usize;
    // what each town made is the books' own, to the bit, in every year played
    let h = &st.acct.hist;
    let kg = isostate_core::accounts::KG;
    let slots = h.goods.len() / (h.n * ng * kg);
    assert_eq!((h.n, r.holds.made_towns), (k + 1, slots));
    for j in [0, k / 2, k] {
        for t in 0..slots {
            for g in 0..ng {
                let want = h.goods[((j * slots + t) * ng + g) * kg];
                assert_eq!(r.holds.made(j, t, g, ng).map(f32::to_bits), Some(want.to_bits()), "year {j}, town {t}, good {g}");
            }
        }
    }
    // every good made that year draws a map, and only those say nobody made any
    let mut drawn = 0;
    for g in 0..ng {
        let made: f32 = (0..slots).filter_map(|t| r.holds.made(k, t, g, ng)).sum();
        let w = overlay::wash(r, k, Overlay::Made, g, None, overlay::Scaling { range: crate::ranges::Range::Year, ends: overlay::year_ends(r, k, Overlay::Made, g) });
        let lit = (0..r.n).filter(|&c| w.cells[c * 4..c * 4 + 3] != [0xd9, 0xd3, 0xc4] && w.cells[c * 4 + 3] != 0).count();
        if made > 0.0 {
            assert!(w.legend.range.is_some() && lit > 0, "{} was made ({made}) and drew no map: {}", r.header.goods[g].name, w.legend.note);
            drawn += 1;
        } else {
            assert!(w.legend.note.starts_with("Nobody made any"), "{}: {}", r.header.goods[g].name, w.legend.note);
        }
    }
    assert!(drawn >= 10, "only {drawn} goods were made");
    // the timber standing and what is left of the deposits, year by year
    assert_eq!(r.holds.cover.len(), r.ny * r.n);
    assert_eq!(&r.holds.cover[k * r.n..(k + 1) * r.n], &isostate_core::record::field_bytes(&st.cover)[..]);
    assert!(r.holds.left.get(k).is_some_and(|l| !l.is_empty()), "no deposit was worked in eighty years");
    let coal = overlay::ground_map(r, k, 0).unwrap();
    assert!(coal.share_left.is_some_and(|s| s > 0.0 && s < 1.0), "{:?}", coal.share_left);
    eprintln!("{drawn} goods made in {} drew their maps; {:.1}% of the coal is left", game.last_year(), coal.share_left.unwrap() * 100.0);
}

#[test]
fn a_games_traffic_and_trade_between_towns_are_its_recordings() {
    // The traffic record (the core's docs/rules/traffic.md): what a game's thread
    // hands over each year (`LiveYear::traffic` and `pairs`) is what the same game's
    // `run.bin` holds, the one as the bytes of the record and the other summed by the
    // viewer from the books' tables (`traffic::pairs_of_books`). Island 7 under the
    // game's rules, which keep the traffic and the books, ten years; it leans on no
    // history, only on both being kept.
    let mut game = isostate_core::save::Game::new(7, 6, 0);
    let p = &game.sim.params;
    assert!(p.traffic.enabled && p.accounts.enabled, "the case is played with the traffic and the books kept");
    let mut board = Board::new(&game.sim);
    for _ in 0..10 {
        game.year().unwrap();
        board.year(&game.sim);
        game.turn();
    }
    let s = &game.sim;
    let bytes = isostate_core::record::pack(&s.st, &s.content, &s.params, &s.events, &s.rec, "test", false);
    let rec = replay::parse("test.bin", &bytes).expect("the game's record reads");
    let (a, b) = (&board.r.traffic, &rec.traffic);
    let k = (game.last_year() - s.content.start_year) as usize;
    let (n, ny) = (board.r.n, board.r.ny);
    assert_eq!((a.goods.len(), b.goods.len(), a.mode.len()), (ny * n * 4, ny * n * 4, ny * n));
    assert!(a.goods[..(k + 1) * n * 4] == b.goods[..(k + 1) * n * 4] && a.mode[..(k + 1) * n] == b.mode[..(k + 1) * n], "the ways' traffic differs");
    let busy = (0..n * 4).filter(|&l| a.goods[k * n * 4 + l] > 0).count();
    assert!(busy > 1000, "only {busy} links carried anything in the tenth year");
    for j in 0..=k {
        assert!(!a.pairs[j].is_empty(), "year {j}: no trade between towns");
        // (the weight to a part in a hundred thousand: a recording's goods' weights are its
        // header's, written to five places, and the game's are the content's own)
        assert_eq!(a.pairs[j].len(), b.pairs[j].len(), "year {j}: pairs of towns that traded");
        for (x, y) in a.pairs[j].iter().zip(&b.pairs[j]) {
            assert_eq!((x.from, x.to, x.value.to_bits(), x.lead), (y.from, y.to, y.value.to_bits(), y.lead), "year {j}: the trade between towns differs");
            assert!((x.weight - y.weight).abs() <= 1e-5 * x.weight.abs().max(1.0), "year {j}: {x:?} against {y:?}");
        }
    }
    // a town's card names five partners, the largest first
    let nt = board.r.n_towns;
    let t = (0..nt).max_by(|&x, &y| board.r.town_urban[k * nt + x].total_cmp(&board.r.town_urban[k * nt + y])).unwrap();
    let ps = crate::traffic::partners(&board.r, k, t, crate::traffic::PARTNERS);
    assert_eq!(ps.len(), crate::traffic::PARTNERS);
    assert!(ps.windows(2).all(|w| w[0].worth() >= w[1].worth()));
    let lines = crate::traffic::trade_lines(&board.r, k, t).unwrap();
    eprintln!("{lines}");
    assert!(lines.starts_with("Trades most with") && lines.lines().count() == 1 + crate::traffic::PARTNERS, "{lines}");
}

/// A square near the first town whose hand's list has a works allowed, and
/// the method that promises most there (as the Works tool offers it).
fn a_square_for_a_works(a: &Table) -> (usize, String) {
    let (k, w, n) = (a.r.filled - 1, a.r.w as i64, a.r.n);
    let node = a.r.header.towns[0].node as i64;
    let mut near: Vec<usize> = (-5..=5i64)
        .flat_map(|dz| (-5..=5i64).map(move |dx| node + dz * w + dx))
        .filter(|&c| c >= 0 && (c as usize) < n && c != node)
        .map(|c| c as usize)
        .filter(|&c| a.r.catch[k * n + c] == 1)
        .collect();
    near.sort_by_key(|&c| ((c as i64 % w - node % w).pow(2) + (c as i64 / w - node / w).pow(2), c));
    near.iter()
        .find_map(|&c| {
            let hand = a.site(c).hand.ok()?;
            let best = hand.into_iter().filter(|o| o.allowed.is_ok()).filter_map(|o| o.sums.ok().map(|s| (o.name, s.expected_return))).max_by(|x, y| x.1.total_cmp(&y.1))?;
            Some((c, best.0))
        })
        .expect("a square near the first town takes a works")
}

/// What the island holds as its save says: its `save::check`, once the save
/// of the year and month asked for is written, played again.
fn check_of_save(file: &std::path::Path, year: i64, month: usize) -> String {
    let kept = saves::waited_for(120.0, || saves::read(file).is_ok_and(|s| s.year == year && s.month == month));
    assert!(kept, "{}", saves::gave_up(120.0, &format!("{month} months into {} was to be saved", year + 1)));
    let save = saves::read(file).unwrap();
    let loaded = isostate_core::save::load(&save, |_| {}).expect("the save plays again");
    assert!(!loaded.drifted && loaded.refused.is_empty(), "the save plays again as it was played");
    isostate_core::save::check(&loaded.game.sim)
}

#[test]
fn an_edit_taken_back_leaves_the_island_that_never_had_it() {
    // Taking back (docs/designs/sandbox.md, 4.4 and 6.8). Island 7, six families,
    // the game's rules as they stand, played to January 1791 through the thread.
    // Each case is set beside the same island played with no edit at all, so it
    // leans on no one island's history and sets no rule by name: the take back
    // must give that island back, in the viewer as on the thread and in its save.
    use isostate_core::edits::Edit;
    let dir = saves::test_dir("take-back-test");
    let game_at = |name: &str| {
        let file = dir.join(format!("{name}.json"));
        (Opening { seed: 7, families: 6, fresh: true, shots: true, sandbox: true, advance: 40, save: Some(file.to_string_lossy().into_owned()), ..default() }, file)
    };
    let (o, untouched) = game_at("untouched");
    let mut u = Table::open(&o);

    // 1. Halving the largest town's people at the turn into 1791, two years on, taken back.
    let (o, file) = game_at("at-the-turn");
    let mut a = Table::open(&o);
    assert_eq!(a.year(), 1790);
    // (no edit has been made since the window opened)
    assert_eq!(a.live.standing.back, None);
    assert_eq!(a.take_back().unwrap_err(), NOTHING_TO_TAKE_BACK);
    let (k, nt) = (a.r.filled - 1, a.r.n_towns);
    let town = (0..nt).max_by(|&x, &y| a.r.town_pop[k * nt + x].total_cmp(&a.r.town_pop[k * nt + y])).unwrap();
    // a refused edit makes no copy
    assert!(!a.order("by_hand", Cmd::ByHand(Edit::People { town, factor: 50.0 })));
    assert_eq!(a.live.standing.back, None);
    assert!(a.order("by_hand", Cmd::ByHand(Edit::People { town, factor: 0.5 })), "{}", a.said);
    assert_eq!(a.live.standing.back, Some((1790, 0)), "the edit can be taken back to the turn into 1791");
    a.step();
    a.step();
    assert_eq!((a.year(), a.live.standing.back), (1792, Some((1790, 0))));
    // (the thread writes the year's save after it has sent the year)
    let kept = saves::waited_for(120.0, || saves::read(&file).is_ok_and(|s| s.year == 1792));
    assert!(kept, "{}", saves::gave_up(120.0, "1792 was to be saved"));
    let with_edit = saves::read(&file).unwrap();
    assert!(with_edit.orders.iter().any(|g| matches!(g.order, Order::ByHand { .. })));
    let to = a.take_back().expect("the edit is taken back");
    assert_eq!((to.year, to.months, to.filled, to.t), (1790, 0, 41, 41 * 12));
    assert_eq!((a.year(), a.live.standing.back), (1790, None));
    same(&a, &u, "the edit at the turn into 1791 taken back two years on");
    assert_eq!(crate::back::differ(&a.r, &u.r), Vec::<&str>::new(), "the run taken back is the run never edited");
    // a second is refused, in the sentence the viewer is to show
    assert_eq!(a.take_back().unwrap_err(), NOTHING_TO_TAKE_BACK);
    // the save that held the edit is set aside whole, once, and the game's own no longer holds it
    let kept = saves::waited_for(120.0, || saves::read(&saves::previous(&file)).is_ok());
    assert!(kept, "{}", saves::gave_up(120.0, "the save with the edit was to be set aside"));
    assert_eq!(saves::read(&saves::previous(&file)).unwrap(), with_edit);
    assert!(saves::read(&file).unwrap().orders.iter().all(|g| !matches!(g.order, Order::ByHand { .. })));
    for _ in 0..10 {
        a.step();
        u.step();
    }
    assert_eq!(a.year(), 1800);
    same(&a, &u, "ten years after the take back");
    assert_eq!(crate::back::differ(&a.r, &u.r), Vec::<&str>::new(), "ten years after the take back");
    assert_eq!(check_of_save(&file, 1800, 0), check_of_save(&untouched, 1800, 0), "the save ten years after the take back");
    assert_eq!(saves::read(&saves::previous(&file)).unwrap(), with_edit, "set aside once, not again at every year");
    assert!(!saves::game_before(&file, 2).exists(), "nothing else was set aside");
    drop(a);
    // the island never edited, five months into 1801, for the cases that follow (kept as the clock
    // stopping keeps it)
    for _ in 0..5 {
        assert!(!u.month());
    }
    u.live.send(Cmd::Keep);

    // 2. The same edit five months into 1791; 3. a works by hand that waited for the turn.
    for case in ["five-months-in", "a-works-that-waited"] {
        let (o, file) = game_at(case);
        let mut a = Table::open(&o);
        let edit = if case == "five-months-in" {
            Edit::People { town, factor: 0.5 }
        } else {
            let (cell, method) = a_square_for_a_works(&a);
            Edit::BuildWorks { method, cell }
        };
        for _ in 0..5 {
            assert!(!a.month());
        }
        assert!(a.order("by_hand", Cmd::ByHand(edit.clone())), "{case}: {}", a.said);
        assert_eq!(a.live.standing.back, Some((1790, 5)), "{case}");
        if edit.at_turn() {
            assert_eq!(a.live.standing.waiting, vec![Order::ByHand { edit }], "{case}: the works waits for the turn");
        }
        a.step();
        a.step();
        assert_eq!(a.year(), 1792);
        let to = a.take_back().expect("the edit is taken back");
        assert_eq!((to.year, to.months, to.t), (1790, 5, 41 * 12 + 5), "{case}");
        assert!(a.live.standing.waiting.is_empty() && a.live.standing.back.is_none(), "{case}: {:?}", a.live.standing);
        for _ in 0..10 {
            a.step();
        }
        for _ in 0..5 {
            assert!(!a.month());
        }
        same(&a, &u, &format!("{case}: ten years after the take back"));
        assert_eq!(crate::back::differ(&a.r, &u.r), Vec::<&str>::new(), "{case}: ten years after the take back");
        same_months(&a, &u, 50 * 12..51 * 12 + 5, case);
        a.live.send(Cmd::Keep);
        assert_eq!(check_of_save(&file, 1800, 5), check_of_save(&untouched, 1800, 5), "{case}: the save ten years after the take back");
    }
    drop(u);
    let _ = std::fs::remove_dir_all(&dir);
}

// MAKING (town_flow.rs): a town's making comes from the game's thread as a
// card does, and is the plain game's; a year on, it is of the year after.
#[test]
fn a_towns_making_comes_from_the_thread_and_follows_the_year() {
    use isostate_core::inspect::Thing;
    let mut a = Table::open(&Opening { seed: 7, families: 6, fresh: true, shots: true, sandbox: true, advance: 9, ..default() });
    let rules = isostate_core::params::Rules::Game;
    let mut b = isostate_core::save::Game::under_to(rules, 7, 6, 0, isostate_core::save::content_last_year(rules)).unwrap();
    for _ in 0..10 {
        b.year().unwrap();
        b.turn();
    }
    let k = a.r.filled - 1;
    let towns: Vec<usize> = (0..a.r.n_towns).filter(|&t| a.r.town_active[k * a.r.n_towns + t] != 0).collect();
    for &t in &towns {
        let told = a.ask(Thing::Making(t)).expect("a town that stands has a making");
        assert_eq!(told.making.as_ref().map(|m| (m.town, m.year)), Some((t, 1759)), "{}", a.r.header.towns[t].name);
        assert_eq!(Ok(told), isostate_core::inspect::tell(&b.sim, Thing::Making(t)));
    }
    a.step();
    let m = a.ask(Thing::Making(towns[0])).unwrap().making.unwrap();
    assert_eq!(m.year, 1760);
}

// TOWN-FLOW-HAND (town_flow.rs): the hand on a town's making. On island 7 at
// the turn into 1790, under the game's rules as they stand, six families,
// nobody at the first: in the town whose making has most hands, the panel's
// menu of a kind with works standing and one offered builds another, closes
// one and turns a way of making it off; sent through the thread they are
// carried out in the lines they write in the news, and a game given the Works
// tool's own edits for the same (and the same way turned off, which no tool
// of the tray makes) holds everything the same to the last digit a year on.
// It looks for its kind rather than naming one.
#[test]
fn the_making_panels_edits_go_through_the_thread_and_give_the_trays_island() {
    use isostate_core::inspect::Thing;
    let open = || Table::open(&Opening { seed: 7, families: 6, fresh: true, shots: true, sandbox: true, advance: 40, ..default() });
    let (mut a, mut b) = (open(), open());
    assert_eq!(a.year(), 1790);
    let k = a.r.filled - 1;
    let towns: Vec<usize> = (0..a.r.n_towns).filter(|&t| a.r.town_active[k * a.r.n_towns + t] != 0).collect();
    let makings: Vec<isostate_core::making::Making> = towns.iter().map(|&t| a.ask(Thing::Making(t)).unwrap().making.unwrap()).collect();
    // the town with most hands that has a kind with works standing, one offered and a way in use
    let mut by_hands: Vec<&isostate_core::making::Making> = makings.iter().collect();
    by_hands.sort_by(|x, y| y.hands.total_cmp(&x.hands));
    let (m, kind) = by_hands
        .iter()
        .find_map(|m| m.hand.iter().find(|k| k.build.is_ok() && !k.standing.is_empty()).map(|k| (*m, k)))
        .expect("a kind with works standing and one offered");
    eprintln!("{}: {} works making {}, one more offered", m.name, kind.standing.len(), m.good_label[kind.good]);
    let menu = crate::town_flow::menu_for(m, kind.good, false, a.r.w).unwrap();
    let edit = |label: &str| menu.items.iter().find(|i| i.label == label).unwrap().edit.clone().unwrap();
    let (build, close) = (edit("Build another here"), edit("Close one"));
    let off = menu.ways.iter().find(|w| w.0.contains("in use")).and_then(|w| w.1.clone()).unwrap().edit.unwrap();
    let isostate_core::edits::Edit::BuildWorks { method, cell } = build.clone() else { panic!("{build:?}") };
    let isostate_core::edits::Edit::CloseWorks { works } = close.clone() else { panic!("{close:?}") };
    let (tray_build, tray_close) = crate::sandbox::works_tool_edits(cell, &method, works);
    assert_eq!((&build, &close), (&tray_build, &tray_close), "the panel's are the Works tool's edits");
    for (e, starts) in [(build, "built for"), (close, "closed"), (off.clone(), "turned off in")] {
        assert!(a.order("by_hand", Cmd::ByHand(e)), "{}", a.said);
        assert!(a.said.starts_with("By hand: ") && a.said.contains(starts), "{}", a.said);
    }
    for e in [tray_build, tray_close, off] {
        assert!(b.order("by_hand", Cmd::ByHand(e)), "{}", b.said);
    }
    // the panel asked again has the hand's lines on the kind
    let again = a.ask(Thing::Making(m.town)).unwrap().making.unwrap();
    let hand = again.hand_of(kind.good).unwrap();
    assert!(hand.by_hand.len() >= 3 && hand.methods.iter().any(|x| x.forbidden), "{:?}", hand.by_hand);
    a.step();
    b.step();
    same(&a, &b, "a year after the panel's edits and the tray's");
}

// GROUND-BRUSH (ground.rs): a stroke of the brush goes through the thread as one
// edit, the year after it holds the squares it changed (`Holds::now`), and taken
// back the island is the one never painted.
#[test]
fn a_stroke_of_the_brush_shows_in_the_years_ground_and_taken_back_leaves_the_untouched_island() {
    use isostate_core::edits::{Edit, Ground};
    // island 7, six families, the game's rules as they stand, played to January 1791
    // through the thread, set beside the same island with no edit: it leans on no
    // one island's history
    let o = Opening { seed: 7, families: 6, fresh: true, shots: true, sandbox: true, advance: 40, ..default() };
    let mut u = Table::open(&o);
    let mut a = Table::open(&o);
    let k = a.r.filled - 1;
    // two squares of land with no coal under them, side by side, the first the viewer finds
    let coal = a.r.holds.field("coal").expect("the ground's coal").to_vec();
    let w = a.r.w;
    let square = (w..a.r.n - w).find(|&c| (0..4).all(|d| a.r.terrain[c + d] > 0 && coal[c + d] == 0)).expect("land with no coal");
    let edit = Edit::Ground { kind: Ground::Coal, lay: true, size: 1, squares: vec![square + 1, square + 2] };
    // the preview the viewer paints is what the island then holds, to the fields' 256 steps
    let (preview, rich) = crate::overlay::preview_stroke(&a.r, k, 0, &Default::default(), true, 1, &[square + 1, square + 2], None).unwrap();
    assert!(a.order("by_hand", Cmd::ByHand(edit)), "{}", a.said);
    assert!(a.said.starts_with(&format!("By hand: coal laid down on {} squares", preview.len())), "the preview reaches the squares the stroke did: {} against {}", preview.len(), a.said);
    a.step();
    let k = a.r.filled - 1;
    let now = &a.r.holds.now[k];
    assert_eq!(now.len(), preview.len(), "the year holds every square the stroke changed");
    let g = crate::overlay::ground_map(&a.r, k, 0).unwrap();
    for &(c, v) in &preview {
        assert!((g.cells[c] - v).abs() <= 0.02 + 0.02 * rich as f32, "square {c}: the preview said {v}, the year holds {}", g.cells[c]);
    }
    assert!(g.by_hand && crate::overlay::ground_map(&u.r, u.r.filled - 1, 0).is_some_and(|g| !g.by_hand));
    // taken back, the island that was never painted
    a.take_back().expect("the stroke is taken back");
    same(&a, &u, "the stroke taken back");
    assert_eq!(crate::back::differ(&a.r, &u.r), Vec::<&str>::new(), "the run taken back is the run never painted");
    a.step();
    u.step();
    same(&a, &u, "a year after the stroke was taken back");
    assert!(a.r.holds.now.is_empty());
}
