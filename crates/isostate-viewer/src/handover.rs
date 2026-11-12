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
//! No build is begun while the queue holds anything (`rebuild_year`), and the
//! year counts as built when the queue has emptied: a year is shown whole or
//! not at all, and when years come faster than they can be shown some are
//! skipped and none is left half sent.
//!
//! Where the run is there to take stills (`--view`, `--shots`), the whole
//! queue goes in the frame it was filled in, as it always did: nobody is
//! watching, and the pictures cannot differ. `ISOSTATE_HANDOVER` is for
//! measuring and for looking: `whole` does the same in a game (the freeze as
//! it was, to set beside the cure), `strict` keeps the rate's rule as it was
//! first written, and `step:SECONDS` hands over one mesh every so many
//! seconds, slowly enough to take pictures of a year half arrived.

use std::collections::VecDeque;

use bevy::prelude::*;

use crate::Timeline;

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
    /// a tenth of a second; the allowance never grows past the largest
    /// that waits, so a quiet while is not saved up and spent in one frame.
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
                self.allowance = (self.allowance + self.rate as f64 * dt as f64).min(largest as f64);
                if hold {
                    return 0;
                }
                let mut go = 0;
                for &size in waiting {
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

/// One mesh waiting to go on screen.
pub struct Piece {
    pub handle: Handle<Mesh>,
    pub mesh: Mesh,
    /// what it will cost the card: its corners and its indices
    pub bytes: usize,
    /// its corners and a fingerprint of them, where a scripted run asked for one
    /// (`scenery`): kept as the record of what is on screen under that handle
    pub print: Option<(usize, u64)>,
}

/// The year's meshes on their way to the renderer.
#[derive(Resource)]
pub struct HandOver {
    queue: VecDeque<Piece>,
    /// the year the queue is of
    year: Option<usize>,
    /// the run moved on under the queue's year while it was going out: it is
    /// on screen, but not to be called built
    stale: bool,
    /// nothing goes in this frame (`hold`)
    held: bool,
    pub budget: Budget,
    /// when the queue was filled, how many meshes it held and of how many bytes
    began: Option<(std::time::Instant, usize, usize)>,
    /// how many meshes have been handed over since the game began, and of how many bytes
    pub sent: (u64, u64),
    /// what was last handed over under each handle, by its fingerprint (`Piece::print`)
    pub on_screen: std::collections::HashMap<AssetId<Mesh>, (usize, u64)>,
}

impl HandOver {
    pub fn new(pace: Pace) -> HandOver {
        HandOver { queue: VecDeque::new(), year: None, stale: false, held: false, budget: Budget::new(pace), began: None, sent: (0, 0), on_screen: default() }
    }

    /// Nothing is waiting: the year last queued is on screen whole.
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    /// How many meshes wait, and of how many bytes.
    pub fn waiting(&self) -> (usize, usize) {
        (self.queue.len(), self.queue.iter().map(|p| p.bytes).sum())
    }

    /// The year whose meshes are going out.
    pub fn year(&self) -> Option<usize> {
        self.year
    }

    /// What still waits, by name and in the order it will go, for a picture's caption.
    pub fn waiting_names(&self, handles: &crate::Handles) -> String {
        let names = ["houses", "works", "smoke", "yards and lanes", "roads", "railway"];
        let all = handles.year();
        let name = |p: &Piece| match all.iter().position(|h| h.id() == p.handle.id()) {
            Some(i) if i < names.len() => names[i].to_string(),
            Some(i) => format!("trees {} {}", (i - names.len()) / 3, ["evergreen", "leaves", "bare limbs"][(i - names.len()) % 3]),
            None => "?".to_string(),
        };
        self.queue.iter().map(name).collect::<Vec<_>>().join(", ")
    }

    /// Queue a finished build's meshes for year `k`, in the order they are to go.
    pub fn fill(&mut self, k: usize, pieces: Vec<Piece>) {
        debug_assert!(self.queue.is_empty(), "a build was begun while the last was still going out");
        let bytes = pieces.iter().map(|p| p.bytes).sum();
        self.began = Some((std::time::Instant::now(), pieces.len(), bytes));
        self.queue.extend(pieces);
        self.year = Some(k);
        self.stale = false;
    }

    /// The run has changed under the year that is going out (a live game's
    /// year arrived for it): what is on its way is of the run as it was.
    pub fn outdated(&mut self) {
        self.stale = true;
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

/// Hand over what the budget lets go this frame; when the last of a year has
/// gone, that year is the one on screen.
pub fn feed(mut hand: ResMut<HandOver>, mut tl: ResMut<Timeline>, data: Res<crate::Data>, real: Res<Time<Real>>, mut meshes: ResMut<Assets<Mesh>>) {
    let _frames = crate::frames::span(crate::frames::Sys::HandOver);
    // (not marked changed in a frame nothing goes in: the budget only counts)
    let hand = hand.bypass_change_detection();
    let sizes: Vec<usize> = hand.queue.iter().map(|p| p.bytes).collect();
    let held = std::mem::take(&mut hand.held);
    let go = hand.budget.frame(real.delta_secs(), &sizes, held);
    for _ in 0..go {
        let Some(piece) = hand.queue.pop_front() else { break };
        // (a mesh assigned under the handle it had: the card frees the old one's place before
        // it finds the new one's, so its buffers do not grow from year to year)
        if let Some(mut on_screen) = meshes.get_mut(&piece.handle) {
            *on_screen = piece.mesh;
            hand.sent.0 += 1;
            hand.sent.1 += piece.bytes as u64;
            if let Some(print) = piece.print {
                hand.on_screen.insert(piece.handle.id(), print);
            }
        }
    }
    if go > 0 && hand.queue.is_empty() {
        if let Some(k) = hand.year.take() {
            if !hand.stale {
                tl.built = Some(k);
            }
            if let (Some((began, n, bytes)), true) = (hand.began.take(), crate::timing()) {
                eprintln!("  year {} on screen: its {n} meshes ({:.0} MB) went to the renderer over {:.0} ms, at {:.0} MB a second by the end", data.r.y0 + k as i32, bytes as f32 / 1e6, began.elapsed().as_secs_f32() * 1e3, hand.budget.rate / 1e6);
            }
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
