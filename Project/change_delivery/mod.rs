mod gate;
mod git;
pub use gate::{
    AffectedMap, arguments as check_arguments, files as checked_files, included,
    selected as check_selected,
};
pub use git::{
    approve, compaction_due, consented_pins, give_consent, guard_commit, guard_reference,
    lift_compaction, merge as feature_merge, propose_pin, push as feature_push, record_approval,
    reset_state, revoke_consent, start as feature_start,
};
