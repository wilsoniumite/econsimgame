//! IDEAS-PLATE: the island's ideas as a timeline, for the game's panel (the
//! atlas's third plate in the game, `docs/designs/sandbox.md`, "Built: the
//! island's ideas as a timeline"). Not in the Python; it only reads.
//!
//! The designer asked on 19 November 2026: "Can I see a tech timeline? Move
//! things around in it?" The atlas's page answers the first half for eight or
//! sixteen islands from the command line (`atlas_ideas.rs`). This is the same
//! plate for the one island being played, as it stands: every idea of the data
//! the run plays (the tree's methods and techniques, and the uses of the tasks
//! rule), each a row, in groups as a town's card groups them (the trades of
//! the works, the ways of working, the machines for tasks), and in each group
//! in the atlas's order, by history's year.
//!
//! For each row: the year and the town of its first find, and how it was
//! found (a draw, a pin or the hand); the towns that know it now, each with
//! the year it came to know it, which is the band of its spread from town to
//! town; history's year (the data's `target`); for one not yet found, what it
//! waits on and, only where the rule's own chance gives one, the year it is
//! likely by; a pin by hand, with its year; and what a drag of its mark along
//! the row may do (`drop_choices`), or the sentence that says why nothing can.
//!
//! It is asked of the game's thread as a card is (`inspect::Thing::Ideas`),
//! with what the thread has watched of the towns' knowledge (`inspect::Learnt`),
//! and made between two months. A simulation that is told about a thousand
//! times runs exactly as one that is never asked.

use crate::edits::{self, Edit, Idea, PinWhen};
use crate::families;
use crate::inspect::{and_list, lower, Learnt, KNOWS};
use crate::sim::Sim;
use crate::util::fmt_pct;

/// The island's ideas as they stand.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IslandIdeas {
    /// The run's first year, the year in progress, and the run's last year.
    pub from: i64,
    pub now: i64,
    pub end: i64,
    /// The towns standing.
    pub towns: usize,
    pub groups: Vec<IdeaGroup>,
}

impl IslandIdeas {
    /// Every row, group by group.
    pub fn rows(&self) -> impl Iterator<Item = &IdeaRow> {
        self.groups.iter().flat_map(|g| g.rows.iter())
    }

    /// The row of an idea of the tree, by its name in the data.
    pub fn row(&self, name: &str) -> Option<&IdeaRow> {
        self.rows().find(|r| r.name == name && r.kind != "use")
    }

    /// The row of a use of the tasks rule, by its name in the data (a machine
    /// that tends has the name of the idea of the tree it waits on).
    pub fn use_row(&self, name: &str) -> Option<&IdeaRow> {
        self.rows().find(|r| r.name == name && r.kind == "use")
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct IdeaGroup {
    pub name: String,
    pub rows: Vec<IdeaRow>,
}

/// What a drag of a row's mark may do: the years a pin may name, and the town
/// an idea of the tree found now would be found in.
#[derive(Clone, Debug, PartialEq)]
pub struct Drag {
    /// The first year a pin may find it in: this year for an idea of the
    /// tree (looked for at the year's end), the next for a use (found at a
    /// year's turn).
    pub first: i64,
    /// The first year a hold may name and still hold anything back.
    pub hold_from: i64,
    /// An idea of the tree: the town it is found in if it is found now (the
    /// town a pin would choose, which handled most of what it is found on
    /// last year), with its name.
    pub now_town: Option<(usize, String)>,
}

/// One idea's row.
#[derive(Clone, Debug, PartialEq)]
pub struct IdeaRow {
    /// As the data names it (`Content::tech_name`, or a use's name).
    pub name: String,
    pub label: String,
    /// "method", "technique" or "use".
    pub kind: &'static str,
    /// History's year, the data's `target`.
    pub target: Option<i64>,
    /// The year it was found (the run's first year for one known before it).
    pub found: Option<i64>,
    /// Known before the island's history began.
    pub at_start: bool,
    /// The town of its first find (an idea of the tree), and its name.
    pub town: Option<(usize, String)>,
    /// How it was found, in a few words.
    pub how: String,
    /// Found by the hand or by a pin.
    pub by_hand: bool,
    /// The towns standing that know it now, each with the year it came to
    /// know it where that was watched, earliest first (an idea of the tree).
    pub learnt: Vec<(String, Option<i64>)>,
    /// The year the last town standing came to know it, once every one does.
    pub spread_to: Option<i64>,
    /// A pin by hand standing: its year, and what it asks.
    pub pin: Option<(i64, PinWhen)>,
    /// What one not yet found waits on, in words; empty once found.
    pub waits: String,
    /// One not yet found: the year it is likely by, where the rule's own
    /// chance gives one (`likely_words` says how).
    pub likely: Option<i64>,
    pub likely_words: String,
    /// Where its mark stands to be dragged: the pin's year, or the likely
    /// year, or history's year if it is still to come, or this year.
    pub handle: i64,
    /// The lines under the pointer: its name, what it is and opens, where and
    /// when it was found, which towns know it, history's year, what it waits on.
    pub said: Vec<String>,
    /// What a drag may do, or the sentence that says why nothing can.
    pub drag: Result<Drag, String>,
}

/// One chip offered when a row's mark is dropped on a year: its words, the
/// edit it sends, and the line that says what it does.
#[derive(Clone, Debug, PartialEq)]
pub struct Offer {
    pub label: String,
    pub edit: Edit,
    pub line: String,
}

/// How each idea of the tree was found, from the island's news: by hand, by
/// a pin or by a draw (as a town's card reads it).
fn found_by(sim: &Sim) -> Vec<Option<(&'static str, bool)>> {
    let C = &sim.content;
    let mut by: Vec<Option<(&'static str, bool)>> = vec![None; C.NT];
    for ev in &sim.events {
        if let Some(k) = ev.tech.filter(|&k| k < C.NT) {
            if by[k].is_none() && (ev.kind == "hand" && ev.title.contains(" found in ") || ev.kind == "discovery") {
                by[k] = Some(if ev.kind == "hand" {
                    ("by hand", true)
                } else if ev.title.ends_with("pinned to") {
                    ("by a pin, in the year it was pinned to", true)
                } else {
                    ("by a draw, on what the island had made", false)
                });
            }
        }
    }
    by
}

/// The towns that know an idea, in a sentence: "Kelside (1826), Melbourne
/// (1829) and 12 more".
fn towns_words(learnt: &[(String, Option<i64>)], most: usize) -> String {
    let mut v: Vec<String> = learnt.iter().take(most).map(|(n, y)| y.map_or(n.clone(), |y| format!("{n} ({y})"))).collect();
    let more = learnt.len().saturating_sub(most);
    if more > 0 {
        v.push(format!("{more} more"));
    }
    and_list(&v)
}

/// The island's ideas, from the state as it stands and, for the year each
/// town came to know each idea, from `learnt` where the caller keeps one.
pub fn island_ideas(sim: &Sim, learnt: Option<&Learnt>) -> IslandIdeas {
    let (st, C, P) = (&sim.st, &sim.content, &sim.params);
    let (start, now) = (C.start_year, sim.year);
    let end = P.end_year.unwrap_or(C.end_year);
    let active: Vec<usize> = (0..st.T_max.min(st.town_name.len())).filter(|&i| st.town_active[i]).collect();
    let by = found_by(sim);
    let pin_of = |idea: Idea| st.by_hand.pins.iter().find(|p| p.idea == idea).map(|p| (p.year, p.when));
    let pin_words = |p: Option<(i64, PinWhen)>| match p {
        Some((y, PinWhen::In)) => format!("Pinned by hand to {y}: the island will not find it before then, and finds it in that year whatever it has made."),
        Some((y, PinWhen::NotBefore)) => format!("Held back by hand until {y}: the island will not find it before then, and after it finds it as it would have."),
        None => String::new(),
    };
    let history = |target: Option<i64>| target.map_or("The data gives it no year in our own history.".to_string(), |y| format!("Our own history had it in {y}."));
    let mut tree: Vec<(Option<families::Group>, usize, IdeaRow)> = Vec::new();
    for k in 0..C.NT {
        let method = k < C.M;
        let kind = if method { "method" } else { "technique" };
        let target = C.tech_target.get(k).copied().filter(|&y| y > 0);
        let at_start = C.tech_known[k];
        let found = if at_start { Some(start) } else { (st.tech_year[k] >= 0).then_some(st.tech_year[k]) };
        let town = (!at_start && st.tech_town[k] >= 0).then(|| st.tech_town[k] as usize).map(|t| (t, st.town_name.get(t).cloned().unwrap_or_default()));
        let (how, by_hand) = if at_start {
            (format!("known before {start}"), false)
        } else if found.is_some() {
            by[k].map_or(("by a draw, on what the island had made".to_string(), false), |(w, h)| (w.to_string(), h))
        } else {
            ("not yet found".to_string(), false)
        };
        // the towns that know it now, each with the year it came to know it
        let mut learnt_v: Vec<(String, Option<i64>)> = Vec::new();
        let mut every = !active.is_empty();
        let mut last: Option<i64> = None;
        for &i in &active {
            if st.know[(i, k)] > KNOWS {
                let y = learnt
                    .and_then(|l| l.year(i, k))
                    .or((st.tech_town[k] == i as i64 && st.tech_year[k] >= 0).then_some(st.tech_year[k]))
                    .or((at_start && st.town_founded[i] <= start).then_some(start));
                if y.is_none() {
                    every = false;
                }
                last = last.max(y);
                learnt_v.push((st.town_name[i].clone(), y));
            } else {
                every = false;
            }
        }
        learnt_v.sort_by_key(|(n, y)| (y.unwrap_or(i64::MAX), n.clone()));
        let spread_to = if every { last } else { None };
        // what it is and what it opens
        let opens = if method {
            let good = lower(&C.good_label[C.method_out[k]]);
            let trade = families::Group::of_method(C, k).map_or(String::new(), |g| format!(", in {}", lower(g.label())));
            let n = (0..st.F_count).filter(|&f| st.f_alive[f] && st.f_method[f] == k).count();
            let stand = match n {
                0 => "none stands on the island now".to_string(),
                1 => "1 stands on the island now".to_string(),
                n => format!("{n} stand on the island now"),
            };
            format!("A way of making {good}: it opens works of the kind {}{trade}; {stand}.", lower(&C.method_label[k]))
        } else {
            let what = crate::inspect::technique_words(&C.tech_name[k]).map_or(String::new(), |w| format!(": {w}"));
            format!("A way of working, with no works of its own{what}.")
        };
        let uses: Vec<String> = C.tasks.uses.iter().filter(|u| u.idea == Some(k)).map(|u| lower(&edits::use_label(&u.name))).collect();
        let opens = if uses.is_empty() { opens } else { format!("{opens} The machines for tasks {} wait on it.", and_list(&uses)) };
        let pin = pin_of(Idea::Tech(k));
        let mut said = vec![format!("{} ({kind})", C.tech_label[k]), opens];
        let (mut waits, mut likely, mut likely_words) = (String::new(), None, String::new());
        let drag: Result<Drag, String>;
        match (found, &town) {
            (Some(_), _) if at_start => {
                said.push(format!("Known before {start}, when the island's history begins."));
                drag = Err(format!("{} was known before the island's history began: there is no find to move.", C.tech_label[k]));
            }
            (Some(y), _) => {
                let place = town.as_ref().map_or(String::new(), |(_, n)| format!(" in {n}"));
                said.push(format!("Found{place} in {y}, {how}."));
                drag = Err(format!("{} was found{place} in {y}: a find cannot be unmade but by taking it back, and only while it is the last edit by hand.", C.tech_label[k]));
            }
            (None, _) => {
                said.push("Not yet found anywhere on the island.".to_string());
                waits = crate::inspect::tree_waits(sim, k);
                // the rule's own chance, as `tech::discovery` reckons it: a draw a year at
                // rate p^4 / (1 + p^4), p the least of made over needed, which only rises as
                // the island makes more; so the mean wait at today's chance is the longest
                // the mean wait can be, and the year it gives is the latest it is likely by
                let req: Vec<usize> = (0..C.G).filter(|&g| st.disc_X0[(k, g)] > 0.0).collect();
                let sought = crate::woods::sought(C, P, k) && (!crate::tasks::idea_of_use(C, k) || crate::tasks::idea_sought(C, P, k)) && (!crate::care::is_idea(C, k) || crate::care::sought(P));
                let after = C.tech_after.get(k).copied().unwrap_or(0);
                if pin.is_none() && sought && now >= after && !req.is_empty() {
                    let p = req.iter().map(|&g| st.cumprod[g] / st.disc_X0[(k, g)]).fold(f64::INFINITY, f64::min).max(0.0);
                    let p4 = p.powf(4.0);
                    let hz = P.discovery_rate * p4 / (1.0 + p4);
                    if hz >= 0.02 {
                        let wait = (1.0 / hz).round() as i64;
                        let by_year = now - 1 + wait.max(1);
                        likely = Some(by_year);
                        let past = if by_year > end { format!(", after the run's last year, {end}") } else { String::new() };
                        likely_words = format!(
                            "At today's chance of {} a year, the mean wait is {wait} year{}: found on the mean by {by_year}{past}, and sooner as the island makes more.",
                            fmt_pct(hz, 1),
                            if wait == 1 { "" } else { "s" }
                        );
                    }
                }
                drag = if !sought {
                    Err(format!("{} is not sought under these rules (its rule or switch is off), so a pin would do nothing.", C.tech_label[k]))
                } else {
                    let t = edits::pinned_town(st, C, k);
                    Ok(Drag { first: now, hold_from: now + 1, now_town: t.map(|t| (t, st.town_name[t].clone())) })
                };
            }
        }
        if !learnt_v.is_empty() {
            let n = learnt_v.len();
            let every_words = spread_to.map_or(String::new(), |y| format!(" Every town standing knew it by {y}."));
            said.push(format!("Known in {n} of the {} towns standing: {}.{every_words}", active.len(), towns_words(&learnt_v, 6)));
        }
        said.push(history(target));
        if !waits.is_empty() {
            said.push(waits.clone());
        }
        if !likely_words.is_empty() {
            said.push(likely_words.clone());
        }
        if pin.is_some() {
            said.push(pin_words(pin));
        }
        said.push(match &drag {
            Ok(_) => "Drag its mark along the row to a year: to find it then, or to hold it back until then.".to_string(),
            Err(why) => why.clone(),
        });
        let handle = handle_of(pin, likely, target, now, end);
        let row = IdeaRow {
            name: C.tech_name[k].clone(),
            label: C.tech_label[k].clone(),
            kind,
            target,
            found,
            at_start,
            town,
            how,
            by_hand,
            learnt: learnt_v,
            spread_to,
            pin,
            waits,
            likely,
            likely_words,
            handle,
            said,
            drag,
        };
        tree.push((if method { families::Group::of_method(C, k) } else { None }, k, row));
    }
    // each group in the atlas's order: history's year, then the year found, then the data's
    let order = |mut v: Vec<(usize, IdeaRow)>| -> Vec<IdeaRow> {
        v.sort_by_key(|(k, r)| (r.target.or(r.found).unwrap_or(end + 1), *k));
        v.into_iter().map(|(_, r)| r).collect()
    };
    let take = |keep: &dyn Fn(Option<families::Group>, usize) -> bool| tree.iter().filter(|x| keep(x.0, x.1)).map(|x| (x.1, x.2.clone())).collect::<Vec<_>>();
    let mut groups = Vec::new();
    let mut push = |name: &str, rows: Vec<IdeaRow>| {
        if !rows.is_empty() {
            groups.push(IdeaGroup { name: name.to_string(), rows });
        }
    };
    for g in families::Group::ALL {
        push(g.label(), order(take(&|grp, k| k < C.M && grp == Some(g))));
    }
    push("Other works", order(take(&|grp, k| k < C.M && grp.is_none())));
    push("Ways of working", order(take(&|_, k| k >= C.M)));

    // the uses of the tasks rule, found by the island as a whole
    let people: f64 = st.pop.iter().sum::<f64>().max(1.0);
    let mut uses = Vec::new();
    for (u, us) in C.tasks.uses.iter().enumerate() {
        let found = st.tasks.found.get(u).copied().filter(|&y| y >= 0);
        let label = edits::use_label(&us.name);
        let class = crate::tasks::CLASSES.get(us.class).copied().unwrap_or("some");
        let mut on: Vec<String> = Vec::new();
        if us.on_works {
            on.push("in the works".to_string());
        }
        let land: Vec<String> = us.on_sector.iter().enumerate().filter(|&(_, &b)| b).filter_map(|(s, _)| C.sector_label.get(s).map(|l| lower(l))).collect();
        if !land.is_empty() {
            on.push(format!("on the land ({})", and_list(&land)));
        }
        let place = if on.is_empty() { String::new() } else { format!(", {}", on.join(" and ")) };
        let pin = pin_of(Idea::Use(u));
        let mut said = vec![format!("{label} (a machine for tasks)"), format!("Machines that take over tasks of the {class} kind{place}. A use of the tasks rule is found by the island as a whole, not town by town.")];
        let (mut waits, mut likely, mut likely_words) = (String::new(), None, String::new());
        let next = now + 1;
        let drag = match found {
            Some(y) => {
                said.push(format!("Found by the island in {y}."));
                Err(format!("{label} was found in {y}: a find cannot be unmade but by taking it back, and only while it is the last edit by hand."))
            }
            None => {
                said.push("Not yet found.".to_string());
                let made = |g: usize| if us.discover_per_head { st.cumprod[g] / people } else { st.cumprod[g] };
                let mut w: Vec<String> = Vec::new();
                let goods: Vec<String> = us
                    .discover
                    .iter()
                    .map(|&(g, x)| {
                        let each = if us.discover_per_head { " a head" } else { "" };
                        let met = if made(g) >= x { ", met" } else { "" };
                        format!("{} made since {start}, {}{each} of the {}{each} it needs{met}", lower(&C.good_label[g]), crate::inspect::qty(made(g)), crate::inspect::qty(x))
                    })
                    .collect();
                if !goods.is_empty() {
                    w.push(and_list(&goods));
                }
                let idea_waits = us.idea.filter(|&t| st.tech_year.get(t).is_none_or(|&y| y < 0));
                if let Some(t) = idea_waits {
                    w.push(format!("{}, an idea of the tree not yet found", C.tech_label[t]));
                }
                let needs: Vec<String> = us.needs.iter().filter(|&&v| st.tasks.found.get(v).is_none_or(|&y| y < 0)).map(|&v| lower(&edits::use_label(&C.tasks.uses[v].name))).collect();
                if !needs.is_empty() {
                    w.push(format!("the {} first", and_list(&needs)));
                }
                if next < us.found_after {
                    w.push(format!("the year {}, before which it is not found", us.found_after));
                }
                let allowed = P.tasks.enabled && us.allowed(P);
                if !P.tasks.enabled {
                    w.push("the tasks rule, which is off, so that it is never found".to_string());
                } else if let Some(s) = us.switch.as_ref().filter(|_| !us.allowed(P)) {
                    w.push(format!("the switch tasks.{s}, which is off, so that it is never found"));
                }
                waits = format!("It waits on {}; then it is found at the next year's turn, with no draw.", if w.is_empty() { "nothing more".to_string() } else { w.join("; ") });
                // with no draw the rule gives its year where only the date is still to come
                let goods_met = us.discover.iter().all(|&(g, x)| made(g) >= x);
                if pin.is_none() && allowed && goods_met && idea_waits.is_none() && needs.is_empty() {
                    let y = next.max(us.found_after);
                    likely = Some(y);
                    likely_words = format!("Every amount it waits on is met: the rule finds it at the turn into {y}.");
                }
                if let Some(t) = us.idea.filter(|&t| C.tech_name.get(t) == Some(&us.name)) {
                    // (a pin names an idea, and this name is the tree's idea's first)
                    Err(format!("{label} is found at the year's turn after {}, the idea of the tree of the same name, which has a row of its own: drag that one.", C.tech_label[t]))
                } else if allowed {
                    Ok(Drag { first: next, hold_from: next + 1, now_town: None })
                } else {
                    Err(format!("{label} stands behind a rule or switch that is off, so a pin would do nothing."))
                }
            }
        };
        said.push(history(us.target));
        if !waits.is_empty() {
            said.push(waits.clone());
        }
        if !likely_words.is_empty() {
            said.push(likely_words.clone());
        }
        if pin.is_some() {
            said.push(pin_words(pin));
        }
        said.push(match &drag {
            Ok(_) => "Drag its mark along the row to a year: to find it then, or to hold it back until then.".to_string(),
            Err(why) => why.clone(),
        });
        let handle = handle_of(pin, likely, us.target, if found.is_some() { now } else { next }, end);
        let row = IdeaRow {
            name: us.name.clone(),
            label,
            kind: "use",
            target: us.target,
            found,
            at_start: false,
            town: None,
            how: if found.is_some() { "by the island as a whole".to_string() } else { "not yet found".to_string() },
            by_hand: false,
            learnt: Vec::new(),
            spread_to: None,
            pin,
            waits,
            likely,
            likely_words,
            handle,
            said,
            drag,
        };
        uses.push((u, row));
    }
    push("Machines for tasks", order(uses));
    IslandIdeas { from: start, now, end, towns: active.len(), groups }
}

/// Where a row's mark stands to be dragged: the pin's year, else the year it
/// is likely by, else history's year if still to come, else `now`.
fn handle_of(pin: Option<(i64, PinWhen)>, likely: Option<i64>, target: Option<i64>, now: i64, end: i64) -> i64 {
    pin.map(|p| p.0).or(likely).or(target.filter(|&y| y > now)).unwrap_or(now).max(now).min(end)
}

/// What a row's mark dropped on `year` offers, the likelier first: for an idea
/// of the tree dropped on this year or before it, found now in the town a pin
/// would choose; for a use, found at the next turn; dropped on a year still to
/// come, found in that year, or held back until it (a hold only where it
/// holds anything back). The chip that would leave the pin as it stands is
/// not offered: nothing is offered that does nothing.
pub fn drop_choices(row: &IdeaRow, year: i64, end: i64) -> Result<Vec<Offer>, String> {
    let d = row.drag.as_ref().map_err(|why| why.clone())?;
    let year = year.min(end);
    let label = &row.label;
    let mut out = Vec::new();
    let pin = |y: i64, when: PinWhen| Edit::Pin { idea: row.name.clone(), year: y, when };
    if year <= d.first {
        match (row.kind, &d.now_town) {
            ("use", _) => out.push(Offer {
                label: format!("Find it at the turn into {}", d.first),
                edit: pin(d.first, PinWhen::In),
                line: format!("{label} is found by the island at the turn into {}, whatever it has made: a use is found at a year's turn, so this is the soonest.", d.first),
            }),
            (_, Some((t, town))) => out.push(Offer {
                label: "Find it now".to_string(),
                edit: Edit::Discover { tech: row.name.clone(), town: *t },
                line: format!("{label} is found in {town} now, by hand: the town that handled most of what it is found on last year. A quarter of the town knows it at once, and it spreads from there."),
            }),
            (_, None) => return Err(format!("{label} cannot be found now: no town stands to find it in.")),
        }
    } else {
        let find = Offer {
            label: format!("Find it in {year}"),
            edit: pin(year, PinWhen::In),
            line: format!("The island will not find {label} before {year}, and finds it in {year} whatever it has made{}.", if row.kind == "use" { "" } else { ", in the town that handles most of what it is found on" }),
        };
        let hold = Offer {
            label: format!("Hold it back until {year}"),
            edit: pin(year, PinWhen::NotBefore),
            line: format!("The island will not find {label} before {year}; from then it finds it as it would have, by its own rule."),
        };
        let later = year > row.handle;
        let mut two = if later { vec![hold, find] } else { vec![find, hold] };
        two.retain(|o| match &o.edit {
            Edit::Pin { year: y, when, .. } => row.pin != Some((*y, *when)) && (*when == PinWhen::In || *y >= d.hold_from),
            _ => true,
        });
        out.extend(two);
    }
    if out.is_empty() {
        return Err(format!("{label} is pinned there already: nothing would change."));
    }
    Ok(out)
}
