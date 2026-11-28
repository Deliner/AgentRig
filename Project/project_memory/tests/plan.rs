// DECISION: D012
// DECISION: D242
use super::consumer::{accepts, card, consumer, rejects};

type Card = (&'static str, &'static str);

const CHECK: &[&str] = &["memory-check"];
const CONTRACT: [(&[Card], Option<&str>); 9] = [
    (&[], None),
    (&[("notes/Plan/RIG-1.md", "")], None),
    (
        &[
            ("notes/Archive/RIG-1.md", ""),
            ("notes/Plan/RIG-2.md", "After: RIG-1"),
        ],
        None,
    ),
    (
        &[
            ("notes/Backlog/RIG-3.md", ""),
            ("notes/Plan/RIG-1.md", "After: RIG-2, RIG-3"),
            ("notes/Plan/RIG-2.md", "After: RIG-3"),
        ],
        None,
    ),
    (
        &[
            ("notes/Plan/RIG-1.md", ""),
            ("notes/Archive/RIG-1/RIG-1.md", ""),
        ],
        Some("RIG-1 lies in both"),
    ),
    (
        &[("notes/Plan/RIG-2.md", "After: RIG-1")],
        Some("After names RIG-1, which is no card"),
    ),
    (
        &[("notes/Plan/RIG-1.md", "After: P001")],
        Some("After names P001, which is not a card name"),
    ),
    (
        &[("notes/Plan/RIG-1.md", "After: RIG-1")],
        Some("After forms a cycle that leaves RIG-1 waiting"),
    ),
    (
        &[
            ("notes/Plan/RIG-1.md", "After: RIG-2"),
            ("notes/Plan/RIG-2.md", "After: RIG-1"),
        ],
        Some("After forms a cycle that leaves RIG-1, RIG-2 waiting"),
    ),
];

// INVARIANT: I009
#[test]
fn plan_delivery_contract() {
    for (cards, expected) in CONTRACT {
        let (_temp, root) = consumer();
        for (place, preamble) in cards {
            card(&root, place, preamble);
        }
        match expected {
            None => accepts(&root),
            Some(diagnostic) => rejects(&root, CHECK, diagnostic),
        }
    }
}
