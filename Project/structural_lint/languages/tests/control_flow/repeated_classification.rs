use super::measured;
use crate::structural_lint::rule_identities::control_flow::ControlFlow;

const MODE: &str = "Mode::Idle | Mode::Off | Mode::Slow => false, Mode::Fast => true";
const DEVICE: &str = "Device::Discrete => false, Device::Cpu | Device::Other => true";

#[test]
fn rust_measures_each_match_classifying_three_or_more_paths_into_true_and_false() {
    let source = "fn fast(mode: Mode) -> bool {
    match mode {
        Mode::Fast => true,
        Mode::Slow | Mode::Idle | Mode::Off => false,
    }
}
fn queue(mode: Mode) -> bool {
    let fast = match mode {
        Mode::Off | Mode::Idle => false,
        crate::engine::Mode::Fast => true,
        | Mode::Slow => false,
    };
    fast
}
#[cfg(test)]
mod tests {
    fn shared(kind: wgpu::Device) -> bool {
        match kind {
            wgpu::Device::Discrete => false,
            wgpu::Device::Other | wgpu::Device::Cpu => true,
        }
    }
}
";
    assert_eq!(
        measured(source, ControlFlow::RepeatedClassification),
        [(2, MODE.into()), (8, MODE.into()), (18, DEVICE.into())]
    );
}

#[test]
fn rust_skips_guarded_bound_wildcard_short_one_sided_and_valued_matches() {
    let source = "fn check(mode: Mode, input: Option<Mode>, ready: bool) -> bool {
    let guarded = match mode {
        Mode::Fast if ready => true,
        Mode::Slow | Mode::Idle => false,
        Mode::Off | Mode::Fast => false,
    };
    let bound = match input {
        Some(Mode::Fast) => true,
        Some(Mode::Slow | Mode::Idle | Mode::Off) | None => false,
    };
    let wildcard = match mode {
        Mode::Fast | Mode::Off => true,
        _ => false,
    };
    let short = match mode {
        Mode::Fast => true,
        Mode::Off => false,
    };
    let one_sided = match mode {
        Mode::Fast | Mode::Slow | Mode::Idle => true,
    };
    let valued = match mode {
        Mode::Fast => { true }
        Mode::Slow | Mode::Idle | Mode::Off => false,
    };
    guarded && bound && wildcard && short && one_sided && valued
}
";
    assert!(measured(source, ControlFlow::RepeatedClassification).is_empty());
}
