mod clusters;
mod corpus;
mod scores;
mod units;
mod words;

use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Default)]
pub struct Texts {
    pub code: BTreeMap<PathBuf, String>,
    pub contracts: BTreeMap<PathBuf, String>,
}

pub fn report(texts: &Texts) -> Result<String> {
    let scores::Measured { findings, widths } = scores::measure(&corpus::Corpus::new(texts)?);
    let total: f64 = findings.iter().map(|finding| finding.score).sum();
    let units = findings.len();
    let mean = total / units::real(units.max(1));
    let mut findings: Vec<scores::Finding> = findings.into_iter().chain(widths).collect();
    findings.sort_by(|a, b| {
        let order = b.score.total_cmp(&a.score);
        order.then_with(|| (&a.path, a.kind).cmp(&(&b.path, b.kind)))
    });
    let mut lines = vec![format!("total {mean:.1} units {units}")];
    for scores::Finding {
        score,
        kind,
        path,
        detail,
    } in &findings
    {
        let line = format!("{score:>3.0} {kind} {path} {detail}");
        lines.push(line.trim_end().to_owned());
    }
    Ok(lines.join("\n"))
}
