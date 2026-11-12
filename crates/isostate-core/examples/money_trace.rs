//! Prints how far the port's issue of money falls short of `money_growth`, and
//! where the money is that the issue does not see; and, for a run that fails
//! the monthly money check, whether the purses leaked, rounded or went below
//! nothing. Written for docs/probes/tr-money.md.
//!
//!     cargo run --release -p isostate-core --example money_trace -- [--seeds 1-8] [--rules game|port]
//!         [--families 6] [--end-year 1900] [--set JSON]... [--no-check] [--years] [--months FROM-TO]
//!
//! For every month it takes the money before the month (`econ::money_total`),
//! the issue (from `State::issued`), and from the issue the total the month
//! counted (`issue / (money_growth / 12)`). Money only moves inside a month, so
//! what the count left out is the money before the month less what was
//! counted. The books (on here; they change nothing) say what households were
//! paid in the year, by kind, and the left-out part should be all of it but
//! the issue. Then, by decade and pooled over the islands: the growth of the
//! money a year, the left-out part as a share of the money and what it was
//! made of, and the money in purses below nothing.
//!
//! `--set` takes a JSON object of parameters over the rules (more than one is
//! applied in turn). `--no-check` turns the money check off and reports, for
//! each island, the first month the check would have stopped it in: the
//! money, the month's own drift against what rounding on purses that size can
//! do, the largest purse, the money below nothing, and the issue's sign.
//! `--years` prints a line for every year of the first island; `--months
//! 1885-1890` prints every month of those years for every island.
#![allow(non_snake_case)]
use isostate_core::*;

#[derive(Clone, Default)]
struct Year {
    year: i64,
    m_jan: f64,
    m_dec: f64,
    m_sum: f64, // the money before each month, added up
    issue: f64,
    counted: f64,
    left_out: f64,
    left_out_share: f64, // mean over the twelve months of left_out / money before the month
    below: f64,          // money in purses below nothing, at the year's end
    below_kind: [f64; 6],
    gross: f64,          // all purses without their signs, at the year's end
    drift: f64,          // the largest month's own drift (after - before - issue), absolute
    books: Option<IslandYear>,
}

#[derive(Clone, Default)]
struct Fail {
    year: i64,
    month: usize,
    money: f64,
    cum: f64,
    own: f64,
    gross: f64,
    largest: f64,
    below: f64,
    issue: f64,
    neg_issue_months: usize,
    first_neg_money: Option<(i64, usize)>,
}

fn purses(st: &State) -> (f64, f64, f64) {
    let mut v: Vec<f64> = Vec::new();
    v.extend(&st.hh_cash);
    v.extend(&st.lord_cash);
    v.extend(&st.merch_cash);
    v.extend((0..st.f_cash.len()).filter(|&f| st.f_alive[f] || st.f_cash[f] != 0.0).map(|f| st.f_cash[f]));
    v.push(st.fund_cash);
    v.extend(st.projects.iter().map(|p| p.cash));
    v.extend(&st.hh_income);
    v.extend(&st.lord_income);
    v.extend(st.families.iter().map(|f| f.cash));
    let below: f64 = v.iter().filter(|x| **x < 0.0).sum();
    let gross: f64 = v.iter().map(|x| x.abs()).sum();
    let largest = v.iter().fold(0.0f64, |a, x| a.max(x.abs()));
    (below, gross, largest)
}

/// Money below nothing by whose purse: households, landlords, merchants, works, families, the fund and projects.
fn below_by_kind(st: &State) -> [f64; 6] {
    let neg = |v: &mut dyn Iterator<Item = f64>| v.filter(|x| *x < 0.0).sum::<f64>();
    [
        neg(&mut st.hh_cash.iter().copied()),
        neg(&mut st.lord_cash.iter().copied()),
        neg(&mut st.merch_cash.iter().copied()),
        neg(&mut (0..st.f_cash.len()).filter(|&f| st.f_alive[f] || st.f_cash[f] != 0.0).map(|f| st.f_cash[f])),
        neg(&mut st.families.iter().map(|f| f.cash)),
        neg(&mut std::iter::once(st.fund_cash).chain(st.projects.iter().map(|p| p.cash))),
    ]
}

fn run(P: &Params, rules: Rules, check: bool, months: Option<(i64, i64)>) -> (Vec<Year>, Option<Fail>, String) {
    let mut sim = sim::start(P, rules.content());
    sim.check_money = false;
    let g = P.money_growth / 12.0;
    let mut years = Vec::new();
    let mut fail: Option<Fail> = None;
    let (mut neg_issue_months, mut first_neg_money) = (0usize, None);
    let mut log = String::new();
    while sim.year <= sim.end_year {
        let year = sim.year;
        let mut y = Year { year, ..Default::default() };
        for month in 0..12 {
            let m0 = econ::money_total(&sim.st);
            if month == 0 {
                y.m_jan = m0;
            }
            let i0 = sim.st.issued;
            if let Err(e) = sim::run_month(&mut sim, month) {
                log.push_str(&format!("stopped: {e}\n"));
                return (years, fail, log);
            }
            let st = &sim.st;
            let issue = st.issued - i0;
            let counted = if g != 0.0 { issue / g } else { f64::NAN };
            let m1 = st.money_now;
            let own = m1 - m0 - issue;
            y.issue += issue;
            y.m_sum += m0;
            y.counted += counted;
            y.left_out += m0 - counted;
            y.left_out_share += (m0 - counted) / m0 / 12.0;
            y.drift = y.drift.max(own.abs());
            if issue < 0.0 {
                neg_issue_months += 1;
            }
            if m1 <= 0.0 && first_neg_money.is_none() {
                first_neg_money = Some((year, month));
            }
            let cum = m1 - (st.money_start + st.issued);
            if let Some((a, b)) = months {
                if year >= a && year <= b {
                    let (below, gross, largest) = purses(st);
                    log.push_str(&format!(
                        "  {year}-{:02} money {m1:12.4e} counted {counted:12.4e} issue {issue:11.3e} own drift {own:10.2e} cum {cum:10.2e} gross {gross:10.3e} largest {largest:10.3e} below {below:11.3e}\n",
                        month + 1
                    ));
                }
            }
            if fail.is_none() && cum.abs() >= 1e-6 * m1.max(1.0) {
                let (below, gross, largest) = purses(st);
                fail = Some(Fail {
                    year,
                    month,
                    money: m1,
                    cum,
                    own,
                    gross,
                    largest,
                    below,
                    issue,
                    neg_issue_months,
                    first_neg_money,
                });
                if check {
                    return (years, fail, log);
                }
            }
        }
        sim::end_year(&mut sim);
        let (below, gross, _) = purses(&sim.st);
        y.m_dec = econ::money_total(&sim.st);
        y.below = below;
        y.below_kind = below_by_kind(&sim.st);
        y.gross = gross;
        y.books = accounts::island_year(&sim.st);
        years.push(y);
    }
    if let Some(f) = &mut fail {
        f.neg_issue_months = neg_issue_months;
        f.first_neg_money = first_neg_money;
    }
    (years, fail, log)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut seeds, mut rules, mut families, mut end_year) = (vec![1u64, 2, 3, 4, 5, 6, 7, 8], Rules::Game, 6usize, None);
    let (mut sets, mut check, mut show_years, mut months) = (Vec::<String>::new(), true, false, None);
    let mut i = 0;
    while i < args.len() {
        let next = || args.get(i + 1).cloned().expect("a value after the option");
        match args[i].as_str() {
            "--seeds" => {
                let v = next();
                seeds = match v.split_once('-') {
                    Some((a, b)) => (a.parse().unwrap()..=b.parse().unwrap()).collect(),
                    None => v.split(',').map(|s| s.parse().unwrap()).collect(),
                };
            }
            "--rules" => rules = Rules::parse(&next()).expect("port or game"),
            "--families" => families = next().parse().unwrap(),
            "--end-year" => end_year = Some(next().parse().unwrap()),
            "--set" => sets.push(next()),
            "--months" => {
                let v = next();
                let (a, b) = v.split_once('-').expect("--months FROM-TO");
                months = Some((a.parse().unwrap(), b.parse().unwrap()));
            }
            "--no-check" => check = false,
            "--years" => show_years = true,
            other => panic!("unknown option {other}"),
        }
        i += if matches!(args[i].as_str(), "--no-check" | "--years") { 1 } else { 2 };
    }
    let mut P = rules.params();
    for s in &sets {
        P = P.with_json_overrides(s).expect("--set takes a JSON object of parameters");
    }
    P.n_families = families;
    P.end_year = end_year;
    P.accounts.enabled = true;

    let runs: Vec<(Vec<Year>, Option<Fail>, String)> = std::thread::scope(|s| {
        let hs: Vec<_> = seeds
            .iter()
            .map(|&seed| {
                let mut P = P.clone();
                P.seed = seed;
                s.spawn(move || run(&P, rules, check, months))
            })
            .collect();
        hs.into_iter().map(|h| h.join().expect("a run panicked")).collect()
    });

    println!("{} rules, {} families, seeds {:?}, money_growth {}; sets {:?}", rules.as_str(), families, seeds, P.money_growth, sets);
    for ((_, fail, log), seed) in runs.iter().zip(&seeds) {
        if !log.is_empty() {
            println!("seed {seed}:\n{log}");
        }
        if let Some(f) = fail {
            let eps = f64::EPSILON * f.gross;
            println!(
                "seed {seed}: the check fails in {}-{:02}: money {:.4e}, drift since the start {:.3e} (allowed {:.3e}); the month's own drift {:.3e}, \
                 which is {:.1} times the rounding of one sum over purses of {:.3e} (largest {:.3e}); below nothing {:.3e}; that month's issue {:.3e}; \
                 months with an issue below nothing so far {}; money first at or below nothing {:?}",
                f.year,
                f.month + 1,
                f.money,
                f.cum,
                1e-6 * f.money.max(1.0),
                f.own,
                f.own.abs() / eps.max(1e-300),
                f.gross,
                f.largest,
                f.below,
                f.issue,
                f.neg_issue_months,
                f.first_neg_money.map(|(y, m)| format!("{y}-{:02}", m + 1)),
            );
        } else {
            println!("seed {seed}: the check holds every month");
        }
    }

    // ---- by decade, pooled: sums over the islands, then the ratio
    let ys: Vec<&Vec<Year>> = runs.iter().map(|r| &r.0).collect();
    let n = ys.iter().map(|v| v.len()).min().unwrap_or(0);
    if n == 0 {
        return;
    }
    println!();
    println!("by decade, the islands pooled (sums first, then the ratio); [least .. greatest island] for the growth");
    println!(
        "{:>9} {:>14} {:>22} {:>9} {:>9} | of the left-out part: {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} {:>6} | {:>9} {:>9}",
        "years", "money a year", "[lo .. hi]", "left out", "issue/M", "pay", "cart", "navvy", "mgrs", "divW", "divM", "fund", "below/M", "booksgap"
    );
    println!("{:>150} below nothing, by purse: households, landlords, merchants, works, families, fund and projects", "");
    let mut k = 0;
    while k < n {
        let k1 = (k + 10).min(n);
        let span = (k1 - k) as f64;
        let mut growth: Vec<f64> = Vec::new();
        let (mut m_start, mut m_end, mut lo_share, mut cnt, mut issue) = (0.0, 0.0, 0.0, 0.0, 0.0);
        let mut parts = [0.0f64; 7];
        let (mut below, mut money_dec, mut gap, mut m_sum) = (0.0, 0.0, 0.0f64, 0.0);
        let mut bk = [0.0f64; 6];
        for v in &ys {
            let a = v[k].m_jan;
            let b = v[k1 - 1].m_dec;
            growth.push((b / a).powf(1.0 / span) - 1.0);
            m_start += a;
            m_end += b;
            for y in &v[k..k1] {
                lo_share += y.left_out_share;
                cnt += 1.0;
                issue += y.issue;
                below += y.below;
                money_dec += y.m_dec;
                m_sum += y.m_sum;
                for (q, x) in bk.iter_mut().zip(y.below_kind) {
                    *q += x;
                }
                if let Some(b) = &y.books {
                    let p = [b.pay_works + b.pay_land, b.carting, b.pay_projects, b.managers, b.div_households, b.merchants_div, b.fund_div];
                    for (q, x) in parts.iter_mut().zip(p) {
                        *q += x;
                    }
                    let hh = b.to_households - b.issue;
                    gap = gap.max((y.left_out - hh).abs() / hh.abs().max(1e-300));
                }
            }
        }
        let pooled = (m_end / m_start).powf(1.0 / span) - 1.0;
        let lo = growth.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = growth.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let tot: f64 = parts.iter().sum();
        print!(
            "{:>4}-{:>4} {:>13.3}% [{:>7.3}% .. {:>7.3}%] {:>9.4} {:>8.3}% |                       ",
            ys[0][k].year,
            ys[0][k1 - 1].year,
            100.0 * pooled,
            100.0 * lo,
            100.0 * hi,
            lo_share / cnt,
            100.0 * 12.0 * issue / m_sum.max(1e-300),
        );
        for q in parts {
            print!("{:>6.3} ", q / tot.max(1e-300));
        }
        print!("| {:>9.4} {:>9.1e} |", below / money_dec.max(1e-300), gap);
        for q in bk {
            print!(" {:>5.2}", q / below.min(-1e-300));
        }
        println!();
        k = k1;
    }
    // ---- the whole run, island by island
    println!();
    println!("island by island: money in the first January and the last December, the growth a year, the mean left-out share");
    for (v, seed) in ys.iter().zip(&seeds) {
        let a = v[0].m_jan;
        let b = v[v.len() - 1].m_dec;
        let span = v.len() as f64;
        let share: f64 = v.iter().map(|y| y.left_out_share).sum::<f64>() / span;
        let drift = v.iter().map(|y| y.drift / y.m_dec.abs().max(1.0)).fold(0.0f64, f64::max);
        println!(
            "  seed {seed}: {:>12.4e} to {:>12.4e}, {:.3}% a year over {} years; left out {:.4} of the money; worst month's own drift {:.1e} of the money",
            a,
            b,
            100.0 * ((b / a).powf(1.0 / span) - 1.0),
            span,
            share,
            drift
        );
    }
    if show_years {
        println!();
        println!("seed {} year by year: money in January, the year's issue, the counted total over twelve, left-out share, below nothing, households' pay+dividends", seeds[0]);
        for y in ys[0].iter() {
            let hh = y.books.as_ref().map(|b| b.to_households - b.issue).unwrap_or(f64::NAN);
            println!(
                "  {} {:>12.4e} {:>11.4e} {:>12.4e} {:>7.4} {:>12.4e} {:>12.4e}",
                y.year,
                y.m_jan,
                y.issue,
                y.counted / 12.0,
                y.left_out_share,
                y.below,
                hh
            );
        }
    }
}
