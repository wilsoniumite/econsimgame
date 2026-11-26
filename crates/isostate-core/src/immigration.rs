//! Settlers from abroad (not in the Python; `docs/rules/immigration.md`).
//!
//! The island trades with nobody, and until this rule nobody came to it: its
//! people grew by their own births alone, and left it only by ship in a hungry
//! year. With `enabled`, once a year (`peopling::growth`, after each town's
//! births and deaths), settlers come by ship to every town on the coast:
//!
//! - **The pull** of a port is how far the living there, `y` baskets a head
//!   (income over the cost of a bare living), stands above the living
//!   abroad: `clamp(y / abroad_now - 1, 0, 1)`, nothing where it is no better,
//!   full where it is twice as good. `abroad_now` is `abroad` until
//!   `abroad_from`, growing `abroad_growth` a year after.
//! - **The settlers** are the pull times `ships` a year shared among the ports
//!   by their people (the ships that call, whatever the colony's size), plus
//!   `rate` of the port's own people (those who come because kin wrote home),
//!   and never more than `max_share` of the port's people in a year (what it
//!   can house).
//! - **Their ages** are mostly young adults (`AGES`): they enter the ages'
//!   reckoning at those ages and not as births (`ages::year`).
//! - **People only.** They bring no money: the island's purses are those of
//!   the island, and the monthly check of money holds as it was.
//!
//! Off, nobody arrives and nothing here runs. It is off under the game's
//! rules, and comes with the landfall start (`landfall::landfall`); on the
//! settled island it can be thrown by its switch to try other growth and other
//! pyramids of ages.

use serde::{Deserialize, Serialize};

use crate::ages::GROUPS;
use crate::params::Params;
use crate::state::{Event, State};
use crate::util::{argmax, fmt_thousands_f0, pairwise_sum};

/// The settlers' ages, one in all, by five-year group (0 to 4, 5 to 9, ...):
/// two in three between 15 and 34, a fifth children who came with them, few
/// past 50. About the shape of those who landed in America in the nineteenth
/// century (from memory).
pub const AGES: [f64; GROUPS] = [0.09, 0.06, 0.05, 0.14, 0.22, 0.17, 0.10, 0.06, 0.04, 0.03, 0.02, 0.01, 0.01, 0.0, 0.0, 0.0, 0.0];

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ImmigrationParams {
    /// Settlers come by ship.
    pub enabled: bool,
    /// The living abroad in 1750, in baskets a head: 1.5, what the settled
    /// island lives on in 1750, as if the country the settlers leave were the
    /// island's own old country.
    pub abroad: f64,
    /// How fast the living abroad grows, a year, from `abroad_from`: 0.01,
    /// western Europe's income a head after 1820 in Maddison's tables, about a
    /// doubling every seventy years (from memory).
    pub abroad_growth: f64,
    /// The year the living abroad begins to grow: 1820. Before it, western
    /// Europe's income a head grew about 0.15 per cent a year (Maddison, from
    /// memory), which is taken as standing still.
    pub abroad_from: i64,
    /// Settlers a year at full pull whatever the colony's size: a ship or two a
    /// year of some two hundred souls.
    pub ships: f64,
    /// Settlers a year at full pull, as a share of a port's people: 0.015, the
    /// United States' immigration in its heaviest decades (the 1850s and the
    /// 1900s, about one in seventy-five a year; from memory).
    pub rate: f64,
    /// The most settlers a port takes in a year, as a share of its people.
    pub max_share: f64,
}

impl Default for ImmigrationParams {
    /// Off: nobody comes.
    fn default() -> ImmigrationParams {
        ImmigrationParams { enabled: false, abroad: 1.5, abroad_growth: 0.01, abroad_from: 1820, ships: 400.0, rate: 0.015, max_share: 0.05 }
    }
}

impl ImmigrationParams {
    /// On, with the numbers above (the landfall start's).
    pub fn game() -> ImmigrationParams {
        ImmigrationParams { enabled: true, ..ImmigrationParams::default() }
    }
}

/// What the rule keeps (`st.immigration`). Empty while it is off.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImmigrationState {
    /// Settlers who came to each town this year.
    pub came: Vec<f64>,
    /// Settlers who came in each year, for the island: (year, settlers).
    pub years: Vec<(i64, f64)>,
    /// All who have come.
    pub total: f64,
}

/// The living abroad in `year`, in baskets a head.
pub fn abroad(P: &Params, year: i64) -> f64 {
    let ip = &P.immigration;
    ip.abroad * (1.0 + ip.abroad_growth).powf((year - ip.abroad_from).max(0) as f64)
}

/// The year's settlers, town by town: added to each port's rate `g` (a share of
/// its people), handed to the ages' reckoning, and told in the news when they
/// are many. `y` is each town's baskets a head. Nothing while the rule is off.
pub fn year(st: &mut State, P: &Params, y: &[f64], year: i64, g: &mut [f64], events: &mut Vec<Event>) {
    if !P.immigration.enabled {
        return;
    }
    let ip = &P.immigration;
    let T = st.T_max;
    let ports: Vec<usize> = (0..T).filter(|&t| st.town_active[t] && st.world.coast[st.town_node[t]] && st.pop[t] > 0.0).collect();
    let port_people = pairwise_sum(&ports.iter().map(|&t| st.pop[t]).collect::<Vec<_>>());
    let there = abroad(P, year);
    let mut came = vec![0.0; T];
    for &t in &ports {
        let pull = (y.get(t).copied().unwrap_or(0.0) / there - 1.0).clamp(0.0, 1.0);
        let n = (pull * (ip.ships * st.pop[t] / port_people.max(1.0) + ip.rate * st.pop[t])).min(ip.max_share * st.pop[t]).max(0.0);
        came[t] = n;
        g[t] += n / st.pop[t];
    }
    let all = pairwise_sum(&came);
    st.immigration.came = came.clone();
    st.immigration.years.push((year, all));
    st.immigration.total += all;
    // ages (docs/rules/ages.md): they enter the reckoning at their ages
    crate::ages::came(st, P, &came, there);
    let island = pairwise_sum(&st.pop);
    // news when the settlers of a year are one in a hundred of the island, or the first ship comes
    let first = st.immigration.years.iter().filter(|&&(_, n)| n > 0.0).count() == 1 && all > 0.0;
    if all > 0.0 && (first || all > 0.01 * island) {
        let port = argmax(&came);
        let title = if first {
            format!("The first settlers' ship comes to {}", st.town_name[port])
        } else {
            format!("{} settlers come ashore", fmt_thousands_f0(all))
        };
        let text = format!(
            "{} settlers come by sea this year, most of them to {}, young men and women for the most part, with what children they have. They come with nothing but their hands: a living here is {:.1} baskets a head where it is {:.1} in the country they left.",
            fmt_thousands_f0(all),
            st.town_name[port],
            y.get(port).copied().unwrap_or(0.0),
            there
        );
        let mut ev = Event::new(year as f64 + 0.5, "settlers", 0.5, title, text);
        ev.town = Some(port);
        ev.cell = Some(st.town_node[port]);
        ev.why = crate::chronicle::why(
            P,
            "settlers come by sea",
            "settlers who came in the year",
            all,
            0.01 * island,
            Some(island),
            format!(
                "{} of {} people came from abroad (news at one in a hundred, or the first ship); a port draws them as far as its living stands above that abroad",
                fmt_thousands_f0(all),
                fmt_thousands_f0(island)
            ),
        );
        events.push(ev);
    }
}
