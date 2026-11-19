//! The hand-over: a year's scenery goes to the renderer a mesh at a time,
//! under a budget, so that the view never waits for it.
//!
//! A year is 320 to 450 MB of mesh in some eighteen pieces. Replaced in one
//! frame, the renderer packs and sends them all to the card before it draws
//! again, and the picture stands still for a third of a second
//! (docs/probes/dx-smooth.md). So `rebuild_year` no longer assigns a finished
//! build's meshes: it puts them in this queue, each with the handle it is for,
//! and `feed` hands over what fits a budget each frame. The old year's mesh of
//! each kind stays on screen until its turn comes, so for up to a second the
//! picture holds two years at once: the new year's houses among the old
//! year's trees.
//!
//! The budget is by the second and not by the frame (a run with no window
//! makes thirty frames a second and a window a hundred and twenty: both must
//! show a year in the same time), and it adapts: a frame long enough to be
//! seen as a stop cuts the rate, frames on time let it creep back up
//! (`Budget::frame`). No mesh goes in a frame that has a year's own work in
//! it (`HandOver::hold`).
//!
//! No build is queued while the queue holds anything (`rebuild_year`), and the
//! year counts as built when the queue has emptied: a year is shown whole or
//! not at all, and when years come faster than they can be shown some are
//! skipped and none is left half sent. A year that arrives while the last is
//! going out waits for it to finish; it is not dropped, since at the fastest
//! speed every year would be dropped by the next and none would ever arrive.
//!
//! Where the run is there to take stills (`--view`, `--shots`), the whole
//! queue goes in the frame it was filled in, as it always did: nobody is
//! watching, and the pictures cannot differ. `ISOSTATE_HANDOVER` is for
//! measuring and for looking: `whole` does the same in a game (the freeze as
//! it was, to set beside the cure), `strict` keeps the rate's rule as it was
//! first written, and `step:SECONDS` hands over one mesh every so many
//! seconds, slowly enough to take pictures of a year half arrived.
//!
//! A mesh in pieces (19 November 2026). A mesh goes to the card whole: the
//! renderer packs and sends every byte of it in the frame it is handed over,
//! 0.4 to 0.7 ms a megabyte on this machine. The trees' meshes are 40 to 63 MB
//! each, so the frame one went in took 26 to 43 ms of the renderer's work
//! whatever the rate said, and on the new land, whose meshes are a little
//! larger, 27 to 58 such frames in fourteen seconds of play ran over 33 ms. So
//! in a game the trees, the houses and the works are cut, off the main thread,
//! into pieces of about `PIECE` bytes, each a run of whole things
//! (`year::cut`), and each piece is a mesh of its own under a handle of its
//! own (`How::Banked`). No more than `FRAME` bytes go in one frame: the budget
//! of time, about six and a half milliseconds of the renderer's. The pieces of
//! a mesh go one after another while the old year's mesh stays on screen
//! whole; when the last piece has gone, the mesh's
//! entities take the new pieces all in the next frame (`feed`), and the old
//! ones are let go of. So a kind of scenery changes from one year to the next
//! in one frame, as it did, and never shows half of each. What lies flat and
//! the smoke are a few megabytes and go whole, as before (`How::InPlace`):
//! where a lane meets a road the two lie at one height, and which is drawn
//! over the other hangs on the order their meshes were made in.
//!
//! A mesh that is the same as the one on screen, to the bit, is not sent
//! again (`HandOver::sums`). (The trees grow every year, so in practice that
//! is only ever a mesh with nothing in it.)
//!
//! Pieces at no more than 13 MB a frame take longer to go than whole meshes
//! did with no window, at thirty frames a second (a year in about one and a
//! half seconds, where it was half a second), so the next year's build may
//! begin while a year is still going out (`year::rebuild_year`), and the
//! island keeps up with the clock.

use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;

use crate::Timeline;

/// About how large a piece of a mesh is, in bytes on the card: a piece ends
/// at the first whole thing past this.
pub const PIECE: usize = 6_000_000;
/// The most that goes in one frame, in bytes on the card: the budget of time,
/// about six and a half milliseconds of the renderer's packing at the 0.5 ms a
/// megabyte measured with the pieces (nine at the 0.67 measured for whole
/// meshes on a busy machine), so that with a frame's own five or so it keeps
/// to sixty frames a second: in practice two pieces. A piece larger than this on
/// its own (a single thing of more than `PIECE` bytes) still goes, alone.
pub const FRAME: usize = 13_000_000;

/// Where the rate begins, in bytes a second: a mesh of 47 MB then goes every
/// ninth frame at 120 Hz and a year is on screen in three quarters of a second.
const RATE: f32 = 600e6;
/// The rate is held between these.
const RATE_FLOOR: f32 = 150e6;
const RATE_CEILING: f32 = 1200e6;
/// A frame that ran this long over its usual length is a stop the eye sees,
/// and cuts the rate: the freeze line of the frame recorder, in seconds.
const STOP: f32 = 0.034;
/// The rule as it was first written (`Pace::Strict`): a frame this many times
/// its usual length cuts the rate.
const LATE: f32 = 1.5;
/// What a frame that was a stop multiplies the rate by, and what a 120th of a
/// second of frames on time does.
const CUT: f32 = 0.7;
const CREEP: f32 = 1.02;
/// A mesh goes whatever the allowance says when none has gone for this long, in seconds.
const HURRY: f32 = 0.1;
/// How many frames the usual length of a frame is read from.
const RECENT: usize = 31;

/// How the queue is let out.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pace {
    /// under the budget: the game
    Budget,
    /// under the budget with the rate's rule as it was first written, where any frame half
    /// as long again as usual cuts the rate: kept to set beside the game's (see `Budget::frame`)
    Strict,
    /// everything in the frame it was queued in: stills, and the freeze as it was
    Whole,
    /// one mesh every so many seconds: for pictures of a year half arrived
    Step(f32),
}

impl Pace {
    /// `ISOSTATE_HANDOVER`: `whole`, `strict`, `step:SECONDS`, or nothing for the budget.
    pub fn from_env() -> Pace {
        match std::env::var("ISOSTATE_HANDOVER").ok().as_deref() {
            Some("whole") => Pace::Whole,
            Some("strict") => Pace::Strict,
            Some(s) if s.starts_with("step:") => match s[5..].parse::<f32>() {
                Ok(secs) if secs > 0.0 => Pace::Step(secs),
                _ => {
                    eprintln!("ISOSTATE_HANDOVER: {s} should be step:SECONDS");
                    std::process::exit(2);
                }
            },
            Some("") | None => Pace::Budget,
            Some(s) => {
                eprintln!("ISOSTATE_HANDOVER: {s} is not whole, strict or step:SECONDS");
                std::process::exit(2);
            }
        }
    }
}

/// The allowance, apart from the meshes it is spent on, so that it can be
/// tested with sizes alone.
#[derive(Debug)]
pub struct Budget {
    pub pace: Pace,
    /// bytes that may go now
    allowance: f64,
    /// bytes a second
    pub rate: f32,
    /// seconds since anything went
    idle: f32,
    /// the last frames' lengths, in seconds
    recent: VecDeque<f32>,
}

impl Budget {
    pub fn new(pace: Pace) -> Budget {
        // (a queue that is filled after a quiet while lets its first mesh go at once)
        Budget { pace, allowance: 0.0, rate: RATE, idle: HURRY, recent: VecDeque::with_capacity(RECENT + 1) }
    }

    /// The usual length of a frame: the middle one of the last few.
    fn usual(&self) -> Option<f32> {
        if self.recent.len() < RECENT / 2 {
            return None;
        }
        let mut v: Vec<f32> = self.recent.iter().copied().collect();
        v.sort_by(|a, b| a.total_cmp(b));
        Some(v[v.len() / 2])
    }

    /// A frame of `dt` seconds has gone by, and these sizes in bytes are
    /// waiting, the first to go first: how many of them go in this frame.
    /// One goes when the allowance covers it, or when nothing has gone for
    /// a tenth of a second; the allowance never grows past a frame's budget
    /// (`FRAME`), or the largest that waits where that is more, so a quiet
    /// while is not saved up and spent in one frame; and no more than a
    /// frame's budget goes in one frame, but for one piece larger than it.
    /// `hold` lets nothing go in this frame, which has work enough of its
    /// own; the allowance keeps.
    ///
    /// The rate follows the frames. One that ran a stop's length over what
    /// frames have been taking cuts it by three tenths; frames on time let it
    /// creep back, two in a hundred for every 120th of a second, so that it
    /// recovers as fast by the clock at thirty frames a second as at a
    /// hundred and twenty. The rule was first written with any frame half as
    /// long again as usual counting as late. But a year's meshes go whole: one
    /// of 50 MB costs the renderer 22 ms whatever the rate is, which at 120 Hz
    /// is three frames' length, so every such mesh cut the rate, the rate lay
    /// on its floor, and the pace was the tenth of a second's (a year took
    /// 1.6 s to arrive where 0.75 was meant). The rate decides how close
    /// together the meshes go, not how long each takes, so what should cut it
    /// is a frame long enough to be seen as a stop. When the scenery goes in
    /// pieces of a few megabytes the first rule will be the right one again.
    pub fn frame(&mut self, dt: f32, waiting: &[usize], hold: bool) -> usize {
        if dt > 0.0 {
            if let Some(usual) = self.usual() {
                let late = if self.pace == Pace::Strict { dt > LATE * usual } else { dt - usual > STOP };
                let creep = if self.pace == Pace::Strict { CREEP } else { 1.0 + (CREEP - 1.0) * dt.min(0.05) * 120.0 };
                self.rate = if late { (self.rate * CUT).max(RATE_FLOOR) } else { (self.rate * creep).min(RATE_CEILING) };
            }
            self.recent.push_back(dt);
            if self.recent.len() > RECENT {
                self.recent.pop_front();
            }
        }
        let dt = dt.clamp(1.0 / 240.0, 1.0 / 20.0);
        self.idle = (self.idle + dt).min(1e3);
        let Some(&largest) = waiting.iter().max() else {
            self.allowance = 0.0;
            return 0;
        };
        match self.pace {
            Pace::Whole => {
                self.idle = 0.0;
                waiting.len()
            }
            Pace::Step(secs) => {
                if self.idle < secs {
                    return 0;
                }
                self.idle = 0.0;
                1
            }
            Pace::Budget | Pace::Strict => {
                // (a frame's budget may be spent on several pieces, so the allowance keeps up to
                // that much, or the largest that waits where that is more)
                self.allowance = (self.allowance + self.rate as f64 * dt as f64).min(largest.max(FRAME) as f64);
                if hold {
                    return 0;
                }
                let (mut go, mut this_frame) = (0, 0);
                let budget = FRAME;
                for &size in waiting {
                    // no more than a frame's budget of time goes in one frame, whatever the rate
                    // allows: one piece at least, if it is let go at all
                    if go > 0 && this_frame + size > budget {
                        break;
                    }
                    this_frame += size;
                    if size as f64 <= self.allowance {
                        self.allowance -= size as f64;
                    } else if self.idle >= HURRY {
                        self.allowance = 0.0;
                    } else {
                        break;
                    }
                    self.idle = 0.0;
                    go += 1;
                }
                go
            }
        }
    }
}

/// How a mesh, or a piece of one, goes on screen.
pub enum How {
    /// assigned under the handle its entity draws, in the frame it goes: a
    /// mesh that goes whole (what lies flat, the smoke, and everything in stills)
    InPlace(Handle<Mesh>),
    /// added under a handle of its own; when the last piece of its mesh has
    /// gone, the mesh's entities take the new pieces in the next frame
    Banked,
}

/// One mesh, or one piece of a mesh, waiting to go on screen.
pub struct Piece {
    /// the year's mesh it is of, as a place in the table's list (`year::list`)
    pub part: usize,
    pub how: How,
    pub mesh: Mesh,
    /// what it will cost the card: its corners and its indices
    pub bytes: usize,
    /// whether it is the last piece of its mesh, and if so the whole mesh's
    /// fingerprint (`year::quick_sum`), kept as what is on screen of that mesh
    pub last: Option<u64>,
    /// on the last piece: the whole mesh's corners and a fingerprint of them,
    /// where a scripted run asked for one (`scenery`), kept as the record of
    /// what is on screen of that mesh
    pub print: Option<(usize, u64)>,
}

/// The year's meshes on their way to the renderer.
#[derive(Resource)]
pub struct HandOver {
    queue: VecDeque<Piece>,
    /// pieces that have gone to the renderer under handles of their own, by
    /// mesh, waiting for the last of their mesh
    arriving: HashMap<usize, Vec<Handle<Mesh>>>,
    /// meshes whose last piece went in an earlier frame: their entities take
    /// the new pieces in this one (the mesh, its fingerprint, its print)
    ready: Vec<(usize, u64, Option<(usize, u64)>)>,
    /// the pieces each mesh that goes in pieces is drawn from on screen, by
    /// mesh; held here, so that they are let go of when the next take their place
    pieces: HashMap<usize, Vec<Handle<Mesh>>>,
    /// each mesh's fingerprint on screen (`year::quick_sum`), by mesh: a year's
    /// mesh that is the same is not sent again
    pub sums: HashMap<usize, u64>,
    /// the year the queue is of
    year: Option<usize>,
    /// the state of the run its build was made from (`Shown::run`: how many
    /// years the run held), to be set beside the run's when the last has gone
    run: u32,
    /// nothing goes in this frame (`hold`)
    held: bool,
    pub budget: Budget,
    /// when the queue was filled, how many meshes it held and of how many bytes
    began: Option<(std::time::Instant, usize, usize)>,
    /// how many meshes (or pieces) have been handed over since the game began, and of how many bytes
    pub sent: (u64, u64),
    /// how many of a year's meshes were not sent, being the same as on screen, and of how many bytes
    pub same: (u64, u64),
    /// what was last put on screen of each mesh, by the handle the table's list gives it, by its
    /// fingerprint (`Piece::print`)
    pub on_screen: HashMap<AssetId<Mesh>, (usize, u64)>,
}

impl HandOver {
    pub fn new(pace: Pace) -> HandOver {
        HandOver {
            queue: VecDeque::new(),
            arriving: default(),
            ready: Vec::new(),
            pieces: default(),
            sums: default(),
            year: None,
            run: 0,
            held: false,
            budget: Budget::new(pace),
            began: None,
            sent: (0, 0),
            same: (0, 0),
            on_screen: default(),
        }
    }

    /// Nothing is waiting: the year last queued is on screen whole.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty() && self.ready.is_empty()
    }

    /// Whether a mesh goes in pieces of its own (not in stills, which take a year whole).
    pub fn in_pieces(&self) -> bool {
        self.budget.pace != Pace::Whole
    }

    /// How many meshes wait, and of how many bytes.
    pub fn waiting(&self) -> (usize, usize) {
        (self.queue.len(), self.queue.iter().map(|p| p.bytes).sum())
    }

    /// The year whose meshes are going out.
    pub fn year(&self) -> Option<usize> {
        self.year
    }

    /// What still waits, by name and in the order it will go, for a picture's caption
    /// (a mesh in pieces is named once, with how many of its pieces wait).
    pub fn waiting_names(&self) -> String {
        let parts = crate::year::list(crate::year::MESHES);
        let mut names: Vec<(usize, usize)> = Vec::new();
        for p in &self.queue {
            match names.last_mut() {
                Some((part, n)) if *part == p.part => *n += 1,
                _ => names.push((p.part, 1)),
            }
        }
        names.iter().map(|&(part, n)| {
            let name = crate::year::name(crate::year::MESHES, parts[part]);
            if n > 1 { format!("{name} ({n} pieces)") } else { name }
        }).collect::<Vec<_>>().join(", ")
    }

    /// Queue a finished build's meshes for year `k`, built from the run as it
    /// stood at `run` (how many years it held), in the order they are to go.
    pub fn fill(&mut self, k: usize, run: u32, pieces: Vec<Piece>) {
        debug_assert!(self.is_empty(), "a build was begun while the last was still going out");
        let bytes = pieces.iter().map(|p| p.bytes).sum();
        self.began = Some((std::time::Instant::now(), pieces.len(), bytes));
        self.queue.extend(pieces);
        self.year = Some(k);
        self.run = run;
    }

    /// A year's mesh was the same as the one on screen and is not sent: counted.
    pub fn same_as_on_screen(&mut self, bytes: usize) {
        self.same.0 += 1;
        self.same.1 += bytes as u64;
    }

    /// The last of the queue has gone: the year it was of, if it may be called
    /// built. It may not when a year arrived for that very year after its build
    /// began (the run now holds `run` years, of which `latest` is the newest):
    /// what went on screen is of the run as it was, and the year is built
    /// again. This is asked when the queue empties, of the run as it is then,
    /// so it does not hang on which system noticed the year arriving first.
    /// (In today's game the clock never stands in a year that has not arrived,
    /// and each year arrives once, so this does not happen; it is a guard.)
    fn finished(&mut self, run: u32, latest: usize) -> Option<usize> {
        let k = self.year.take()?;
        (run == self.run || k != latest).then_some(k)
    }

    /// Let no mesh go in this frame, which has work enough of its own: the
    /// one a year arrives in (the pump writes it into the run and lays out
    /// its works, the flags and fences are built, the panel is drawn again)
    /// and the one the clock's year changes in (the flags and fences, every
    /// cart and boat, the panel). A mesh on top of either made the longest
    /// stops there were at the fastest speed. Stills take no notice.
    pub fn hold(&mut self) {
        self.held = true;
    }
}

/// The entities that draw each of a year's meshes, by its place in the
/// table's list (`year::list`): one at the start (`year::spawn`), and one to
/// each piece once the mesh goes in pieces.
/// (Kept in `Handles`, beside the meshes' handles.)
#[derive(Default)]
pub struct Drawers(pub Vec<Vec<Entity>>);

/// Hand over what the budget lets go this frame; when the last of a year has
/// gone, that year is the one on screen.
#[allow(clippy::too_many_arguments)]
pub fn feed(
    mut hand: ResMut<HandOver>,
    mut tl: ResMut<Timeline>,
    data: Res<crate::Data>,
    real: Res<Time<Real>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut handles: ResMut<crate::Handles>,
    mut drawn: Query<&mut Mesh3d>,
    mut commands: Commands,
) {
    let _frames = crate::frames::span(crate::frames::Sys::HandOver);
    // (not marked changed in a frame nothing goes in: the budget only counts)
    let hand = hand.bypass_change_detection();
    // the meshes whose last piece went in an earlier frame (so that the renderer has taken every
    // piece in before anything draws them): their entities take the new pieces all in this frame,
    // a piece to an entity, the first entity's twins made or let go of to match, and the pieces
    // drawn until now are let go of
    for (part, sum, print) in std::mem::take(&mut hand.ready) {
        let new = hand.arriving.remove(&part).unwrap_or_default();
        if let Some(list) = handles.drawers.0.get_mut(part).filter(|l| !l.is_empty()) {
            for (j, h) in new.iter().enumerate() {
                match list.get(j) {
                    Some(&e) => {
                        if let Ok(mut m) = drawn.get_mut(e) {
                            m.0 = h.clone();
                        }
                    }
                    None => {
                        let twin = commands.entity(list[0]).clone_and_spawn().insert(Mesh3d(h.clone())).id();
                        list.push(twin);
                    }
                }
            }
            for e in list.drain(new.len().max(1)..) {
                commands.entity(e).despawn();
            }
        }
        hand.pieces.insert(part, new);
        hand.sums.insert(part, sum);
        if let (Some(print), Some(h)) = (print, handles.meshes.get(part)) {
            hand.on_screen.insert(h.id(), print);
        }
    }
    let sizes: Vec<usize> = hand.queue.iter().map(|p| p.bytes).collect();
    let held = std::mem::take(&mut hand.held);
    let go = hand.budget.frame(real.delta_secs(), &sizes, held);
    for _ in 0..go {
        let Some(piece) = hand.queue.pop_front() else { break };
        hand.sent.0 += 1;
        hand.sent.1 += piece.bytes as u64;
        match piece.how {
            How::InPlace(handle) => {
                // (a mesh assigned under the handle it had: the card frees the old one's place
                // before it finds the new one's, so its buffers do not grow from year to year)
                if let Some(mut on_screen) = meshes.get_mut(&handle) {
                    *on_screen = piece.mesh;
                }
                if let Some(sum) = piece.last {
                    hand.sums.insert(piece.part, sum);
                }
                if let Some(print) = piece.print {
                    hand.on_screen.insert(handle.id(), print);
                }
            }
            How::Banked => {
                // (under a handle of its own, so the card holds the old mesh and the new pieces
                // side by side until the switch: the most it grows by is one mesh)
                let h = meshes.add(piece.mesh);
                hand.arriving.entry(piece.part).or_default().push(h);
                if let Some(sum) = piece.last {
                    hand.ready.push((piece.part, sum, piece.print));
                }
            }
        }
    }
    // (a year all of whose meshes were the same as on screen is queued empty, and done here at once)
    if hand.year.is_some() && hand.is_empty() {
        let k = hand.year;
        match hand.finished(data.r.filled as u32, data.latest()) {
            Some(k) => tl.built = Some(k),
            None if k.is_some() && crate::timing() => eprintln!("  the year that went on screen was built from the run before its own year arrived: it is built again"),
            None => {}
        }
        if let (Some(k), Some((began, n, bytes)), true) = (k, hand.began.take(), crate::timing()) {
            eprintln!("  year {} on screen: its {n} meshes and pieces ({:.0} MB) went to the renderer over {:.0} ms, at {:.0} MB a second by the end; {} meshes ({:.0} MB) not sent since the start, being the same as on screen", data.r.y0 + k as i32, bytes as f32 / 1e6, began.elapsed().as_secs_f32() * 1e3, hand.budget.rate / 1e6, hand.same.0, hand.same.1 as f32 / 1e6);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Feed sizes through a budget at a steady frame rate: for each frame, what went in it.
    fn run(budget: &mut Budget, mut waiting: Vec<usize>, hz: f32, frames: usize) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        for _ in 0..frames {
            let n = budget.frame(1.0 / hz, &waiting, false);
            out.push(waiting.drain(..n).collect());
        }
        out
    }

    const MB: usize = 1_000_000;

    #[test]
    fn every_mesh_goes_in_order_and_none_before_the_allowance_covers_it() {
        // a year as the island has them: houses, works, smoke, a dozen of trees, the flat ones
        let year: Vec<usize> = [47, 22, 1, 48, 30, 12, 44, 28, 9, 40, 25, 11, 36, 21, 8, 5, 3, 2].iter().map(|m| m * MB).collect();
        let total: usize = year.iter().sum();
        let mut b = Budget::new(Pace::Budget);
        let frames = run(&mut b, year.clone(), 120.0, 240);
        // every one, in the order given
        let went: Vec<usize> = frames.iter().flatten().copied().collect();
        assert_eq!(went, year);
        // the first at once (nothing had gone for a while), and then never more in a frame
        // than the largest mesh, nor in any stretch more than the rate allows and one mesh over
        assert_eq!(frames[0], vec![47 * MB]);
        assert!(frames.iter().all(|f| f.iter().sum::<usize>() <= 48 * MB), "{frames:?}");
        let mut sent = 0usize;
        for (i, f) in frames.iter().enumerate() {
            sent += f.iter().sum::<usize>();
            let allowed = RATE_CEILING as f64 * (i + 1) as f64 / 120.0 + 48e6;
            assert!(sent as f64 <= allowed, "frame {i}: {sent} bytes sent, {allowed:.0} allowed");
        }
        // a year of 392 MB at 600 MB a second and rising: on screen within a second, and not in a blink
        let last = frames.iter().rposition(|f| !f.is_empty()).unwrap();
        assert_eq!(total, 392 * MB);
        assert!((30..120).contains(&last), "the last mesh went in frame {last}");
        // and with nothing waiting, nothing is saved up
        assert_eq!(b.frame(1.0 / 120.0, &[], false), 0);
        assert_eq!(b.allowance, 0.0);
    }

    #[test]
    fn a_mesh_waits_for_its_allowance_unless_nothing_has_gone_for_a_tenth_of_a_second() {
        let mut b = Budget::new(Pace::Budget);
        // 600 MB a second at 100 frames a second is 6 MB a frame: after the first, which goes
        // at once, a mesh of 30 MB waits five frames or six
        let frames = run(&mut b, vec![10 * MB, 30 * MB], 100.0, 8);
        assert_eq!(frames[0], vec![10 * MB]);
        let at = frames.iter().position(|f| f == &vec![30 * MB]).unwrap();
        assert!((5..=6).contains(&at), "{frames:?}");
        // a mesh far over what a tenth of a second allows goes after a tenth of a second all the same
        let mut b = Budget::new(Pace::Budget);
        let frames = run(&mut b, vec![MB, 500 * MB, 500 * MB], 100.0, 30);
        let at: Vec<usize> = frames.iter().enumerate().filter(|(_, f)| !f.is_empty()).map(|(i, _)| i).collect();
        assert_eq!(at.len(), 3);
        assert!((9..=11).contains(&(at[1] - at[0])) && (9..=11).contains(&(at[2] - at[1])), "{at:?}");
    }

    #[test]
    fn the_budget_is_by_the_second_whatever_the_frame_rate() {
        // the same year at 30 frames a second and at 120 takes about as long by the clock
        let year: Vec<usize> = vec![40 * MB; 10];
        let secs = |hz: f32| {
            let mut b = Budget::new(Pace::Budget);
            let frames = run(&mut b, year.clone(), hz, (hz * 3.0) as usize);
            (frames.iter().rposition(|f| !f.is_empty()).unwrap() + 1) as f32 / hz
        };
        let (slow, fast) = (secs(30.0), secs(120.0));
        assert!((slow - fast).abs() < 0.25, "{slow} s at 30 frames a second, {fast} s at 120");
        assert!(slow < 1.0 && fast < 1.0);
    }

    #[test]
    fn a_frame_that_was_a_stop_cuts_the_rate_and_frames_on_time_restore_it() {
        let mut b = Budget::new(Pace::Budget);
        for _ in 0..40 {
            b.frame(1.0 / 120.0, &[], false);
        }
        // forty frames on time: the rate has crept up from where it began
        let crept = b.rate;
        assert!(crept > RATE && crept <= RATE_CEILING);
        // a frame of three times the usual length is what one whole mesh costs, and no stop:
        // the rate is how close together the meshes go, and is left alone
        b.frame(3.0 / 120.0, &[], false);
        assert!(b.rate >= crept);
        // a frame that ran a stop's length over cuts it by three tenths
        let before = b.rate;
        b.frame(1.0 / 120.0 + 0.04, &[], false);
        assert!((b.rate - before * CUT).abs() < 1.0);
        // it is never cut below the floor, nor raised above the ceiling
        for _ in 0..40 {
            b.frame(0.2, &[], false);
            b.frame(1.0 / 120.0, &[], false);
        }
        assert!(b.rate >= RATE_FLOOR);
        for _ in 0..400 {
            b.frame(1.0 / 120.0, &[], false);
        }
        assert_eq!(b.rate, RATE_CEILING);
        // and it comes back as fast by the clock at thirty frames a second as at 120
        let after = |hz: f32| {
            let mut b = Budget::new(Pace::Budget);
            for _ in 0..(hz as usize) {
                b.frame(1.0 / hz, &[], false);
            }
            b.rate = RATE_FLOOR;
            for _ in 0..(hz as usize / 2) {
                b.frame(1.0 / hz, &[], false);
            }
            b.rate
        };
        assert!((after(30.0) / after(120.0) - 1.0).abs() < 0.1, "{} at 30, {} at 120", after(30.0), after(120.0));
    }

    #[test]
    fn no_more_than_a_frames_budget_goes_in_one_frame() {
        // a year in pieces of 6 MB, at the highest rate and thirty frames a second (40 MB a frame
        // by the rate): a frame takes two pieces at most, 12 MB, within the frame's budget
        let year: Vec<usize> = vec![6 * MB; 80];
        let mut b = Budget::new(Pace::Budget);
        b.rate = RATE_CEILING;
        let frames = run(&mut b, year.clone(), 30.0, 120);
        assert!(frames.iter().all(|f| f.iter().sum::<usize>() <= FRAME), "{frames:?}");
        assert_eq!(frames.iter().flatten().count(), year.len());
        assert!(frames.iter().filter(|f| f.len() == 2).count() > 20, "the budget is spent, not left over: {frames:?}");
        // a piece larger than the budget still goes, alone
        let mut b = Budget::new(Pace::Budget);
        let frames = run(&mut b, vec![MB, 30 * MB, MB], 30.0, 20);
        assert!(frames.iter().any(|f| f == &vec![30 * MB]), "{frames:?}");
    }

    /// A world with one of a year's meshes drawn by one entity, as `setup_world` leaves it, and
    /// a run of island 0 to build nothing from: for `feed`.
    fn world_with_one_mesh(pace: Pace) -> (World, Handle<Mesh>, Entity) {
        let mut world = World::new();
        let mut meshes = Assets::<Mesh>::default();
        let h = meshes.add(crate::placeholder_mesh());
        let e = world.spawn((Mesh3d(h.clone()), crate::Leaves(2))).id();
        let r = crate::world::tests::island();
        let g = crate::world::build_ground(&r);
        let (slots, cen) = (crate::scatter::make_slots(&r, &g), crate::network::centres(&r));
        world.insert_resource(crate::Data { r: std::sync::Arc::new(r), g: std::sync::Arc::new(g), slots: std::sync::Arc::new(slots), cen: std::sync::Arc::new(cen), live: false, arriving: None });
        world.insert_resource(meshes);
        world.insert_resource(crate::Handles { year_cells: Handle::default(), year_field: Handle::default(), meshes: vec![h.clone()], drawers: Drawers(vec![vec![e]]) });
        world.insert_resource(crate::Timeline { t: 0.0, built: None, playing: false, speed: 1.0 });
        world.insert_resource(Time::<Real>::default());
        world.insert_resource(HandOver::new(pace));
        (world, h, e)
    }

    fn one_piece(part: usize, x: f32, last: Option<u64>) -> Piece {
        let mut b = crate::meshkit::Builder::default();
        b.tri(Vec3::new(x, 0.0, 0.0), Vec3::new(x, 0.0, 0.1), Vec3::new(x + 0.1, 0.0, 0.0), [0.5, 0.5, 0.5, 1.0]);
        Piece { part, how: How::Banked, mesh: crate::meshkit::into_mesh(b), bytes: 132, last, print: None }
    }

    #[test]
    fn a_mesh_in_pieces_changes_on_screen_in_one_frame_and_its_old_pieces_are_let_go_of() {
        use bevy::ecs::system::RunSystemOnce;
        let (mut world, first, e) = world_with_one_mesh(Pace::Budget);
        let run = world.resource::<crate::Data>().r.filled as u32;
        world.resource_mut::<HandOver>().fill(0, run, vec![one_piece(0, 0.0, None), one_piece(0, 1.0, None), one_piece(0, 2.0, Some(7))]);
        // the first frame hands the three pieces to the renderer; the old mesh is still drawn whole
        world.run_system_once(feed).unwrap();
        assert_eq!(world.resource::<HandOver>().waiting().0, 0);
        assert_eq!(world.get::<Mesh3d>(e).unwrap().0, first);
        assert_eq!(world.resource::<crate::Timeline>().built, None, "not on screen until it is drawn");
        // the next, the three are drawn, by the entity and two twins of it, which are trees of
        // the same group; and the year is on screen
        world.run_system_once(feed).unwrap();
        let drawers = world.resource::<crate::Handles>().drawers.0[0].clone();
        assert_eq!(drawers.len(), 3);
        assert_eq!(drawers[0], e);
        let drawn: Vec<Handle<Mesh>> = drawers.iter().map(|&d| world.get::<Mesh3d>(d).unwrap().0.clone()).collect();
        assert!(drawn.iter().all(|h| *h != first));
        assert!(drawers.iter().all(|&d| world.get::<crate::Leaves>(d).is_some_and(|l| l.0 == 2)));
        assert_eq!(world.resource::<crate::Timeline>().built, Some(0));
        assert_eq!(world.resource::<HandOver>().sums.get(&0), Some(&7));
        // a year of one piece: the twins are let go of, and so are the three pieces
        let held = drawn.clone();
        world.resource_mut::<HandOver>().fill(0, run, vec![one_piece(0, 5.0, Some(8))]);
        world.run_system_once(feed).unwrap();
        world.run_system_once(feed).unwrap();
        let drawers = world.resource::<crate::Handles>().drawers.0[0].clone();
        assert_eq!(drawers, vec![e]);
        assert_eq!(world.resource::<HandOver>().pieces[&0].len(), 1);
        assert!(!held.contains(&world.get::<Mesh3d>(e).unwrap().0));
    }

    #[test]
    fn a_year_the_same_as_the_one_on_screen_is_on_screen_at_once() {
        use bevy::ecs::system::RunSystemOnce;
        let (mut world, _, _) = world_with_one_mesh(Pace::Budget);
        let run = world.resource::<crate::Data>().r.filled as u32;
        world.resource_mut::<HandOver>().fill(0, run, Vec::new());
        world.run_system_once(feed).unwrap();
        assert_eq!(world.resource::<crate::Timeline>().built, Some(0));
    }

    #[test]
    fn the_rule_as_first_written_cuts_the_rate_for_every_whole_mesh() {
        // what `Pace::Strict` keeps: at 120 Hz a mesh of 50 MB makes its frame three frames
        // long, which is "late", so a year of them holds the rate near its floor
        let mut b = Budget::new(Pace::Strict);
        for _ in 0..40 {
            b.frame(1.0 / 120.0, &[], false);
        }
        for _ in 0..12 {
            b.frame(3.0 / 120.0, &[], false);
            for _ in 0..8 {
                b.frame(1.0 / 120.0, &[], false);
            }
        }
        assert!(b.rate < 2.0 * RATE_FLOOR, "{}", b.rate);
    }

    #[test]
    fn a_held_frame_lets_nothing_go_and_keeps_the_allowance() {
        let mut b = Budget::new(Pace::Budget);
        assert_eq!(b.frame(1.0 / 120.0, &[10 * MB, 4 * MB], false), 1);
        // 5 MB a frame: the next would go in this frame, and goes in the one after instead
        assert_eq!(b.frame(1.0 / 120.0, &[4 * MB], true), 0);
        assert_eq!(b.frame(1.0 / 120.0, &[4 * MB], false), 1);
        // what the held frame allowed is not lost: 9 MB goes after two frames of 5, one of them held
        let mut b = Budget::new(Pace::Budget);
        assert_eq!(b.frame(1.0 / 120.0, &[10 * MB, 9 * MB], false), 1);
        assert_eq!(b.frame(1.0 / 120.0, &[9 * MB], true), 0);
        assert_eq!(b.frame(1.0 / 120.0, &[9 * MB], false), 1);
        // stills take no notice
        let mut b = Budget::new(Pace::Whole);
        assert_eq!(b.frame(0.0, &[10 * MB, 4 * MB], true), 2);
    }

    #[test]
    fn a_year_whose_own_year_arrived_after_its_build_began_is_not_counted_built() {
        let piece = || Piece { part: 0, how: How::InPlace(Handle::default()), mesh: Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::RENDER_WORLD), bytes: 120, last: Some(0), print: None };
        let mut hand = HandOver::new(Pace::Budget);
        // a year queued and let go (as `feed` does), then asked about with the run as it then is
        let mut out = |k: usize, built_from: u32, run: u32, latest: usize| {
            hand.fill(k, built_from, vec![piece()]);
            hand.queue.clear();
            hand.finished(run, latest)
        };
        // year 6 built while the run held seven years (0 to 6): the run as it was when it went out
        assert_eq!(out(6, 7, 7, 6), Some(6));
        // year 6 built from a run that held six years (0 to 5), and year 6 came in while it was
        // built or went out: what went on screen is of the run before its own year, and it is
        // built again
        assert_eq!(out(6, 6, 7, 6), None);
        // a year arriving that is not the one going out leaves it as it was
        assert_eq!(out(5, 6, 7, 6), Some(5));
        // and a year is counted once
        assert_eq!(hand.finished(7, 6), None);
    }

    #[test]
    fn stills_take_the_whole_queue_at_once_and_a_step_takes_one() {
        let mut b = Budget::new(Pace::Whole);
        assert_eq!(b.frame(0.0, &[400 * MB, 30 * MB, MB], false), 3);
        let mut b = Budget::new(Pace::Step(0.5));
        let frames = run(&mut b, vec![MB; 4], 20.0, 45);
        let at: Vec<usize> = frames.iter().enumerate().filter(|(_, f)| !f.is_empty()).map(|(i, _)| i).collect();
        assert_eq!(at.len(), 4);
        assert!(at.windows(2).all(|w| (9..=11).contains(&(w[1] - w[0]))), "{at:?}");
    }
}
