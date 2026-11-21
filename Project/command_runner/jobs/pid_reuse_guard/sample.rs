use serde::Serialize;

#[derive(Serialize)]
pub struct Resources {
    pub cpu_millis: u64,
    pub resident_bytes: u64,
}
pub(super) struct Sample {
    pub(super) start: u64,
    pub(super) zombie: bool,
    pub(super) resources: Resources,
}
