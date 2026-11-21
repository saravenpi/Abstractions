//! Tests of the water reflections option: what each preset sets, that it survives a save, and
//! that a file written before it existed reads the option of its own preset.

use super::{Graphics, Quality, Reflections, Settings};

/// Returns the settings a file of the old format holds: `graphics` as written by a game that had
/// no reflections option.
fn old_file(quality: Quality) -> String {
    let graphics = Graphics::default().with_quality(quality);
    let mut settings = Settings {
        graphics,
        ..Settings::default()
    };
    settings.graphics.lighting.reflections = Reflections::Off;
    let text = toml::to_string(&settings).unwrap();
    let kept: Vec<&str> = text
        .lines()
        .filter(|line| !line.starts_with("reflections"))
        .collect();
    kept.join("\n")
}

/// The actors are mirrored on High and Ultra and nothing is on Low and Medium; the default is
/// High, so a new player sees them.
#[test]
fn high_and_ultra_mirror_the_actors() {
    let own = Graphics::default();
    assert_eq!(own.lighting.reflections, Reflections::Actors);
    for (quality, want) in [
        (Quality::Low, Reflections::Off),
        (Quality::Medium, Reflections::Off),
        (Quality::High, Reflections::Actors),
        (Quality::Ultra, Reflections::Actors),
    ] {
        assert_eq!(
            own.with_quality(quality).lighting.reflections,
            want,
            "{quality:?}"
        );
    }
}

/// Changing the option by hand reads as a custom preset, and each value survives a save.
#[test]
fn the_option_round_trips() {
    for value in Reflections::ALL {
        let mut settings = Settings::default();
        settings.graphics.lighting.reflections = value;
        let text = toml::to_string(&settings).unwrap();
        let back: Settings = toml::from_str(&text).unwrap();
        assert_eq!(back.graphics.lighting.reflections, value, "{text}");
        assert_eq!(back, settings);
    }
    let mut own = Graphics::default();
    own.lighting.reflections = Reflections::Off;
    assert_eq!(own.quality(), Quality::Custom);
}

/// A file without the option loads: one on a preset reads the option of that preset and stays on
/// it, so no player on Low or Medium gets the extra pass and nobody on High loses it.
#[test]
fn old_files_load_with_the_option_of_their_preset() {
    for quality in Quality::PRESETS {
        let text = old_file(quality);
        assert!(!text.contains("reflections"), "{text}");
        let parsed: Settings = toml::from_str(&text).unwrap();
        assert_eq!(parsed.graphics.quality(), quality, "{text}");
    }
    let bare: Settings = toml::from_str("[graphics.lighting]\nauto_exposure = false\n").unwrap();
    assert_eq!(bare.graphics.lighting.reflections, Reflections::Actors);
    assert!(!bare.graphics.lighting.auto_exposure);
    let empty: Settings = toml::from_str("").unwrap();
    assert_eq!(empty, Settings::default());
}

/// A step cycles through the options, and each one has its name.
#[test]
fn a_step_cycles() {
    assert_eq!(Reflections::Off.next(), Reflections::Actors);
    assert_eq!(Reflections::Actors.next(), Reflections::Off);
    assert_eq!(Reflections::Off.name(), "Off");
    assert_eq!(Reflections::Actors.name(), "Actors");
}
