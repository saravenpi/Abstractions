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
    (
        page(Tab::Graphics, active),
        children![
            stepper("Pixel size", Field::PixelSize),
            stepper("Render distance", Field::RenderDistance),
            toggle("Shadows", Field::Shadows),
            toggle("Vsync", Field::Vsync),
            toggle("Fullscreen", Field::Fullscreen),
            toggle("Ambient occlusion", Field::AmbientOcclusion),
            toggle("Bloom", Field::Bloom),
            toggle("Clouds", Field::Clouds),
            toggle("Show FPS", Field::ShowFps),
            toggle("Ray tracing (experimental)", Field::RayTracing),
            stepper("Color depth", Field::ColorLevels),
            stepper("Field of view", Field::Fov),
        ],
    )
}

/// Controls tab: the mouse and the keys used on foot.
pub fn controls(active: Tab) -> impl Bundle {
    (
        page(Tab::Controls, active),
        Children::spawn((
            Spawn(stepper("Mouse sensitivity", Field::Sensitivity)),
            Spawn(toggle("Invert Y", Field::InvertY)),
            SpawnIter(Bind::FOOT.into_iter().map(binding)),
        )),
    )
}

/// Ship tab: the keys used while seated. Boarding is on the controls tab.
pub fn ship(active: Tab) -> impl Bundle {
    let seated = Bind::SHIP.into_iter().filter(|bind| *bind != Bind::Board);
    (
        page(Tab::Ship, active),
        Children::spawn(SpawnIter(seated.map(binding))),
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
