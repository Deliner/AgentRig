use super::findings;
use crate::structural_lint::rules::reference::Reference;

#[test]
fn rust_flags_searches_in_error_text_bound_by_err_arms_guards_if_let_branches_and_map_err() {
    let source = "fn load(store: &Store, id: u32, names: &[&str]) -> Result<Item, Error> {
    match store.load(id) {
        Err(error) if error.to_string().contains(\"not found\") => Ok(Item::empty()),
        Err(error) => {
            if format!(\"{error}\").starts_with(\"timeout\") {
                return retry(id);
            }
            if names.iter().any(|name| error.to_string().ends_with(name)) {
                return retry(id);
            }
            Err(error)
        }
        Ok(item) => Ok(item),
    }
}
fn save(store: &Store, item: Item) -> Result<(), String> {
    if let Err(failure) = store.save(&item) {
        let busy = format!(\"{}\", failure).ends_with(\"locked\");
    }
    store.save(&item).map_err(|cause| {
        if cause.to_string().contains(\"denied\") {
            \"denied\".to_owned()
        } else {
            \"other\".to_owned()
        }
    })
}
";
    assert_eq!(
        findings(Reference::ErrorTextBranch, source),
        [
            "3:contains",
            "5:starts_with",
            "8:ends_with",
            "18:ends_with",
            "21:contains"
        ]
    );
}

#[test]
fn rust_keeps_variant_matches_other_text_shadowed_names_justified_and_test_code() {
    let source = "fn keep(store: &Store, output: &Output, names: &[String]) -> Result<(), Error> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains(\"not found\") {}
    match store.load(1) {
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => {
            let text = error.to_string();
            if text.contains(\"busy\") {}
            if error.message().contains(\"busy\") {}
            if error.to_string().to_lowercase().contains(\"busy\") {}
            if names.iter().any(|error| error.to_string().contains(\"busy\")) {}
        }
        Ok(item) => {
            if item.to_string().contains(\"busy\") {}
        }
    }
    if let Err(error) = store.save() {
    } else if error.to_string().contains(\"busy\") {
    }
    store.save().or_else(|cause| if cause.to_string().contains(\"busy\") { Ok(()) } else { Err(cause) })?;
    if let Err(error) = store.save() {
        // DECISION: D001
        if error.to_string().contains(\"busy\") {}
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    fn probe(result: Result<(), Error>) {
        if let Err(error) = result {
            let found = error.to_string().contains(\"busy\");
        }
    }
}
";
    assert!(findings(Reference::ErrorTextBranch, source).is_empty());
}
