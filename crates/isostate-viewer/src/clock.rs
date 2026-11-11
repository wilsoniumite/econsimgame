//! The calendar on the clock: months, the named speeds, and the arithmetic
//! between the clock's number and a date. Plain functions, so they can be
//! tested without a window.
//!
//! The clock (`Timeline::t`) counts years. Its whole part is the year's end
//! whose picture is on the map: 0 is the end of the first year, since a game
//! opens with its first year played and the island is redrawn once a year.
//! The part after the point is how far the year after that has gone. So a
//! clock at 33.21 in a game begun in 1750 shows the island as 1783 left it,
//! and stands in March 1784: `date` says so.
//!
//! The simulation has months and nothing shorter. How far through its month
//! the clock stands (`Date::part`) is the clock's own, shown as a small bar
//! and never named as a week or a day.

pub const MONTHS: [&str; 12] = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// One of the clock's speeds.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Speed {
    /// what its button says
    pub name: &'static str,
    /// years a second
    pub rate: f32,
    /// what goes by at it, for the line under the date
    pub pace: &'static str,
}

/// The clock's speeds, slowest first: a few named steps, never a slider. The
/// slowest lets a month be read as it goes by; the fastest still crosses the
/// three centuries to 2050 in a few minutes (`crossing` says how long each takes).
pub const SPEEDS: [Speed; 5] = [
    Speed { name: "Slow", rate: 1.0 / 24.0, pace: "a month every two seconds" },
    Speed { name: "Steady", rate: 1.0 / 12.0, pace: "a month a second" },
    Speed { name: "Brisk", rate: 0.25, pace: "a season a second" },
    Speed { name: "Fast", rate: 1.0, pace: "a year a second" },
    Speed { name: "Fastest", rate: 2.0, pace: "two years a second" },
];

/// The speed a game starts at: a month a second.
pub const START_SPEED: f32 = SPEEDS[1].rate;

/// The button a speed belongs to, if it is one of theirs.
pub fn speed_at(rate: f32) -> Option<&'static Speed> {
    SPEEDS.iter().find(|s| (s.rate - rate).abs() < 1e-4)
}

/// The next of the buttons' speeds above or below `rate`, for the Up and Down
/// keys: the keys land on a button, so one of them is always lit.
pub fn step_speed(rate: f32, faster: bool) -> f32 {
    let (first, last) = (SPEEDS[0].rate, SPEEDS[SPEEDS.len() - 1].rate);
    if faster {
        SPEEDS.iter().map(|s| s.rate).find(|&v| v > rate + 1e-4).unwrap_or(last)
    } else {
        SPEEDS.iter().rev().map(|s| s.rate).find(|&v| v < rate - 1e-4).unwrap_or(first)
    }
}

/// How long the clock takes over `years` at a speed, in seconds, if the
/// simulation keeps up and nothing stops it.
pub fn crossing(years: usize, rate: f32) -> f32 {
    years as f32 / rate
}

/// A length of time in the largest two units that matter: "2 hours", "1 hour
/// 15 minutes", "2 minutes 30 seconds", "75 seconds".
pub fn length_words(secs: f32) -> String {
    let s = secs.round() as u64;
    let unit = |n: u64, word: &str| format!("{n} {word}{}", if n == 1 { "" } else { "s" });
    let two = |a: u64, aw: &str, b: u64, bw: &str| if b == 0 { unit(a, aw) } else { format!("{} {}", unit(a, aw), unit(b, bw)) };
    if s >= 3600 {
        two(s / 3600, "hour", (s % 3600 + 30) / 60, "minute")
    } else if s >= 120 {
        two(s / 60, "minute", s % 60, "second")
    } else {
        unit(s, "second")
    }
}

/// Twelfths are not whole in binary, so a clock set to the start of a month
/// may stand a hair before it: this much of a month is forgiven.
const SLACK: f32 = 0.01;

/// The month the clock stands in, 0 for January, and how far through it, 0 to 1.
pub fn month_of(t: f32) -> (usize, f32) {
    let m = (t - t.floor()) * 12.0;
    let month = ((m + SLACK).floor() as usize).min(11);
    (month, (m - month as f32).clamp(0.0, 1.0))
}

/// The clock at the start of month `month` (0 to 12; 12 is the next year's turn) after year's end `k`.
pub fn at(k: usize, month: usize) -> f32 {
    k as f32 + month as f32 / 12.0
}

/// Whether the clock stands exactly on a year's turn.
pub fn on_turn(t: f32) -> bool {
    (t - t.round()).abs() * 12.0 < SLACK
}

/// The start of the month after the one the clock stands in.
pub fn next_month(t: f32) -> f32 {
    let (m, _) = month_of(t);
    at(t.floor() as usize, m + 1)
}

/// The month's place in the run's monthly series (0 is the first January):
/// the month the clock stands in, of the year after the year's end on the map.
pub fn series_month(t: f32) -> usize {
    (t.floor() as usize + 1) * 12 + month_of(t).0
}

/// A date on the calendar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Date {
    pub year: i32,
    /// 0 for January
    pub month: usize,
    /// how far through the month the clock stands, 0 to 1
    pub part: f32,
}

/// The date a clock at `t` stands on, in a run whose first year is `y0`.
pub fn date(y0: i32, t: f32) -> Date {
    let (month, part) = month_of(t);
    Date { year: y0 + t.floor() as i32 + 1, month, part }
}

/// "March 1784".
pub fn month_year(d: Date) -> String {
    format!("{} {}", MONTHS[d.month.min(11)], d.year)
}

/// The clock's time as the news is dated: years, with the part of the year after the point.
pub fn news_time(y0: i32, t: f32) -> f32 {
    y0 as f32 + t + 1.0
}

/// The month a piece of news is dated in: 1784.46 is "June 1784".
pub fn news_month(year: f32) -> String {
    let m = (((year - year.floor()) * 12.0).floor() as usize).min(11);
    format!("{} {}", MONTHS[m], year.floor() as i32)
}

/// How many months of the year in progress should have been asked of the
/// simulation by now: every month whose end the clock will reach within the
/// time the work takes. `into` is how far into the year the clock stands, in
/// years; `lead` how long before a month's end to ask for it and `lead_turn`
/// how long before the year's end to ask for December, which brings the
/// year's end with it and takes far longer. Never more than the twelve.
pub fn months_due(into: f32, lead: f32, lead_turn: f32) -> usize {
    if into < 0.0 {
        return 0;
    }
    let by_month = (((into + lead) * 12.0 + SLACK).floor() as usize).min(11);
    if into + lead_turn >= 1.0 - SLACK / 12.0 { 12 } else { by_month }
}

/// How long a month takes to work out, and a year's end with its December,
/// in seconds, before either has been timed: on the long side for a debug
/// build, so the first year of a game does not keep the clock waiting.
pub const MONTH_GUESS: f32 = 0.06;
pub const TURN_GUESS: f32 = 0.8;

/// How long before it is due to ask for a piece of the simulation's work, as
/// a part of a year: half as long again as the last such piece took (`took`,
/// in seconds; `guess` until one has been timed), and a tenth of a second for
/// the frames either side.
pub fn lead_years(took: Option<f32>, guess: f32, rate: f32) -> f32 {
    ((took.unwrap_or(guess) * 1.5 + 0.1) * rate).clamp(0.0, 1.0)
}

/// The furthest the clock may run with `months` months of the year after
/// year's end `latest` worked out: into the first month still to come, and
/// not past its end, so a month is never shown as over before it is.
pub fn frontier(latest: usize, months: usize) -> f32 {
    at(latest, (months + 1).min(12)) - 0.001
}

/// A year takes at least this many seconds for the seasons to go by the date:
/// at the two slowest speeds they do, and above them they keep their own calm round.
pub const FOLLOW_YEAR_SECS: f32 = 12.0;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_clock_reads_as_a_month_and_a_year() {
        // the game opens with 1750 played: the clock at nought is the turn into 1751
        assert_eq!(month_year(date(1750, 0.0)), "January 1751");
        assert_eq!(month_year(date(1750, 33.21)), "March 1784");
        assert_eq!(month_year(date(1750, 33.999)), "December 1784");
        assert_eq!(month_year(date(1750, 34.0)), "January 1785");
        // every month of a late year, set from its number, reads back as itself
        for k in [0usize, 33, 149, 299] {
            for m in 0..12 {
                let d = date(1750, at(k, m));
                assert_eq!((d.year, d.month), (1750 + k as i32 + 1, m), "year's end {k}, month {m}");
                assert!(d.part < 0.02, "the start of a month is its start, got {}", d.part);
                // and the middle and the last frame of the month are still that month
                assert_eq!(date(1750, at(k, m) + 0.5 / 12.0).month, m);
                assert_eq!(date(1750, at(k, m) + 0.98 / 12.0).month, m);
            }
        }
        let d = date(1750, 33.0 + 2.5 / 12.0);
        assert!((d.part - 0.5).abs() < 0.01, "half way through March, got {}", d.part);
    }

    #[test]
    fn next_goes_to_the_start_of_the_following_month_and_the_twelfth_is_the_turn() {
        let mut t = 33.0;
        for m in 1..=12 {
            t = next_month(t);
            assert!((t - (33.0 + m as f32 / 12.0)).abs() < 1e-4, "step {m}: {t}");
        }
        assert!(on_turn(t) && t.round() == 34.0);
        // from the middle of a month, the start of the next
        assert!((next_month(33.21) - (33.0 + 3.0 / 12.0)).abs() < 1e-4);
        assert!(on_turn(12.0) && on_turn(at(12, 12)) && !on_turn(12.05) && !on_turn(at(12, 11)));
    }

    #[test]
    fn the_series_month_is_the_month_on_the_clock() {
        // year's end 0 is 1750's: the months after it are the second year's
        assert_eq!(series_month(0.0), 12);
        assert_eq!(series_month(at(0, 11) + 0.01), 23);
        assert_eq!(series_month(at(33, 2)), 34 * 12 + 2);
    }

    #[test]
    fn news_is_dated_by_its_month() {
        assert_eq!(news_month(1784.46), "June 1784");
        assert_eq!(news_month(1784.0), "January 1784");
        assert_eq!(news_month(1784.95), "December 1784");
        // the clock in March 1784 has passed the news of February and not that of April
        let now = news_time(1750, at(33, 2) + 0.5 / 12.0);
        assert!(now > 1784.0 + 1.5 / 12.0 && now < 1784.0 + 3.5 / 12.0, "{now}");
    }

    #[test]
    fn the_speeds_are_a_few_named_steps_from_a_readable_month_to_decades_a_minute() {
        assert!(SPEEDS.windows(2).all(|w| w[0].rate < w[1].rate));
        // the slowest: a month in two seconds; the top: a year or more a second
        assert!((1.0 / (SPEEDS[0].rate * 12.0) - 2.0).abs() < 1e-4);
        assert!(SPEEDS[SPEEDS.len() - 1].rate >= 1.0);
        assert_eq!(speed_at(START_SPEED).map(|s| s.name), Some("Steady"));
        assert_eq!(speed_at(0.6), None);
        // the keys go from button to button and stop at the ends
        let rates: Vec<f32> = SPEEDS.iter().map(|s| s.rate).collect();
        for w in rates.windows(2) {
            assert_eq!(step_speed(w[0], true), w[1]);
            assert_eq!(step_speed(w[1], false), w[0]);
        }
        assert_eq!(step_speed(rates[4], true), rates[4]);
        assert_eq!(step_speed(rates[0], false), rates[0]);
        // from a speed that is no button's, onto the next one
        assert_eq!(step_speed(0.5, true), 1.0);
        assert_eq!(step_speed(0.5, false), 0.25);
    }

    #[test]
    fn how_long_the_centuries_take_at_each_speed() {
        // 1750 to 1900, and on to 2050
        let took: Vec<(String, String)> = SPEEDS.iter().map(|s| (length_words(crossing(150, s.rate)), length_words(crossing(300, s.rate)))).collect();
        let want = [("1 hour", "2 hours"), ("30 minutes", "1 hour"), ("10 minutes", "20 minutes"), ("2 minutes 30 seconds", "5 minutes"), ("75 seconds", "2 minutes 30 seconds")];
        for (i, (a, b)) in want.iter().enumerate() {
            assert_eq!((took[i].0.as_str(), took[i].1.as_str()), (*a, *b), "{}", SPEEDS[i].name);
        }
        assert_eq!(length_words(4500.0), "1 hour 15 minutes");
        assert_eq!(length_words(1.0), "1 second");
    }

    #[test]
    fn months_are_asked_for_late_but_in_time() {
        // nothing timed yet: a guess, and at a month a second still nothing is asked for at the turn
        assert_eq!(months_due(0.0, lead_years(None, MONTH_GUESS, START_SPEED), lead_years(None, TURN_GUESS, START_SPEED)), 0);
        // a month takes 0.02 s to work out and the year's end 0.3 s
        let (month, turn) = (Some(0.02), Some(0.3));
        for s in SPEEDS {
            let (lead, lead_turn) = (lead_years(month, MONTH_GUESS, s.rate), lead_years(turn, TURN_GUESS, s.rate));
            // at the turn itself nothing is asked for until January is nearly over, at the slow speeds
            if s.rate <= 0.25 {
                assert_eq!(months_due(0.0, lead, lead_turn), 0, "{}", s.name);
                assert_eq!(months_due(0.5 / 12.0, lead, lead_turn), 0, "{}", s.name);
            }
            // every month has been asked for by the time the clock reaches its end
            for m in 0..12 {
                let end = (m + 1) as f32 / 12.0;
                assert!(months_due(end - 0.0001, lead, lead_turn) >= m + 1, "{}: month {m} is not asked for by its end", s.name);
                // and not a whole month before it is due, while the work is quick
                if s.rate <= 0.25 && m < 11 {
                    assert!(months_due(end - 1.0 / 12.0, lead, lead_turn) <= m, "{}: month {m} is asked for a month early", s.name);
                }
            }
            // December, with the year's end, is asked for earlier than a month would be
            let early = 1.0 - lead_turn;
            assert_eq!(months_due(early + 0.0001, lead, lead_turn), 12, "{}", s.name);
        }
        // before the year in progress (the clock is in the past) nothing is due
        assert_eq!(months_due(-0.5, 0.1, 0.1), 0);
    }

    #[test]
    fn the_clock_stops_inside_the_first_month_still_to_come() {
        // nothing of the year worked out: January may be entered, February not
        assert!(frontier(33, 0) > at(33, 0) && frontier(33, 0) < at(33, 1));
        assert_eq!(date(1750, frontier(33, 0)).month, 0);
        // eleven months in: December, and not the turn
        assert_eq!(date(1750, frontier(33, 11)).month, 11);
        assert!(frontier(33, 11) < 34.0 && frontier(33, 12) < 34.0);
    }
}
