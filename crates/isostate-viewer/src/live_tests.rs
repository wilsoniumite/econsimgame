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
        let (r, live, view) = start(o).expect("the game starts");
        Table { r, live, view, carried: Default::default(), refused: 0, said: String::new(), turned: Vec::new() }
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
                    apply_month(&mut self.r, &m);
                    crate::game::month_cash(&mut self.view, &m.cash);
                    self.live.month_came();
                    if self.live.months < 12 {
                        return false;
                    }
                }
                Msg::Year(end) => {
                    apply_year(&mut self.r, &end.year);
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

    fn site(&self, cell: usize) -> SiteInfo {
        self.live.send(Cmd::Site(cell));
        match self.wait() {
            Msg::Site(s) => s,
            _ => panic!("a square was asked about"),
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
    assert_eq!(b.live.resumed, Some(Resumed { year: 1779, month: 0, orders: saved.orders.len(), refused: 0, drifted: false, passed_over: 0 }));
    assert!(a.live.kept && !b.live.kept, "the first is saved where it was told to be; the second only takes pictures");
    same(&a, &b, "after loading");
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
    drop(c);
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
    assert_eq!((saved.format, saved.year, saved.month, saved.waiting.len()), (2, 1753, 5, 3));
    assert_eq!(saved.orders, before.orders, "nothing has been carried out since the turn");
    // two more months, and the game is left: the thread saves them on its way out
    assert!(!a.month() && !a.month());

    // gone on with: the same month, the same orders waiting, the same island
    let mut b = Table::open(&Opening { load: Some(name.clone()), shots: true, ..default() });
    // (the first game is still open: its months since the last order are not in the file yet)
    assert_eq!(b.live.resumed.as_ref().map(|r| (r.year, r.month, r.date(false))), Some((1753, 5, "June 1754".to_string())));
    assert_eq!((b.live.months, b.live.asked), (5, 5));
    assert_eq!(b.live.standing, stands);
    assert!(!b.month() && !b.month());
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
    drop(b);

    // left with the clock in the middle of a year and nothing ordered since the turn: the months are kept all the same
    assert!(!a.month() && !a.month());
    assert_eq!(saves::read(&file).unwrap().month, 0, "a month is not saved as it is played");
    // the clock stops: the interface asks for the months to be kept, and they are
    a.live.send(Cmd::Keep);
    let kept = (0..500).any(|_| {
        std::thread::sleep(std::time::Duration::from_millis(10));
        saves::read(&file).is_ok_and(|s| (s.year, s.month) == (1754, 2))
    });
    assert!(kept, "the months were not saved when the clock stopped");
    // and one more month, saved by the thread on its way out
    assert!(!a.month());
    drop(a);
    let left = saves::read(&file).unwrap();
    assert_eq!((left.year, left.month, left.waiting.len()), (1754, 3, 0));
    let c = Table::open(&Opening { load: Some(name), shots: true, ..default() });
    assert_eq!(c.live.resumed.as_ref().unwrap().date(false), "April 1755");
    // a game that is over has no January after its last year
    assert_eq!(Resumed { year: 1900, month: 0, orders: 0, refused: 0, drifted: false, passed_over: 0 }.date(true), "1900");
    assert!(!c.live.resumed.as_ref().unwrap().drifted);
    drop(c);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_card_says_what_could_not_be_brought_back() {
    let back = |refused, drifted| Resumed { year: 1779, month: 0, orders: 41 - refused, refused, drifted, passed_over: 0 };
    assert_eq!(crate::game::resumed_note(&back(0, false)), None);
    assert_eq!(
        crate::game::resumed_note(&back(0, true)).unwrap(),
        "The island's rules have changed since this game was saved, so the years have played out a little differently."
    );
    assert_eq!(
        crate::game::resumed_note(&back(3, true)).unwrap(),
        "The island's rules have changed since this game was saved. Three of your 41 orders could not be given again, so some things stand differently."
    );
    assert!(crate::game::resumed_note(&Resumed { year: 1751, month: 0, orders: 0, refused: 1, drifted: false, passed_over: 0 }).unwrap().contains(" Your one order could not"));
    assert!(crate::game::resumed_note(&Resumed { year: 1751, month: 0, orders: 0, refused: 4, drifted: true, passed_over: 0 }).unwrap().contains(" All four of your orders could not"));
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
}

#[test]
fn only_a_plain_start_goes_on_with_the_autosave() {
    // ISOSTATE_SAVES is this test's alone: the other test names its file
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
    assert_eq!(b.live.resumed, Some(Resumed { year: 1752, month: 0, orders, refused: 0, drifted: false, passed_over: 0 }));
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
    // it was asked for, and the first copy's game is as it was
    let mut first = Table::open(&Opening { seed: 7, families: 3, ..default() });
    for o in [Opening { seed: 7, families: 3, ..default() }, Opening { seed: 7, families: 3, fresh: true, ..default() }, Opening { load: Some(file.to_string_lossy().into_owned()), ..default() }] {
        let t0 = std::time::Instant::now();
        let e = start(&o).err().expect("a second copy started");
        assert!(e.starts_with("another copy of the game is open and is keeping its game in ") && e.contains("island-7-families-3.json: play on in that window"), "{e}");
        assert!(t0.elapsed().as_secs_f32() < 1.0, "the second copy was refused only after {:?}", t0.elapsed());
    }
    assert_eq!(std::fs::read(&file).unwrap(), played);
    // what is not kept there is not in its way: pictures, a script, a trial, another island's game
    for o in [
        Opening { seed: 7, families: 3, shots: true, ..default() },
        Opening { seed: 7, families: 3, unattended: true, ..default() },
        Opening { seed: 7, families: 3, advance: 1, ..default() },
        Opening { seed: 7, families: 2, ..default() },
    ] {
        drop(Table::open(&o));
    }
    // the first copy plays on and is saved as ever
    first.step();
    // (the thread hands the year over and then writes it)
    let written = (0..500).any(|_| {
        std::thread::sleep(std::time::Duration::from_millis(10));
        saves::read(&file).is_ok_and(|s| s.year == 1753)
    });
    assert!(written, "the first copy's year was not saved");
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
    drop(f);

    unsafe { std::env::remove_var("ISOSTATE_SAVES") };
    let _ = std::fs::remove_dir_all(&dir);
}
