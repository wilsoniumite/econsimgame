//! The month's ledger of buyers (`econ::step_month`): one list of what every
//! buyer asks of its town's market, filled once, from which each rule is handed
//! what its lines got, the market's totals are added, and the books read.
//!
//! Before 27 November 2026 each of eleven kinds of buyer was written out by
//! hand at five steps of the month: what it asked, what it got at the fill,
//! what it was handed, what it paid, and what the books saw. A new buyer meant
//! five more places. Now a buyer writes one line (or two, where it buys both
//! with the day's trade and for its plant), and the rest is done here once.
//!
//! A line says, row by row, the town whose market is asked and who pays (a
//! town's households, a works, a unit of the land, a town and sector, a
//! railway or canal), and what of each good is asked; and whether it is the
//! day's trade or an order for plant, which under the tree's "plant waits"
//! (`docs/rules/tree.md`) is served second. Its got is its ask times its
//! market's fill for that trade, but for the works' inputs, which a works uses
//! as its scarcest input allows (`econ::step_month`, "execute"), and which its
//! own rule writes back.
//!
//! The order in which sums are added is part of the result, so it is written
//! down here and nowhere else. What was asked is added in [`ASK_ORDER`]; what
//! was sold in [`SALE_ORDER`], another order, the port's own: the port adds
//! the works' orders for capacity to its sales before the land's machinery,
//! and to its asks after it. A buyer with two lines (the land's machines for
//! tasks, the carters' lorries) has their gots added into one before they are
//! sold, paid for or booked, as they always were. How a line goes into its
//! town's total matters to the bit too: summed by town first and the town's
//! sum added ([`Fold::ByTown`], numpy's `bincount`), or row by row
//! ([`Fold::EachRow`]). All of this was kept to the bit when the ledger came
//! in, and the port's and the game's golden tests held.
//!
//! Of the four orders for plant, only the works' orders for capacity are
//! paced (`pace::share`, docs/rules/pace.md); the machines for tasks' three
//! (the works', the land's and the lorries') ask the port's quarter a month
//! (`pace::PORT_SHARE`). Pace should reach all four, since a builder's pace is
//! the builder's whoever orders; it would give every island another history
//! after 1901, so it waits for the first century to be laid out anew.
//!
//! The kinds of buyer, and where each line is written in `econ::step_month`:
//!
//! | Buyer | Day's trade | Plant | Rows |
//! |---|---|---|---|
//! | households | what they buy | | a town |
//! | landlords (and families, through them) | what they buy | | a town |
//! | works | inputs and fuel, their machines' run goods among them | | a works at work |
//! | the land's machinery and rails | two goods | | a unit of the land |
//! | the land's chemicals | nitrogen's and breeding's goods | | a town and sector |
//! | the land's machines for tasks | run goods | plant | a town and sector |
//! | the carters' lorries | oil | plant | a town |
//! | the works' machines for tasks | (in the works' inputs) | plant | a works |
//! | the works' orders for capacity | | plant | a works |
//! | railways and canals | | materials | a project |
//! | the university's rooms | | building | a town |

use std::borrow::Cow;

use crate::content::Content;
use crate::mat::Mat;

/// Who buys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Buyer {
    Households,
    Landlords,
    Works,
    LandMachinery,
    LandChemicals,
    LandTasks,
    Lorries,
    WorksTasks,
    WorksOrders,
    Ways,
    University,
}

/// The day's trade, served first under "plant waits", or an order for plant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trade {
    Day,
    Plant,
}

/// How a line goes into its town's total: its rows summed by town first and
/// each town's sum added, or each row added in turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fold {
    ByTown,
    EachRow,
}

/// A line of the ledger is a buyer's asks for one trade.
pub type Kind = (Buyer, Trade);

/// The lines in the order their asks are added into what each market was
/// asked. The day's trade comes first, since "plant waits" serves it first
/// and reads what it alone asked.
pub const ASK_ORDER: [Kind; 13] = [
    (Buyer::Households, Trade::Day),
    (Buyer::Landlords, Trade::Day),
    (Buyer::Works, Trade::Day),
    (Buyer::LandMachinery, Trade::Day),
    (Buyer::LandChemicals, Trade::Day),
    (Buyer::LandTasks, Trade::Day),
    (Buyer::Lorries, Trade::Day),
    (Buyer::WorksOrders, Trade::Plant),
    (Buyer::WorksTasks, Trade::Plant),
    (Buyer::LandTasks, Trade::Plant),
    (Buyer::Lorries, Trade::Plant),
    (Buyer::Ways, Trade::Plant),
    (Buyer::University, Trade::Plant),
];

/// The order in which what each line got is added into what each market sold.
/// The lines of one group have the same rows, and their gots are added into
/// one before it goes into the total.
pub const SALE_ORDER: [&[Kind]; 11] = [
    &[(Buyer::Households, Trade::Day)],
    &[(Buyer::Landlords, Trade::Day)],
    &[(Buyer::Works, Trade::Day)],
    &[(Buyer::WorksOrders, Trade::Plant)],
    &[(Buyer::WorksTasks, Trade::Plant)],
    &[(Buyer::LandMachinery, Trade::Day)],
    &[(Buyer::LandChemicals, Trade::Day)],
    &[(Buyer::LandTasks, Trade::Day), (Buyer::LandTasks, Trade::Plant)],
    &[(Buyer::Lorries, Trade::Day), (Buyer::Lorries, Trade::Plant)],
    &[(Buyer::Ways, Trade::Plant)],
    &[(Buyer::University, Trade::Plant)],
];

fn place(kind: Kind) -> usize {
    ASK_ORDER.iter().position(|&k| k == kind).expect("ledger: a kind of line not in ASK_ORDER")
}

/// One buyer's asks for one trade: a row for each who pays.
pub struct Line {
    /// The town whose market each row asks.
    pub town: Vec<usize>,
    /// Who pays for each row: a town, a works, a unit of the land, a town and
    /// sector (`t * NS + s`) or a project, as the buyer is.
    pub who: Vec<usize>,
    /// The good of each column, or every good in order where `None`.
    pub goods: Option<Vec<usize>>,
    pub fold: Fold,
    /// (rows, columns): what is asked.
    pub ask: Mat,
    /// The same shape: what was got. Set by the fill, or by the buyer's own
    /// rule where `by_rule` (the works' inputs).
    pub got: Mat,
    by_rule: bool,
}

impl Line {
    /// A line over every good.
    pub fn new(town: Vec<usize>, who: Vec<usize>, fold: Fold, ask: Mat) -> Line {
        Line { town, who, goods: None, fold, ask, got: Mat::default(), by_rule: false }
    }

    /// A line over the goods named, a column each.
    pub fn of_goods(goods: Vec<usize>, town: Vec<usize>, who: Vec<usize>, fold: Fold, ask: Mat) -> Line {
        Line { town, who, goods: Some(goods), fold, ask, got: Mat::default(), by_rule: false }
    }

    /// A line whose got its own rule writes (`Ledger::set_got`).
    pub fn by_rule(mut self) -> Line {
        self.by_rule = true;
        self
    }

    /// The good of column `j`.
    pub fn good(&self, j: usize) -> usize {
        match &self.goods {
            Some(g) => g[j],
            None => j,
        }
    }
}

/// Adds `m`, a line's asks or gots, into `total` (towns by goods) as the line folds.
fn fold_into(total: &mut Mat, line: &Line, m: &Mat) {
    match (line.fold, &line.goods) {
        (Fold::EachRow, None) => {
            for r in 0..m.rows {
                let t = line.town[r];
                for (x, y) in total.row_mut(t).iter_mut().zip(m.row(r)) {
                    *x += y;
                }
            }
        }
        (Fold::EachRow, Some(goods)) => {
            for r in 0..m.rows {
                let t = line.town[r];
                for (j, &g) in goods.iter().enumerate() {
                    total[(t, g)] += m[(r, j)];
                }
            }
        }
        (Fold::ByTown, _) => {
            let mut acc = Mat::zeros(total.rows, m.cols);
            for r in 0..m.rows {
                let t = line.town[r];
                for (x, y) in acc.row_mut(t).iter_mut().zip(m.row(r)) {
                    *x += y;
                }
            }
            match &line.goods {
                None => {
                    for (x, y) in total.data.iter_mut().zip(&acc.data) {
                        *x += y;
                    }
                }
                Some(goods) => {
                    for (j, &g) in goods.iter().enumerate() {
                        for t in 0..total.rows {
                            total[(t, g)] += acc[(t, j)];
                        }
                    }
                }
            }
        }
    }
}

/// The month's ledger.
pub struct Ledger {
    lines: Vec<Option<Line>>,
    /// Towns by goods: all that was asked of each market (once `add_asks` has run).
    pub asked: Mat,
    /// The share of what was asked that each market could give, for the day's
    /// trade and for plant (once `fill` has run; one number without the tree's
    /// "plant waits").
    pub fill: Mat,
    pub fill_plant: Mat,
    /// All that each market sold (once `sell` has run).
    pub sold: Mat,
}

impl Ledger {
    pub fn new(T: usize, G: usize) -> Ledger {
        Ledger {
            lines: (0..ASK_ORDER.len()).map(|_| None).collect(),
            asked: Mat::zeros(T, G),
            fill: Mat::default(),
            fill_plant: Mat::default(),
            sold: Mat::zeros(T, G),
        }
    }

    /// A buyer's line for one trade. A kind has one line a month.
    pub fn ask(&mut self, kind: Kind, line: Line) {
        let k = place(kind);
        debug_assert!(self.lines[k].is_none(), "ledger: {kind:?} asked twice");
        debug_assert_eq!(line.town.len(), line.ask.rows);
        debug_assert_eq!(line.who.len(), line.ask.rows);
        self.lines[k] = Some(line);
    }

    pub fn line(&self, kind: Kind) -> Option<&Line> {
        self.lines[place(kind)].as_ref()
    }

    pub fn got(&self, kind: Kind) -> Option<&Mat> {
        self.line(kind).map(|l| &l.got)
    }

    /// Every line's asks into `asked`, in `ASK_ORDER`. With `day_apart`, also
    /// what the day's trade alone asked, for "plant waits".
    pub fn add_asks(&mut self, day_apart: bool) -> Option<Mat> {
        let mut day = None;
        for (k, kind) in ASK_ORDER.iter().enumerate() {
            if day_apart && day.is_none() && kind.1 == Trade::Plant {
                day = Some(self.asked.clone());
            }
            if let Some(l) = &self.lines[k] {
                fold_into(&mut self.asked, l, &l.ask);
            }
        }
        if day_apart && day.is_none() {
            day = Some(self.asked.clone());
        }
        day
    }

    /// The fill of each market for the day's trade and for plant, and each
    /// line's got at its trade's fill (but for the lines its own rule fills).
    pub fn fill(&mut self, fill: Mat, fill_plant: Mat) {
        for (k, kind) in ASK_ORDER.iter().enumerate() {
            let Some(l) = &mut self.lines[k] else { continue };
            if l.by_rule {
                continue;
            }
            let f = if kind.1 == Trade::Day { &fill } else { &fill_plant };
            let mut got = Mat::zeros(l.ask.rows, l.ask.cols);
            for r in 0..l.ask.rows {
                let fr = f.row(l.town[r]);
                match &l.goods {
                    None => {
                        for ((x, &a), &b) in got.row_mut(r).iter_mut().zip(l.ask.row(r)).zip(fr) {
                            *x = a * b;
                        }
                    }
                    Some(goods) => {
                        for (j, &g) in goods.iter().enumerate() {
                            got[(r, j)] = l.ask[(r, j)] * fr[g];
                        }
                    }
                }
            }
            l.got = got;
        }
        self.fill = fill;
        self.fill_plant = fill_plant;
    }

    /// What a line that its own rule fills got.
    pub fn set_got(&mut self, kind: Kind, got: Mat) {
        let l = self.lines[place(kind)].as_mut().expect("ledger: set_got on a line not asked");
        debug_assert!(l.by_rule && got.rows == l.ask.rows && got.cols == l.ask.cols);
        l.got = got;
    }

    /// What every line got into `sold`, in `SALE_ORDER`.
    pub fn sell(&mut self) {
        let mut sold = std::mem::take(&mut self.sold);
        for group in SALE_ORDER {
            if let Some((l, got)) = self.together(group) {
                fold_into(&mut sold, l, &got);
            }
        }
        self.sold = sold;
    }

    /// What the lines of `kinds` got, added into one in that order, with the
    /// first present line (for its rows); None if none was asked. The lines
    /// must have the same rows.
    pub fn together(&self, kinds: &[Kind]) -> Option<(&Line, Cow<'_, Mat>)> {
        let mut out: Option<(&Line, Cow<'_, Mat>)> = None;
        for &kind in kinds {
            let Some(l) = self.line(kind) else { continue };
            out = Some(match out {
                None => (l, Cow::Borrowed(&l.got)),
                Some((first, acc)) => {
                    let mut acc = acc.into_owned();
                    for (x, y) in acc.data.iter_mut().zip(&l.got.data) {
                        *x += y;
                    }
                    (first, Cow::Owned(acc))
                }
            });
        }
        out
    }

    /// What each row of the lines of `kinds` (got together) is worth at its
    /// town's prices; empty if none was asked. For lines over every good.
    pub fn paid(&self, C: &Content, price: &Mat, kinds: &[Kind]) -> Vec<f64> {
        let Some((l, got)) = self.together(kinds) else { return Vec::new() };
        (0..got.rows).map(|r| C.sum_goods(|g| got[(r, g)] * price[(l.town[r], g)])).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_day_s_trade_is_asked_first() {
        let first_plant = ASK_ORDER.iter().position(|k| k.1 == Trade::Plant).unwrap();
        assert!(ASK_ORDER[first_plant..].iter().all(|k| k.1 == Trade::Plant));
    }

    #[test]
    fn every_line_is_sold_once() {
        for kind in ASK_ORDER {
            let n: usize = SALE_ORDER.iter().map(|g| g.iter().filter(|&&k| k == kind).count()).sum();
            assert_eq!(n, 1, "{kind:?}");
        }
    }
}
