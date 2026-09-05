use super::storage;
use anyhow::Result;
use std::{collections::BTreeMap, path::Path};

pub fn print(runtime: &Path) -> Result<()> {
    let mut totals = BTreeMap::<String, (usize, usize, f64, f64)>::new();
    for record in storage::list(runtime)? {
        let Some(code) = record.exit_code else {
            continue;
        };
        let duration = record.duration_seconds.unwrap_or(0.0);
        let entry = totals.entry(record.command).or_default();
        entry.0 += 1;
        entry.1 += usize::from(code != 0);
        entry.2 += duration;
        entry.3 = entry.3.max(duration);
    }
    println!("command calls failures total_s avg_s max_s");
    for (name, (calls, failed, total, max)) in totals {
        println!(
            "{name} {calls} {failed} {total:.3} {:.3} {max:.3}",
            total / calls as f64
        );
    }
    Ok(())
}
