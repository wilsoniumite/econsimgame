//! Taking an edit back: the viewer's half (docs/designs/sandbox.md, 4.4 and
//! 6.8). The game's thread keeps one copy of the game as it stood just before
//! the last edit by hand, and puts it back when asked (`Cmd::TakeBack`); the
//! viewer is then told where the island stands again (`Msg::TakenBack`), and
//! forgets everything it was sent after that moment.
//!
//! Forgetting is cutting, because everything a year or a month brings is
//! added on: the years filled, the works, towns and news in the header, the
//! months' figures, the families' histories, the works' ground. `forget` cuts
//! the run; `taken_back` cuts the rest (the scenery built or on its way for
//! years now gone, the clock, the panels' answers) and puts the clock at the
//! moment the copy stands at, stopped. The rows of the yearly tables past the
//! years kept are set back to what a new game holds there, so that the run is
//! the very run the viewer had then, and nothing that reads a whole row (a
//! works' life is read along all of its years) can find a forgotten year.
//!
//! A new yearly table is forgotten by `forget` only if it is named there: its
//! place is the list of tables in `cut_years`, with what a new game holds in a
//! row. A table the year writes into the run that is not named there keeps the
//! forgotten years' rows past `filled`, which nothing should read; the test in
//! this file compares every field of the run and would say so.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::game::Game;
use crate::live::{Live, TakenBack};
use crate::replay::Replay;
use crate::{Data, Picked, Timeline};

/// The process's private bytes (what it has committed of its own), where the
/// system can say: on Windows, `GetProcessMemoryInfo`. For measuring what the
/// copy the game's thread keeps costs (`ISOSTATE_TIMING`).
#[cfg(windows)]
pub fn private_bytes() -> Option<u64> {
    use std::ffi::c_void;
    // PROCESS_MEMORY_COUNTERS_EX
    #[repr(C)]
    #[derive(Default)]
    struct Counters {
        cb: u32,
        page_faults: u32,
        peak_working_set: usize,
        working_set: usize,
        peak_paged_pool: usize,
        paged_pool: usize,
        peak_non_paged_pool: usize,
        non_paged_pool: usize,
        pagefile: usize,
        peak_pagefile: usize,
        private: usize,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn K32GetProcessMemoryInfo(process: *mut c_void, counters: *mut Counters, cb: u32) -> i32;
    }
    let mut c = Counters { cb: std::mem::size_of::<Counters>() as u32, ..Default::default() };
    // SAFETY: the counters are the size they say they are, and the process is this one
    let ok = unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut c, c.cb) };
    (ok != 0).then_some(c.private as u64)
}

#[cfg(not(windows))]
pub fn private_bytes() -> Option<u64> {
    None
}

/// Cut the run back to what the viewer had been sent when the copy was
/// taken: the years filled, the works, the towns and the news, the months'
/// figures, and the marks the chronicle had set on the news.
pub fn forget(r: &mut Replay, to: &TakenBack) {
    let kept = to.filled.min(r.ny);
    r.filled = kept;
    cut_years(r, kept);

    // works: those sent after are gone, with their rows of plant; a works kept has no plant in a year
    // now forgotten, and one closed in such a year stands again
    let firms = to.firms.min(r.header.firms.len());
    let (ny, last) = (r.ny, r.y0 + kept as i32 - 1);
    r.header.firms.truncate(firms);
    r.firm_first.truncate(firms);
    r.firm_len.truncate(firms);
    r.firm_off.truncate(firms);
    r.firm_k.truncate(firms * ny);
    for f in 0..firms {
        r.firm_k[f * ny + kept..(f + 1) * ny].fill(0.0);
    }
    for firm in &mut r.header.firms {
        if firm.died > last {
            firm.died = -1;
        }
    }

    // the universities, which stood after the works (and were cut with them above), stand again as
    // they stood then
    crate::replay::stand_universities(r, &to.unis, kept);

    // towns: the slots are the run's, but the names are sent as each is founded
    r.header.towns.truncate(to.towns);

    // news: by its place in the simulation's list (the run's list is in order of date), with the
    // chronicle's mark as it was sent (a later year's ranking may have changed it)
    r.header.events.retain(|e| e.seq < to.news);
    for e in &mut r.header.events {
        if let Some(&kept) = to.kept.get(e.seq) {
            e.kept = kept;
        }
    }
    // every way over water, as the year before the copy sent them
    r.header.crossings = to.crossings.clone();

    // the months: every figure from the year in progress on is forgotten, and the months played of that
    // year are written in again as they were sent (the year's end wrote the books' figure over them)
    let from = (kept * 12).min(r.series_len);
    for row in r.series.chunks_mut(r.series_len.max(1)) {
        row[from..].fill(f32::NAN);
    }
    for (t, values) in &to.series {
        crate::live::apply_series(r, *t, values);
    }
    // a row of the viewer's own (work_charts.rs) that a forgotten year added goes again
    while r.series_names.last().is_some_and(|name| name.starts_with("viewer_")) && r.series[r.series.len() - r.series_len..].iter().all(|v| v.is_nan()) {
        r.series_names.pop();
        r.series.truncate(r.series_names.len() * r.series_len);
    }
}

/// Every table the year writes into the run by its place among the years,
/// with the rows from year `kept` on set back to what a new game holds there.
/// A table that a year makes only when it has something to put in it (the
/// workings, the trade between towns) is let go again when no year kept had
/// anything for it, as a new game would not have it yet.
fn cut_years(r: &mut Replay, kept: usize) {
    let n = r.n;
    let cut = |v: &mut Vec<u8>, per: usize, blank: u8| {
        if v.len() > kept * per {
            v[kept * per..].fill(blank);
        }
    };
    for v in [&mut r.use_, &mut r.inten, &mut r.trans, &mut r.catch, &mut r.rent, &mut r.dist, &mut r.mix_a, &mut r.mix_b] {
        cut(v, n, 0);
    }
    cut(&mut r.mix_s, n, 255);
    for v in [&mut r.built, &mut r.res, &mut r.works] {
        cut(v, n, 0);
    }
    let b = &mut r.builds;
    for v in [&mut b.dens, &mut b.value, &mut b.fill, &mut b.park] {
        cut(v, n, 0);
    }
    let know = b.know_towns * b.techs;
    cut(&mut b.know, know, 0);
    // (squares: the log of the years forgotten)
    b.squares.forget(kept);
    // (each year sends all of them; owner is -1 where no family owns a square)
    if r.owner.len() > kept * n {
        r.owner[kept * n..].fill(-1);
    }
    if r.rent_scale.len() > kept {
        r.rent_scale[kept..].fill(1.0);
    }
    let nt = r.n_towns;
    if r.town_active.len() > kept * nt {
        r.town_active[kept * nt..].fill(0);
        r.town_pop[kept * nt..].fill(0.0);
        r.town_urban[kept * nt..].fill(0.0);
        r.town_wage[kept * nt..].fill(0.0);
        let ng = r.header.goods.len();
        r.town_price[kept * nt * ng..].fill(0.0);
    }
    // what the ground holds and each town made
    let h = &mut r.holds;
    let made = h.made_towns * r.header.goods.len();
    if h.made.len() > kept * made {
        h.made[kept * made..].fill(0.0);
    }
    cut(&mut h.cover, n, 0);
    for row in h.left.iter_mut().skip(kept) {
        row.clear();
    }
    // (GROUND-BRUSH: what the hand laid down or took away, let go as the workings are)
    for row in h.now.iter_mut().skip(kept) {
        row.clear();
    }
    if h.now.iter().all(|w| w.is_empty()) {
        h.now.clear();
    }
    // the year's pits, quarries and mines
    for row in r.workings.iter_mut().skip(kept) {
        row.clear();
    }
    if r.workings.iter().all(|w| w.is_empty()) {
        r.workings.clear();
    }
    // traffic: what each link carried, and the trade between towns
    let t = &mut r.traffic;
    cut(&mut t.goods, n * 4, 0);
    cut(&mut t.mode, n, 0);
    for row in t.pairs.iter_mut().skip(kept) {
        row.clear();
    }
    if t.pairs.iter().all(|p| p.is_empty()) {
        t.pairs.clear();
    }
    // (courses: the ways' log of the years forgotten)
    t.ways.forget(kept);
}

/// The panels and the scenery that a take back makes forget.
#[derive(SystemParam)]
pub(crate) struct Forgetting<'w> {
    builds: Option<ResMut<'w, crate::year::Builds>>,
    shown: Option<ResMut<'w, crate::year::Shown>>,
    hand: Option<ResMut<'w, crate::handover::HandOver>>,
    overlay: Option<ResMut<'w, crate::overlay::MapOverlay>>,
    sandbox: Option<ResMut<'w, crate::sandbox::Sandbox>>,
    told: Option<ResMut<'w, crate::inspector::Answers>>,
    rules: Option<ResMut<'w, crate::rulebook::Known>>,
    rows: Option<ResMut<'w, crate::hand_tab::Pending>>,
}

/// The answer to a take back, which `game::pump` keeps for this to do the
/// rest: the run cut (`forget`), the works' ground of the works forgotten let
/// go, the scenery built or being built for years now gone dropped, the
/// clock put at the moment the copy stands at and stopped, and every answer
/// the panels hold let go, so that they are asked again.
#[allow(clippy::too_many_arguments)]
pub(crate) fn taken_back(
    mut game: ResMut<Game>,
    mut data: ResMut<Data>,
    mut live: ResMut<Live>,
    mut tl: ResMut<Timeline>,
    mut fam: ResMut<crate::family::FamilyView>,
    mut months: ResMut<crate::live::Months>,
    mut ui: ResMut<crate::ui::UiState>,
    mut picked: ResMut<Picked>,
    mut others: Forgetting,
) {
    let Some(to) = game.taken_back.take() else { return };
    let t0 = std::time::Instant::now();
    // a scenery build and a reckoning of the map's colours are of a run that is gone, and are let go
    // of. (One still running holds the run until it ends, and the run is then copied here, as a year
    // copies it when a build holds it: 20 to 30 ms off the main thread on island 7 past 1900.)
    if let Some(b) = others.builds.as_mut() {
        b.task = None;
    }
    if let Some(o) = others.overlay.as_mut() {
        o.forget(to.filled);
    }
    // the months that came before the answer and wait to go into the run: those before the copy go in
    let held = game.forget(&to.standing.pegs);
    let Data { r, g, slots, .. } = &mut *data;
    // (the run is the main thread's own unless a scenery build holds it, when it is copied, as a year is)
    let r = std::sync::Arc::make_mut(r);
    let owned = t0.elapsed().as_secs_f32() * 1e3;
    for m in held.iter().filter(|m| m.index < to.t) {
        crate::live::apply_month(r, m);
    }
    let gone = (r.filled.saturating_sub(to.filled), r.header.firms.len().saturating_sub(to.firms), r.header.towns.len().saturating_sub(to.towns), r.header.events.len().saturating_sub(to.news));
    // where the universities stood, after the works (they come back as the first of those)
    let unis_from = r.header.firms.len() - crate::replay::universities_standing(r);
    forget(r, &to);
    let cut = t0.elapsed().as_secs_f32() * 1e3;
    // the works' ground: the works forgotten give theirs up, and the universities standing after the
    // works keep theirs in their new places (the copy's universities are the first of those there were,
    // since a university is only ever added after the others)
    let slots = std::sync::Arc::make_mut(slots);
    let unis = crate::replay::universities_standing(r);
    // and the first year of each road, which the town's streets are laid by, is gone through again
    g.land.closes.forget_roads_from(to.filled);
    // and the roads the carts wore (`network::drawn_links`)
    g.worn.forget_from(to.filled);
    g.courses.forget_from(to.filled);
    // and where the pits were placed: a working opened after the year gone back to may open again
    // with the same name on another history (`scatter::pits`)
    slots.pits.forget();
    if unis == 0 {
        slots.forget(to.firms);
    } else if slots.firm_pos.len() >= unis_from + unis {
        slots.keep_after(unis_from, to.firms, unis);
    } else {
        // (ground never laid out for them: found now)
        slots.forget(to.firms);
        crate::scatter::place_new_firms(r, g, slots);
    }
    let latest = to.filled.clamp(1, r.ny) - 1;
    let r = &*r;

    months.forget_from(to.t);
    live.taken_back(latest, to.months, to.standing.clone());
    crate::game::families_back(&mut fam, &to.families, to.family.as_ref(), to.year as i32, to.filled);
    fam.prospects = to.prospects.clone();

    // the clock at the moment the copy stands at, stopped
    tl.t = crate::clock::at(latest, to.months.min(11));
    tl.playing = false;
    tl.built = None;
    // a works or a town that is forgotten is not looked at any more
    picked.what = match picked.what {
        Some(crate::Pick::Firm(f)) if f >= to.firms => None,
        Some(crate::Pick::Town(t)) if t >= to.towns => None,
        p => p,
    };

    // the scenery built or on its way for a year now gone (the build itself was let go of above, with
    // the colours of the map views' years)
    if let Some(s) = others.shown.as_mut() {
        s.forget();
    }
    if let Some(h) = others.hand.as_mut() {
        h.forget();
    }
    // the answers the panels hold, asked again
    if let Some(s) = others.sandbox.as_mut() {
        s.forget();
    }
    if let Some(t) = others.told.as_mut() {
        t.forget();
    }
    if let Some(k) = others.rules.as_mut() {
        k.forget();
    }
    if let Some(p) = others.rows.as_mut() {
        p.forget();
    }
    ui.dirty += 1;
    ui.months += 1;

    let date = crate::clock::month_year(crate::clock::date(r.y0, tl.t));
    let line = format!("Taken back: the island stands as it stood in {date}, before the last edit by hand.");
    game.note = Some((true, line.clone()));
    let ms = t0.elapsed().as_secs_f32() * 1e3;
    eprintln!(
        "{line} Forgot {} year{}, {} works, {} town{} and {} item{} of news; {ms:.0} ms on the main thread ({owned:.0} making the run its own, {:.0} cutting it, {:.0} the rest), {:.0} ms on the game's thread",
        gone.0,
        if gone.0 == 1 { "" } else { "s" },
        gone.1,
        gone.2,
        if gone.2 == 1 { "" } else { "s" },
        gone.3,
        if gone.3 == 1 { "" } else { "s" },
        cut - owned,
        ms - cut,
        to.took
    );
}

/// A fingerprint of anything that can be printed, without printing it whole.
#[cfg(test)]
fn print_sum(x: &impl std::fmt::Debug) -> u64 {
    use std::hash::Hasher;
    struct Sum(std::collections::hash_map::DefaultHasher);
    impl std::fmt::Write for Sum {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            self.0.write(s.as_bytes());
            Ok(())
        }
    }
    let mut s = Sum(Default::default());
    std::fmt::write(&mut s, format_args!("{x:?}")).unwrap();
    s.0.finish()
}

/// The fields of two runs that differ, by name: every field of `Replay` but
/// the island's own (its heights, its rivers, its land), which no year
/// changes. Figures are compared to the bit, so that a month forgotten
/// (NaN) is the same as a month never sent.
#[cfg(test)]
pub(crate) fn differ(a: &Replay, b: &Replay) -> Vec<&'static str> {
    let bits = |v: &[f32]| v.iter().map(|x| x.to_bits()).collect::<Vec<u32>>();
    let mut out = Vec::new();
    macro_rules! same {
        ($($f:ident),*) => { $( if a.$f != b.$f { out.push(stringify!($f)); } )* };
    }
    same!(w, h, n, ny, y0, n_towns, use_, inten, trans, catch, rent, dist, mix_a, mix_b, mix_s, town_active, firm_first, firm_len, firm_off, series_names, series_len, filled, owner, built, res, works);
    macro_rules! same_bits {
        ($($f:ident),*) => { $( if bits(&a.$f) != bits(&b.$f) { out.push(stringify!($f)); } )* };
    }
    same_bits!(town_pop, town_urban, town_wage, town_price, firm_k, series, rent_scale);
    macro_rules! same_print {
        ($($f:ident),*) => { $( if print_sum(&a.$f) != print_sum(&b.$f) { out.push(stringify!($f)); } )* };
    }
    same_print!(header, workings, holds, builds, traffic);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::Played;

    /// Forgetting three years played after a copy gives the copy back, field
    /// for field: from the turn into 1795, and from five months into it.
    /// Island 7 under the game's rules with six families, played as a game
    /// is gone on with (`Board`), to 2050.
    #[test]
    fn forgetting_three_years_gives_the_run_as_it_was() {
        for months in [0, 5] {
            let mut p = Played::to(7, 1795, months);
            let copy = p.r().clone();
            let to = p.taken_back();
            assert_eq!((to.year, to.months, to.filled, to.t), (1794, months, 45, 45 * 12 + months), "where the copy stands");
            p.play_years(3);
            assert_eq!(p.r().filled, copy.filled + 3, "three years were played");
            let moved = differ(p.r(), &copy);
            for field in ["filled", "header", "series", "firm_k", "use_", "town_pop"] {
                assert!(moved.contains(&field), "the years played changed {field}: {moved:?}");
            }
            forget(p.r_mut(), &to);
            assert_eq!(differ(p.r(), &copy), Vec::<&str>::new(), "{months} months into 1795: the run forgotten differs from the copy");
        }
    }

    /// A live game stands its universities as a recorded run does, from the
    /// year each is founded, after the simulation's own works, and forgetting
    /// years gives them back as they stood. Island 7 under the game's rules
    /// (the university, `state.university`, on), whose treasury founds the
    /// University of Glenwick at the end of 1905: played to the turn into
    /// 1907, two years more, and forgotten.
    #[test]
    fn a_live_game_stands_its_universities_and_forgetting_gives_them_back() {
        let mut p = Played::to_within(7, 1907, 0, 2050);
        assert!(isostate_core::university::on(&p.game.sim.params), "the game's rules found universities");
        let stood = |r: &Replay| (0..r.header.firms.len()).filter(|&f| crate::replay::is_university(r, f)).map(|f| (f, r.header.firms[f].name.clone(), r.header.firms[f].born)).collect::<Vec<_>>();
        let r = p.r();
        let unis = stood(r);
        assert_eq!(unis.len(), 1, "{unis:?}");
        let (f, name, born) = unis[0].clone();
        assert_eq!((name.as_str(), born), ("The University of Glenwick", 1905));
        // at the end of the works, standing from 1905's map to the last year played and not after
        assert_eq!((f, crate::replay::universities_standing(r)), (r.header.firms.len() - 1, 1));
        let y0 = r.y0;
        let k = move |year: i32| (year - y0) as usize;
        let size = |r: &Replay, year: i32| crate::replay::firm_k(r, f, k(year));
        assert!(size(r, 1904) == 0.0 && size(r, 1905) > 0.0 && size(r, 1906) > 0.0 && size(r, 1907) == 0.0, "{} {} {} {}", size(r, 1904), size(r, 1905), size(r, 1906), size(r, 1907));
        // the simulation's works before it are dense, one row of the run's years each, as the year writes them
        assert_eq!(r.firm_off[f - 1] as usize, (f - 1) * r.ny);
        let copy = r.clone();
        let to = p.taken_back();
        p.play_years(2);
        let r = p.r();
        let now = stood(r);
        assert_eq!(now.len(), 1);
        assert_eq!(now[0].0, r.header.firms.len() - 1, "the university stands after the works the two years brought");
        assert!(crate::replay::firm_k(r, now[0].0, k(1908)) > 0.0);
        forget(p.r_mut(), &to);
        assert_eq!(differ(p.r(), &copy), Vec::<&str>::new(), "the run forgotten to 1907 differs from the copy");
    }
}
