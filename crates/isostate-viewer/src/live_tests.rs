//! The saved game end to end, without a window: a game is played through the
//! simulation thread as the interface plays it (orders over the channel, a
//! year at a time), the thread's own save is loaded as `--load` loads it, and
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
}

impl Table {
    fn open(o: &Opening) -> Table {
        let (r, live, view) = start(o).expect("the game starts");
        Table { r, live, view, carried: Default::default(), refused: 0 }
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

    fn step(&mut self) {
        self.live.send(Cmd::Step);
        match self.wait() {
            Msg::Year(y, prospects) => {
                apply_year(&mut self.r, &y);
                crate::game::year_families(&mut self.view, &y.families, y.year as i32);
                self.view.prospects = prospects;
            }
            Msg::Failed(e) => panic!("the year failed: {e}"),
            _ => panic!("a year was asked for"),
        }
    }

    fn order(&mut self, kind: &'static str, c: Cmd) -> bool {
        self.live.send(c);
        match self.wait() {
            Msg::Done { ok, family, .. } => {
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
    let dir = std::env::temp_dir().join(format!("isostate-live-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
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
    let mut b = Table::open(&Opening { load: Some(name), shots: true, ..default() });
    assert_eq!(b.live.resumed, Some(Resumed { year: 1779, orders: saved.orders.len(), refused: 0, drifted: false, passed_over: 0 }));
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

    // a save the rules no longer agree with: one order names a method that is
    // gone, and one that changed the island is missing. It loads, to the same
    // year, and the player is owed the count and the warning.
    let mut odd = saved.clone();
    let bought = odd.orders.iter().position(|g| matches!(g.order, Order::BuyLand { .. })).unwrap();
    odd.orders.remove(bought);
    odd.orders.insert(0, isostate_core::save::Given { year: 1750, order: Order::Found { method: "a_works_of_the_future".into(), cell: 100 } });
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

#[test]
fn the_card_says_what_could_not_be_brought_back() {
    let back = |refused, drifted| Resumed { year: 1779, orders: 41 - refused, refused, drifted, passed_over: 0 };
    assert_eq!(crate::game::resumed_note(&back(0, false)), None);
    assert_eq!(
        crate::game::resumed_note(&back(0, true)).unwrap(),
        "The island's rules have changed since this game was saved, so the years have played out a little differently."
    );
    assert_eq!(
        crate::game::resumed_note(&back(3, true)).unwrap(),
        "The island's rules have changed since this game was saved. Three of your 41 orders could not be given again, so some things stand differently."
    );
    assert!(crate::game::resumed_note(&Resumed { year: 1751, orders: 0, refused: 1, drifted: false, passed_over: 0 }).unwrap().contains(" Your one order could not"));
    assert!(crate::game::resumed_note(&Resumed { year: 1751, orders: 0, refused: 4, drifted: true, passed_over: 0 }).unwrap().contains(" All four of your orders could not"));
    // an earlier save than the last, with or without the rules having changed
    assert_eq!(
        crate::game::resumed_note(&Resumed { passed_over: 1, ..back(0, false) }).unwrap(),
        "The last save could not be read, so this is an earlier one, from 1779."
    );
    let both = crate::game::resumed_note(&Resumed { passed_over: 2, ..back(0, true) }).unwrap();
    assert!(both.starts_with("The last save could not be read, so this is an earlier one, from 1779. The island's rules have changed"), "{both}");
}

#[test]
fn only_a_plain_start_goes_on_with_the_autosave() {
    // ISOSTATE_SAVES is this test's alone: the other test names its file
    let dir = std::env::temp_dir().join(format!("isostate-autosave-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
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
    assert_eq!(b.live.resumed, Some(Resumed { year: 1752, orders, refused: 0, drifted: false, passed_over: 0 }));
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
    let mut t = Table::open(&Opening { seed: 7, families: 3, shots: true, ..default() });
    assert!(t.live.resumed.is_none() && !t.live.kept);
    t.play_year();
    t.step();
    drop(t);
    assert_eq!(saves::read(&file).unwrap().year, 1752);
    assert_eq!(saves::read(&file).unwrap().orders.len(), orders);

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
