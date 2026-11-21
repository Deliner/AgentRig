use anyhow::Result;
use std::num::NonZero;

pub fn in_parallel<T: Sync, R: Send>(items: &[T], each: impl Fn(&T) -> R + Sync) -> Result<Vec<R>> {
    let available = std::thread::available_parallelism().map_or(1, NonZero::get);
    let workers = available.min(items.len());
    let each = &each;
    std::thread::scope(|scope| {
        let handles = Vec::from_fn(workers, |worker| {
            scope.spawn(move || {
                let assigned = items.iter().enumerate().skip(worker).step_by(workers);
                assigned
                    .map(|(index, item)| (index, each(item)))
                    .collect::<Vec<_>>()
            })
        });
        let mut done = Vec::with_capacity(items.len());
        for handle in handles {
            let part = handle
                .join()
                .map_err(|panic| anyhow::anyhow!("source inspection panicked: {panic:?}"))?;
            done.extend(part);
        }
        done.sort_by_key(|(index, _)| *index);
        Ok(done.into_iter().map(|(_, result)| result).collect())
    })
}
