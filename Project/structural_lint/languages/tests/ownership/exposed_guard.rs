use super::measured;
use crate::structural_lint::rules::ownership::Ownership;

// DECISION: D166
#[test]
fn rust_flags_visible_functions_that_return_a_lock_guard_or_cell_borrow_at_any_depth() {
    let source = "impl Store {
    pub fn state(&self) -> MutexGuard<'_, State> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }
    pub(crate) fn read(&self) -> LockResult<std::sync::RwLockReadGuard<'_, State>> {
        self.rw.read()
    }
    pub(super) fn write(&self) -> Option<RwLockWriteGuard<'_, State>> {
        self.rw.write().ok()
    }
    pub fn borrow(&self) -> Ref<'_, State> {
        self.cell.borrow()
    }
    pub fn borrow_mut(&self) -> std::cell::RefMut<'_, State> {
        self.cell.borrow_mut()
    }
}
#[must_use]
pub fn global() -> parking_lot::MutexGuard<'static, State> {
    STATE.lock()
}
";
    assert_eq!(
        measured(source, &[], Ownership::ExposedGuard),
        [
            (2, "state".to_string()),
            (5, "read".to_string()),
            (8, "write".to_string()),
            (11, "borrow".to_string()),
            (14, "borrow_mut".to_string()),
            (19, "global".to_string()),
        ]
    );
}

#[test]
fn rust_skips_private_trait_justified_test_and_other_named_returns() {
    let source = "impl Store {
    fn state(&self) -> MutexGuard<'_, State> {
        self.inner.lock().unwrap()
    }
    pub fn mapped(&self) -> MappedMutexGuard<'_, u32> {
        map(self.state())
    }
    pub fn owned(&self) -> OwnedMutexGuard<State> {
        self.shared.lock_owned()
    }
    pub fn with_state<R>(&self, f: impl FnOnce(&mut State) -> R) -> R {
        f(&mut self.state())
    }
    // DECISION: D001
    pub fn lock(&self) -> MutexGuard<'_, State> {
        self.state()
    }
}
pub trait Lock {
    fn lock(&self) -> MutexGuard<'_, State>;
    fn read(&self) -> Ref<'_, State> {
        self.cell().borrow()
    }
}
impl Lock for Store {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state()
    }
}
#[cfg(test)]
mod tests {
    pub fn probe(store: &Store) -> MutexGuard<'_, State> {
        store.state()
    }
}
";
    assert!(measured(source, &[], Ownership::ExposedGuard).is_empty());
}
