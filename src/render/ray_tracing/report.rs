//! The plain text diagnostics file `ray-tracing.log`, built by pure functions and written without ever panicking.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::Path;

use bevy::prelude::*;

/// Name of the diagnostics file, kept next to the settings file.
pub const LOG: &str = "ray-tracing.log";

/// Everything the start entry of the log reports.
pub struct StartInfo {
    /// Game version.
    pub version: &'static str,
    /// Adapter name, backend and device type.
    pub adapter: String,
    /// Each required wgpu feature and whether the device has it.
    pub features: Vec<(&'static str, bool)>,
    /// Each binding array limit as name, value and required value.
    pub limits: Vec<(&'static str, u32, u32)>,
    /// Whether the HDR target format can be bound for reading and writing.
    pub hdr_storage: bool,
    /// The result of the GPU check.
    pub support: String,
    /// The Solari workload chosen for the device.
    pub workload: &'static str,
    /// Whether the Solari plugins were added.
    pub added: bool,
    /// What the boot guard decided.
    pub guard: String,
    /// Whether the setting asks for ray tracing.
    pub setting: bool,
    /// Whether ray tracing is active.
    pub active: bool,
}

/// What a toggle entry of the log reports.
pub struct ToggleInfo {
    /// Seconds since the game started.
    pub at: f32,
    /// What happened: the toggle itself or the state once it settled.
    pub label: &'static str,
    /// Whether ray tracing is active.
    pub active: bool,
    /// Entities carrying a ray tracing mesh.
    pub meshes: usize,
    /// Cameras carrying Solari lighting.
    pub cameras: usize,
    /// Size of the game camera target, when there is one.
    pub target: Option<UVec2>,
}

/// Returns "yes" or "no".
fn yes(flag: bool) -> &'static str {
    if flag { "yes" } else { "no" }
}

/// Formats the entry written at every start.
pub fn start(info: &StartInfo) -> String {
    let mut text = format!("Abstractions {} ray tracing log\n", info.version);
    let _ = writeln!(text, "adapter: {}", info.adapter);
    for (name, present) in &info.features {
        let state = if *present { "present" } else { "missing" };
        let _ = writeln!(text, "feature {name}: {state}");
    }
    for (name, have, need) in &info.limits {
        let _ = writeln!(text, "limit {name}: {have} (needs {need})");
    }
    let _ = writeln!(
        text,
        "Rgba16Float read-write storage: {}",
        yes(info.hdr_storage)
    );
    let _ = writeln!(text, "gpu check: {}", info.support);
    let _ = writeln!(text, "workload: {}", info.workload);
    let _ = writeln!(text, "plugins added: {}", yes(info.added));
    let _ = writeln!(text, "boot guard: {}", info.guard);
    let _ = writeln!(text, "setting: {}", if info.setting { "on" } else { "off" });
    let _ = writeln!(text, "ray tracing active: {}", yes(info.active));
    text
}

/// Formats an entry written when ray tracing is toggled or once the toggle settled.
pub fn toggle(info: &ToggleInfo) -> String {
    let target = info
        .target
        .map_or_else(|| "none".to_string(), |t| format!("{}x{}", t.x, t.y));
    let mut text = format!(
        "[{:.1}s] {}: active {}",
        info.at,
        info.label,
        yes(info.active)
    );
    let _ = write!(text, ", ray tracing meshes {}", info.meshes);
    let _ = write!(text, ", cameras with Solari lighting {}", info.cameras);
    let _ = writeln!(text, ", camera target {target}");
    text
}

/// Writes `text` to the log in `dir`, replacing the file; failures are logged and ignored.
pub fn write(dir: &Path, text: &str) {
    let done = std::fs::create_dir_all(dir).and_then(|()| std::fs::write(dir.join(LOG), text));
    if let Err(error) = done {
        warn!("could not write {LOG}: {error}");
    }
}

/// Appends `text` to the log in `dir`; failures are logged and ignored.
pub fn append(dir: &Path, text: &str) {
    let open = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(LOG));
    if let Err(error) = open.and_then(|mut file| file.write_all(text.as_bytes())) {
        warn!("could not append to {LOG}: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The start entry names every feature, limit and decision.
    #[test]
    fn start_entry_is_complete() {
        let text = start(&StartInfo {
            version: "1.2.3",
            adapter: "Apple M2 (Metal, IntegratedGpu)".into(),
            features: vec![("A", true), ("B", false)],
            limits: vec![("samplers", 1024, 5000)],
            hdr_storage: true,
            support: "too limited".into(),
            workload: "light (no ray tracing hardware)",
            added: false,
            guard: "armed".into(),
            setting: true,
            active: false,
        });
        for line in [
            "Abstractions 1.2.3",
            "adapter: Apple M2",
            "feature A: present",
            "feature B: missing",
            "limit samplers: 1024 (needs 5000)",
            "workload: light",
            "plugins added: no",
            "setting: on",
            "ray tracing active: no",
        ] {
            assert!(text.contains(line), "{line} missing in {text}");
        }
    }

    /// The toggle entry carries the counts and the target size.
    #[test]
    fn toggle_entry_has_counts() {
        let text = toggle(&ToggleInfo {
            at: 12.34,
            label: "settled",
            active: true,
            meshes: 7,
            cameras: 1,
            target: Some(UVec2::new(960, 600)),
        });
        assert!(text.contains("[12.3s] settled"));
        assert!(text.contains("meshes 7") && text.contains("960x600"));
    }

    /// An unwritable directory is reported, not fatal.
    #[test]
    fn unwritable_directory_does_not_panic() {
        let file = std::env::temp_dir().join(format!("abstractions-log-{}", std::process::id()));
        std::fs::write(&file, b"").unwrap();
        write(&file, "x");
        append(&file, "x");
        std::fs::remove_file(&file).unwrap();
    }
}
