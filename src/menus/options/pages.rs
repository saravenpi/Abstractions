//! The pages of the settings menu, one for each tab.

use bevy::prelude::*;

use super::Tab;
use super::field::{Bind, Field};
use super::rows::{binding, stepper, toggle};

/// Marks the container of one tab.
#[derive(Component)]
pub struct Page(pub Tab);

/// Display mode for a page: visible only on the active tab.
pub fn display_for(page: Tab, active: Tab) -> Display {
    if page == active {
        Display::Flex
    } else {
        Display::None
    }
}

/// Column that holds the rows of one tab.
pub fn page(tab: Tab, active: Tab) -> impl Bundle {
    (
        Page(tab),
        Node {
            width: percent(100),
            flex_direction: FlexDirection::Column,
            flex_shrink: 0.0,
            display: display_for(tab, active),
            ..default()
        },
    )
}

/// Graphics tab.
pub fn graphics(active: Tab) -> impl Bundle {
    let light = (
        Spawn(stepper("Lighting quality", Field::LightingQuality)),
        Spawn(toggle("Shadows", Field::Shadows)),
        Spawn(stepper("Shadow detail", Field::ShadowDetail)),
        Spawn(stepper("Shadow distance", Field::ShadowDistance)),
        Spawn(toggle("Ambient occlusion", Field::AmbientOcclusion)),
        Spawn(toggle("Bloom", Field::Bloom)),
        Spawn(toggle("Clouds", Field::Clouds)),
        Spawn(toggle("Cloud shadows", Field::CloudShadows)),
        Spawn(toggle("Auto exposure", Field::AutoExposure)),
        Spawn(toggle("White balance", Field::WhiteBalance)),
        Spawn(toggle("Lamp shadows", Field::LocalShadows)),
        Spawn(toggle("Sea reflections", Field::WaterGlint)),
    );
    (
        page(Tab::Graphics, active),
        Children::spawn((
            Spawn(stepper("Render distance", Field::RenderDistance)),
            light,
            Spawn(toggle("Vsync", Field::Vsync)),
            Spawn(toggle("Fullscreen", Field::Fullscreen)),
            Spawn(toggle("Show FPS", Field::ShowFps)),
            Spawn(stepper("Field of view", Field::Fov)),
        )),
    )
}

/// Controls tab: the mouse, the camera and the keys used on foot.
pub fn controls(active: Tab) -> impl Bundle {
    (
        page(Tab::Controls, active),
        Children::spawn((
            Spawn(stepper("Mouse sensitivity", Field::Sensitivity)),
            Spawn(toggle("Invert Y", Field::InvertY)),
            Spawn(toggle("View on foot", Field::ViewFoot)),
            Spawn(toggle("View in ship", Field::ViewShip)),
            Spawn(toggle("Head bob", Field::HeadBob)),
            Spawn(toggle("Camera auto-follow", Field::AutoFollow)),
            Spawn(stepper("Speed effects", Field::SpeedFx)),
            SpawnIter(Bind::FOOT.into_iter().map(binding)),
        )),
    )
}

/// Ship tab: the keys used while seated. Boarding and the view key are on the controls tab.
pub fn ship(active: Tab) -> impl Bundle {
    let shared = [Bind::Board, Bind::View];
    let seated = Bind::SHIP
        .into_iter()
        .filter(move |bind| !shared.contains(bind));
    (
        page(Tab::Ship, active),
        Children::spawn(SpawnIter(seated.map(binding))),
    )
}

/// General tab: what is neither picture, controls nor sound.
pub fn general(active: Tab) -> impl Bundle {
    (
        page(Tab::General, active),
        children![
            toggle("Check for updates at launch", Field::CheckUpdates),
            stepper("Hitstop", Field::Hitstop),
            toggle("Markers over the world", Field::WorldMarkers),
            toggle("Blood", Field::Blood),
            stepper("Interface size", Field::Interface),
        ],
    )
}

/// Audio tab.
pub fn audio(active: Tab) -> impl Bundle {
    (
        page(Tab::Audio, active),
        children![
            stepper("Master", Field::Master),
            stepper("Music", Field::Music),
            stepper("Effects", Field::Effects),
        ],
    )
}
