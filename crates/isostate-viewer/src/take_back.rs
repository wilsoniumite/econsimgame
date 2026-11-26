//! Taking back on screen (`docs/designs/sandbox.md`, 4.4 and 6.8, stage 8c):
//! the chips that take the last edit by hand back, and what is said when
//! they are pressed and answered. The game's thread keeps one copy of the
//! island from just before the last edit (live.rs, `Back`) and says where it
//! stands and what the edit was (`Standing::back` and `Standing::last`); a
//! press sends `Cmd::TakeBack`, and back.rs forgets what came after.
//!
//! The chips: "Take back the last edit" in the tray's second row, beside Run
//! on, with a muted line under the tray's tools saying where the island would
//! go back to and before what ("The island goes back to January 1791,
//! before: By hand: Melbourne's people, 19,380 to 9,690."); and "Take it
//! back" on the cards that carry a hand's edit, while that edit is the last:
//! a town's ideas card after a discovery there (town_ideas.rs) and a rule's
//! row on the Rules tab after it was set by hand (rulebook.rs). Every one of
//! them is the same order, so a card's chip takes back exactly what the
//! tray's would. The words of the whole phrase stay on the tray's chip so that
//! a script's `click:Take back` still finds the Hand tab's smaller chip on an
//! edit that waits (which cancels that order, and is another thing).
//!
//! A press is under the tray's guards (the clock at the present, nothing out,
//! no year turning, no run on) and stops the clock; the order goes once no
//! month is on its way (`Live::stepping`), since a month asked for before it
//! would be played on the island taken back. The answer goes up as a toast in
//! the hand's ink, "Taken back to January 1791: the island stands as it did
//! before Melbourne's people were halved, and runs on from there without it.",
//! and a refusal as a red one in its own words.

// see the note at the top of ui.rs
#![allow(deprecated)]

use bevy::prelude::*;
use isostate_core::edits::{Edit, PinWhen, Purse};

use crate::game::Game;
use crate::live::{Cmd, Live, Standing};
use crate::{clock, Data, Timeline, Toasts};

/// The words of the tray's chip: the whole phrase, which a script clicks by.
pub const TRAY_CHIP: &str = "Take back the last edit";
/// The words of a card's chip.
pub const CARD_CHIP: &str = "Take it back";

/// A chip that takes the last edit by hand back: the tray's or a card's.
#[derive(Component)]
pub struct TakeBackChip;

/// A take back pressed and not yet answered.
#[derive(Resource, Default)]
pub struct Undo {
    /// pressed, and to be sent once no month is on its way
    armed: bool,
    /// sent, and how many answers the game had had then (`Game::answered`)
    sent: Option<u64>,
    /// where the island goes back to ("January 1791"), and what it stands
    /// before ("Melbourne's people were halved"), for the toast
    to: String,
    before: String,
}

impl Undo {
    /// A take back is pressed and its answer has not come (a script's `settle` waits on it).
    pub fn out(&self) -> bool {
        self.armed || self.sent.is_some()
    }
}

pub fn plugin(app: &mut App) {
    app.init_resource::<Undo>().add_systems(
        Update,
        (
            press.before(crate::game::pump),
            send.after(press).before(crate::game::pump),
            // (after the answer is read and the island forgotten: back.rs)
            answered.after(crate::back::taken_back).before(crate::game::ask_site),
        )
            .run_if(resource_exists::<crate::sandbox::Sandbox>)
            .run_if(resource_exists::<Live>),
    );
}

/// The month the copy stands at, as the clock says it: "January 1791" for
/// the turn into 1791, "June 1791" five months into it.
pub fn date_of(back: (i64, usize)) -> String {
    clock::month_year(clock::Date { year: back.0 as i32 + 1, month: back.1.min(11), part: 0.0 })
}

/// The muted line under the tray while there is something to take back.
pub fn tray_line(s: &Standing) -> Option<String> {
    let (back, last) = (s.back?, s.last.as_ref()?);
    Some(format!("The island goes back to {}, before: {}.", date_of(back), last.said.trim_end_matches('.')))
}

/// The edit that would be taken back, while there is one.
pub fn last_edit(s: &Standing) -> Option<&Edit> {
    s.back.and(s.last.as_ref()).map(|l| &l.edit)
}

/// What a factor did, in a word or two: "halved", "raised by a tenth".
fn by(f: f64) -> String {
    let near = |x: f64| (f - x).abs() < 1e-9;
    match () {
        _ if near(0.5) => "halved".to_string(),
        _ if near(2.0) => "doubled".to_string(),
        _ if near(1.1) => "raised by a tenth".to_string(),
        _ if near(1.5) => "raised by half".to_string(),
        _ if near(0.9) => "cut by a tenth".to_string(),
        _ if near(0.7) => "cut by three tenths".to_string(),
        _ => format!("multiplied by {f}"),
    }
}

/// What an edit did, in words that follow "before" ("Melbourne's people
/// were halved"), with the simulation's names: made on the game's thread
/// when the edit is given (live.rs).
pub fn before_words(s: &isostate_core::Sim, edit: &Edit) -> String {
    let town = |t: usize| s.st.town_name.get(t).cloned().unwrap_or_else(|| format!("town {t}"));
    let w = s.st.world.W.max(1);
    let at = |c: usize| format!("{}, {}", c % w, c / w);
    let near = |c: usize| s.st.catch.get(c).and_then(|&t| (t >= 0).then(|| town(t as usize)));
    let idea = |name: &str| s.content.tech_name.iter().position(|n| n == name).and_then(|i| s.content.tech_label.get(i).cloned()).unwrap_or_else(|| name.replace('_', " "));
    let way = |name: &str| s.content.method_name.iter().position(|n| n == name).and_then(|m| s.content.method_label.get(m).cloned()).unwrap_or_else(|| name.replace('_', " "));
    match edit {
        Edit::Money { purse, town: t, factor } => {
            let whose = match purse {
                Purse::Households => "households'",
                Purse::Landlords => "landlords'",
                Purse::Merchants => "merchants'",
            };
            format!("the {whose} money in {} was {}", t.map_or_else(|| "every town".to_string(), town), by(*factor))
        }
        Edit::People { town: t, factor } => format!("{}'s people were {}", town(*t), by(*factor)),
        Edit::Harvest { town: t, factor } => format!("the rest of the year's crop {} was {}", t.map_or_else(|| "on the whole island".to_string(), |t| format!("round {}", town(t))), by(*factor)),
        Edit::Discover { tech, town: t } => format!("{} was found in {} by hand", idea(tech), town(*t)),
        Edit::Rule { name, value } => format!("the rule {name} was set to {}", crate::rulebook::figure(value)),
        Edit::FoundTown { cell } => match near(*cell) {
            Some(t) => format!("a town was ordered at {}, in {t}'s district", at(*cell)),
            None => format!("a town was ordered at {}", at(*cell)),
        },
        Edit::BuildWorks { method, cell } => {
            let label = s.content.method_name.iter().position(|n| n == method).map_or(method.clone(), |m| s.content.method_label[m].to_lowercase());
            format!("a works ({label}) was ordered at {}", at(*cell))
        }
        Edit::CloseWorks { works } => format!("{} was ordered closed", s.st.f_name.get(*works).cloned().unwrap_or_else(|| format!("works {works}"))),
        Edit::Pin { idea: name, year, when } => format!("{} was {} {year}", idea(name), if *when == PinWhen::In { "pinned to" } else { "held back until" }),
        Edit::FoundUniversity { cell } => match near(*cell) {
            Some(t) => format!("a university was founded in {t}"),
            None => "a university was founded".to_string(),
        },
        Edit::CloseUniversity { town: t } => format!("the University of {} was closed", town(*t)),
        // the hand on a town's making (town_flow.rs, edits.rs TOWN-FLOW-HAND)
        Edit::Forbid { town: t, method } => format!("{} was turned off in {} by hand", way(method), town(*t)),
        Edit::Allow { town: t, method } => format!("{} was turned on again in {} by hand", way(method), town(*t)),
        Edit::CloseKind { town: t, good } => {
            let label = s.content.good_name.iter().position(|n| n == good).map_or(good.replace('_', " "), |g| s.content.good_label[g].to_lowercase());
            format!("the works making {label} in {} were ordered closed and forbidden", town(*t))
        }
        // GROUND-BRUSH (ground.rs)
        Edit::Ground { kind, lay, squares, .. } => crate::ground::before_words(*kind, *lay, squares, &at, &near),
    }
}

/// Whether the hand may act now, or the tray's refusal: the present only, and
/// one thing at a time.
fn may_take_back(tl: &Timeline, data: &Data, live: &Live, game: &Game, running: bool) -> Result<(), &'static str> {
    if (tl.t.floor() as usize) < data.latest() || data.r.filled >= data.r.ny {
        return Err(crate::sandbox::PAST);
    }
    if game.busy || live.turning() || running {
        return Err(crate::sandbox::BUSY);
    }
    Ok(())
}

/// A take back chip pressed: under the tray's guards, the clock is stopped and
/// the order armed, to go once no month is on its way.
#[allow(clippy::too_many_arguments)]
fn press(
    q: Query<&Interaction, (Changed<Interaction>, With<TakeBackChip>)>,
    mut undo: ResMut<Undo>,
    mut tl: ResMut<Timeline>,
    data: Res<Data>,
    live: Res<Live>,
    mut game: ResMut<Game>,
    sb: Res<crate::sandbox::Sandbox>,
    mut toasts: ResMut<Toasts>,
    time: Res<Time>,
) {
    if !q.iter().any(|i| *i == Interaction::Pressed) {
        return;
    }
    let now = time.elapsed_secs();
    if let Err(why) = may_take_back(&tl, &data, &live, &game, sb.running()) {
        toasts.say("orders_refused", why, "By hand", now);
        return;
    }
    let (Some(back), Some(last)) = (live.standing.back, live.standing.last.as_ref()) else {
        toasts.say("orders_refused", crate::live::NOTHING_TO_TAKE_BACK, "By hand", now);
        return;
    };
    (undo.to, undo.before) = (date_of(back), last.before.clone());
    undo.armed = true;
    tl.playing = false;
    game.follow = None;
    // (nothing else may be sent until the answer comes)
    game.busy = true;
    game.note = None;
}

/// The armed take back goes once no month is on its way.
fn send(mut undo: ResMut<Undo>, live: Res<Live>, game: Res<Game>) {
    if !undo.armed || live.stepping() {
        return;
    }
    undo.armed = false;
    undo.sent = Some(game.answered);
    live.send(Cmd::TakeBack);
}

/// The answer: a toast in the hand's ink, or the refusal in red.
fn answered(mut undo: ResMut<Undo>, game: Res<Game>, data: Res<Data>, live: Res<Live>, mut toasts: ResMut<Toasts>, time: Res<Time>) {
    let Some(at) = undo.sent else { return };
    if game.busy || game.answered <= at {
        return;
    }
    undo.sent = None;
    let when = clock::month_year(clock::date(data.r.y0, clock::at(data.latest(), live.months.min(11))));
    match &game.note {
        Some((true, line)) if line.starts_with("Taken back") => {
            let said = taken_back_words(&undo.to, &undo.before);
            toasts.say("hand", &said, &when, time.elapsed_secs());
            println!("sandbox: {when}: {said}");
        }
        Some((_, why)) => {
            toasts.say("orders_refused", why, "By hand", time.elapsed_secs());
            println!("sandbox: refused, {when}: {why}");
        }
        None => {}
    }
}

/// The toast of an edit taken back.
pub fn taken_back_words(to: &str, before: &str) -> String {
    format!("Taken back to {to}: the island stands as it did before {before}, and runs on from there without it.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::Last;

    #[test]
    fn the_tray_says_where_the_island_goes_back_to_and_before_what() {
        let mut s = Standing::default();
        assert_eq!((tray_line(&s), last_edit(&s)), (None, None), "nothing to take back");
        let edit = Edit::People { town: 3, factor: 0.5 };
        s.back = Some((1790, 0));
        s.last = Some(Last { edit: edit.clone(), said: "By hand: Melbourne's people, 19,380 to 9,690".into(), before: "Melbourne's people were halved".into() });
        assert_eq!(tray_line(&s).unwrap(), "The island goes back to January 1791, before: By hand: Melbourne's people, 19,380 to 9,690.");
        assert_eq!(last_edit(&s), Some(&edit));
        // five months into 1791, and an answer that ends in a full stop already
        s.back = Some((1790, 5));
        s.last.as_mut().unwrap().said = "Ordered by hand: a works (brickworks) at 31, 40, at the year's turn, in January 1792.".into();
        assert_eq!(tray_line(&s).unwrap(), "The island goes back to June 1791, before: Ordered by hand: a works (brickworks) at 31, 40, at the year's turn, in January 1792.");
        assert_eq!(taken_back_words("January 1791", "Melbourne's people were halved"), "Taken back to January 1791: the island stands as it did before Melbourne's people were halved, and runs on from there without it.");
        // with no copy the edit is not offered, whatever was last said
        s.back = None;
        assert_eq!((tray_line(&s), last_edit(&s)), (None, None));
        assert_eq!([by(0.5), by(2.0), by(1.1), by(0.9), by(1.25)], ["halved", "doubled", "raised by a tenth", "cut by a tenth", "multiplied by 1.25"]);
    }
}
