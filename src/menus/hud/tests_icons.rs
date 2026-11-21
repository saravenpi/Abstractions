//! Tests of the icons of the HUD: every row of the ship readout has an icon and a label, the
//! gear and the headlight show their state in their icon, and the readout and the climate strip
//! fit the screen at every interface size from a 960 by 540 window up.

use bevy::math::Vec2;
use bevy::prelude::*;

use super::climate::{WORDS, sky_icon};
use super::flight::{Flight, Gauge, reading, symbol};
use super::panel::{LABEL_WIDTH, Symbol, block};
use super::switch::key;
use crate::settings::InterfaceSize;
use crate::ship::ShipTelemetry;
use crate::space::Frame;
use crate::ui::{Icon, theme, ui_scale};
use crate::universe::BodyId;

/// Returns a ship on the ground of a planet with its gear at `gear` and the headlight `on`.
fn landed(gear: f64, on: bool) -> Flight {
    Flight {
        telemetry: ShipTelemetry {
            frame: Frame::Surface(BodyId::default()),
            ..ShipTelemetry::default()
        },
        gear: Some(gear),
        headlight: Some((on, "L".to_string())),
        ..Flight::default()
    }
}

/// The block holds one icon for each of its rows, and each row has a label.
#[test]
fn every_readout_row_has_an_icon_and_a_label() {
    let mut world = World::new();
    world.spawn(block());
    let mut symbols = world.query::<&Symbol>();
    assert_eq!(symbols.iter(&world).count(), Gauge::ALL.len());
    for (gauge, name) in Gauge::ALL {
        assert!(!name.is_empty());
        let icon = symbol(gauge, &Flight::default());
        assert_eq!(icon.glyph().chars().count(), 1, "{gauge:?}");
    }
    let rows: Vec<&str> = Gauge::ALL.iter().map(|(_, name)| *name).collect();
    assert!(rows.contains(&"Headlight"));
}

/// The gear icon differs between down, up and on its way, the headlight icon between on and off,
/// and the headlight row says on or off, names its key after it, and shows only near a planet.
#[test]
fn the_gear_and_the_headlight_show_their_state() {
    let icons = [1.0, 0.0, 0.5].map(|gear| symbol(Gauge::Gear, &landed(gear, true)));
    assert_ne!(icons[0], icons[1]);
    assert_ne!(icons[1], icons[2]);
    assert_ne!(icons[0], icons[2]);
    assert_ne!(
        symbol(Gauge::Headlight, &landed(1.0, true)),
        symbol(Gauge::Headlight, &landed(1.0, false))
    );
    assert_eq!(
        reading(Gauge::Headlight, &landed(1.0, true)).as_deref(),
        Some("on")
    );
    assert_eq!(
        reading(Gauge::Headlight, &landed(1.0, false)).as_deref(),
        Some("off")
    );
    assert_eq!(key(Gauge::Headlight, &landed(1.0, true)), "L");
    assert_eq!(key(Gauge::Gear, &landed(1.0, true)), "");
    let space = Flight {
        telemetry: ShipTelemetry {
            frame: Frame::System(Default::default()),
            ..ShipTelemetry::default()
        },
        ..landed(1.0, true)
    };
    assert_eq!(reading(Gauge::Headlight, &space), None);
}

/// Width in pixels of `chars` characters of small text.
fn words(chars: usize) -> f32 {
    chars as f32 * theme::GLYPH_ASPECT * theme::FONT_SMALL
}

/// At every interface size in windows from 960 by 540 up, the widest row of the ship readout and
/// the climate strip with its longest weather word fit the screen inside the edge margin.
#[test]
fn the_readout_and_the_climate_strip_fit_at_every_size() {
    let longest = "66.7 x light speed warp".chars().count();
    let row =
        2.0 * theme::S4 + theme::ICON_SMALL + theme::S2 + LABEL_WIDTH + theme::S2 + words(longest);
    let word = WORDS.iter().map(|word| word.len()).max().unwrap();
    let strip = 2.0 * theme::S3
        + 2.0 * (theme::ICON_SMALL + theme::S2)
        + words("-123 C".len())
        + theme::S2
        + words(word);
    for window in [
        Vec2::new(960.0, 540.0),
        Vec2::new(1280.0, 720.0),
        Vec2::new(1920.0, 1080.0),
    ] {
        for size in InterfaceSize::ALL {
            let logical = window / ui_scale(window, size);
            let across = logical.x - 2.0 * theme::EDGE;
            assert!(row <= across, "{window} {size:?}: {row} over {across}");
            assert!(strip <= across * 0.5, "{window} {size:?}: {strip}");
        }
    }
}

/// Every weather word has a glyph that is not the one of another word, by day and by night.
#[test]
fn weather_icons_are_distinct_by_day_and_night() {
    for day in [true, false] {
        let mut icons: Vec<Icon> = WORDS
            .iter()
            .filter_map(|word| sky_icon(word, day))
            .collect();
        assert_eq!(icons.len(), WORDS.len());
        icons.sort_by_key(|icon| *icon as usize);
        icons.dedup();
        assert_eq!(icons.len(), WORDS.len());
    }
}
