//! The pages of the settings menu, one for each tab. A page is a column of sections, and a
//! section a title over its rows.

use bevy::prelude::*;

use super::Tab;
use super::field::{Bind, Field};
use super::rows::{binding, reset, stepper, toggle};
use crate::ui::{Icon, rows, section, sections};

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

/// The container of one tab, shown on its tab only. `content` is a `children!` list of the
/// sections of the page.
pub fn page(tab: Tab, active: Tab, content: impl Bundle) -> impl Bundle {
    let node = Node {
        width: percent(100),
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        display: display_for(tab, active),
        ..default()
    };
    (Page(tab), node, children![(sections(), content)])
}

/// The lighting rows of the graphics tab.
fn light() -> impl Bundle {
    children![
        stepper("Lighting quality", Field::LightingQuality),
        toggle("Shadows", Field::Shadows),
        stepper("Shadow detail", Field::ShadowDetail),
        stepper("Shadow distance", Field::ShadowDistance),
        toggle("Lamp shadows", Field::LocalShadows),
        toggle("Ambient occlusion", Field::AmbientOcclusion),
        toggle("Bloom", Field::Bloom),
        toggle("Auto exposure", Field::AutoExposure),
        toggle("White balance", Field::WhiteBalance),
    ]
}

/// Graphics tab: how far and how wide the view is, the light, the sky and the window.
pub fn graphics(active: Tab) -> impl Bundle {
    let view = children![
        stepper("Render distance", Field::RenderDistance),
        stepper("Field of view", Field::Fov),
    ];
    let sky = children![
        toggle("Clouds", Field::Clouds),
        toggle("Cloud shadows", Field::CloudShadows),
        toggle("Sea glint", Field::WaterGlint),
        toggle("Water reflections", Field::WaterReflections),
    ];
    let display = children![
        toggle("Vsync", Field::Vsync),
        toggle("Fullscreen", Field::Fullscreen),
        toggle("Show FPS", Field::ShowFps),
    ];
    let content = children![
        section(Icon::Eye, "View", (rows(), view)),
        section(Icon::Light, "Light", (rows(), light())),
        section(Icon::Cloudy, "Sky and sea", (rows(), sky)),
        section(Icon::Monitor, "Display", (rows(), display)),
    ];
    page(Tab::Graphics, active, content)
}

/// Controls tab: the mouse, the camera, the keys used on foot and the row that resets the keys.
pub fn controls(active: Tab) -> impl Bundle {
    let mouse = children![
        stepper("Mouse sensitivity", Field::Sensitivity),
        toggle("Invert Y", Field::InvertY),
    ];
    let camera = children![
        toggle("View on foot", Field::ViewFoot),
        toggle("View in ship", Field::ViewShip),
        toggle("Head bob", Field::HeadBob),
        toggle("Camera auto-follow", Field::AutoFollow),
        stepper("Speed effects", Field::SpeedFx),
    ];
    let keys = Children::spawn((
        SpawnIter(Bind::FOOT.into_iter().map(binding)),
        Spawn(reset()),
    ));
    let content = children![
        section(Icon::Mouse, "Mouse", (rows(), mouse)),
        section(Icon::Eye, "Camera", (rows(), camera)),
        section(Icon::Keyboard, "Keys on foot", (rows(), keys)),
    ];
    page(Tab::Controls, active, content)
}

/// Ship tab: the keys used while seated. Boarding and the view key are on the controls tab.
pub fn ship(active: Tab) -> impl Bundle {
    let shared = [Bind::Board, Bind::View];
    let seated = Bind::SHIP
        .into_iter()
        .filter(move |bind| !shared.contains(bind));
    let keys = Children::spawn((SpawnIter(seated.map(binding)), Spawn(reset())));
    let content = children![section(Icon::Keyboard, "Keys in the ship", (rows(), keys))];
    page(Tab::Ship, active, content)
}

/// General tab: what is neither picture, controls nor sound.
pub fn general(active: Tab) -> impl Bundle {
    let game = children![
        toggle("Check for updates", Field::CheckUpdates),
        stepper("Hitstop", Field::Hitstop),
        stepper("Markers over the world", Field::WorldMarkers),
        toggle("Blood", Field::Blood),
    ];
    let interface = children![
        stepper("Interface size", Field::Interface),
        toggle("Reduce motion", Field::ReduceMotion),
    ];
    let content = children![
        section(Icon::Planet, "Game", (rows(), game)),
        section(Icon::Monitor, "Interface", (rows(), interface)),
    ];
    page(Tab::General, active, content)
}

/// Audio tab.
pub fn audio(active: Tab) -> impl Bundle {
    let volume = children![
        stepper("Master", Field::Master),
        stepper("Music", Field::Music),
        stepper("Effects", Field::Effects),
    ];
    let output = children![
        stepper("Dynamic range", Field::Range),
        toggle("Mono", Field::Mono),
    ];
    let voices = children![stepper("Companion voice", Field::CompanionVoice)];
    let content = children![
        section(Icon::Volume, "Volume", (rows(), volume)),
        section(Icon::Volume, "Output", (rows(), output)),
        section(Icon::Volume, "Voices", (rows(), voices)),
    ];
    page(Tab::Audio, active, content)
}
