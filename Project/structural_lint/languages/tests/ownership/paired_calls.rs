use super::measured;
use crate::structural_lint::rule_identities::ownership::{Ownership, Pair};

fn pairs() -> [Pair; 2] {
    let pair = |open: &str, close: &str| Pair {
        open: open.into(),
        close: close.into(),
    };
    [pair("begin", "commit"), pair("lock", "unlock")]
}

#[test]
fn rust_flags_an_opening_call_whose_closing_call_follows_an_early_exit() {
    let source = "fn save(db: &mut Db, items: &[Item]) -> Result<()> {
    db.begin();
    write(items)?;
    db.commit();
    for item in items {
        self.store.lock();
        if item.skip {
            continue;
        }
        self.store.unlock();
    }
    Ok(())
}
";
    assert_eq!(
        measured(source, &pairs(), Ownership::PairedCalls),
        [(2, "begin".to_string()), (6, "lock".to_string())]
    );
}

#[test]
fn rust_skips_closure_exits_other_receivers_unconfigured_pairs_and_justified_calls() {
    let source = "fn closure(db: &mut Db) -> Result<()> {
    db.begin();
    run(|| -> Result<()> { write()?; Ok(()) });
    db.commit();
    Ok(())
}
fn receiver(db: &mut Db, other: &mut Db) -> Result<()> {
    db.begin();
    write()?;
    other.commit();
    Ok(())
}
fn unconfigured(file: &mut File) -> Result<()> {
    file.open();
    write()?;
    file.close();
    Ok(())
}
fn closed(db: &mut Db) -> Result<()> {
    db.begin();
    db.commit();
    write()?;
    Ok(())
}
fn justified(db: &mut Db) -> Result<()> {
    // DECISION: D001
    db.begin();
    write()?;
    db.commit();
    Ok(())
}
";
    assert!(measured(source, &pairs(), Ownership::PairedCalls).is_empty());
}

#[test]
fn rust_measures_no_paired_calls_without_configured_pairs() {
    let source = "fn save(db: &mut Db) -> Result<()> {
    db.begin();
    write()?;
    db.commit();
    Ok(())
}
";
    assert!(measured(source, &[], Ownership::PairedCalls).is_empty());
}
