//! The planet level of the map: the flat map of one planet on its sheet, the caption that names
//! the level shown, and the latitude and longitude under the pointer.
//!
//! The flat map is the coloured picture of the planet the raster paints, black where nobody
//! has been, from [`super::raster::Rasters`]. Until a picture is ready a plain dark rectangle
//! stands in for it.

use bevy::prelude::*;

use super::geo::coords as words;
use super::level::Level;
use super::raster::Rasters;
use super::surface::{PlanetSheet, Surface};
use crate::computer::{Board, ChartLayer, Tab, climate_line};
use crate::space::Clock;
use crate::ui::{text, theme};
use crate::universe::Universe;

/// Colour of the flat map while its picture is not painted yet.
const BLANK: Color = Color::srgb(0.08, 0.09, 0.11);

/// Marks the layer of the planet, shown only on the planet level.
#[derive(Component)]
pub struct PlanetLayer;

/// Marks the picture of the planet on its sheet.
#[derive(Component)]
pub struct Picture;

/// Marks the line that names the level shown.
#[derive(Component)]
pub struct Caption;

/// Marks the line that says the latitude and longitude under the pointer.
#[derive(Component)]
pub struct Coords;

/// The hidden layer of the planet: the sheet the lens moves, holding the picture in its middle
/// half, twice as wide as it is high.
pub fn planet_layer() -> impl Bundle {
    let cover = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    let picture = Node {
        position_type: PositionType::Absolute,
        left: percent(0),
        top: percent(25),
        width: percent(100),
        height: percent(50),
        ..default()
    };
    let layer = Node {
        display: Display::None,
        ..cover.clone()
    };
    (
        PlanetLayer,
        layer,
        children![(
            PlanetSheet,
            cover,
            children![(Picture, picture, BackgroundColor(BLANK))]
        )],
    )
}

/// The line that names the level shown.
pub fn caption() -> impl Bundle {
    (
        Caption,
        text("", theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
    )
}

/// The line that says the latitude and longitude under the pointer.
pub fn coords() -> impl Bundle {
    (
        Coords,
        text("", theme::FONT_SMALL, theme::MUTED),
        TextLayout::no_wrap(),
    )
}

/// The two layers the level shows one of.
type Layers<'w, 's> =
    Query<'w, 's, (&'static mut Node, Has<PlanetLayer>), Or<(With<PlanetLayer>, With<ChartLayer>)>>;

/// Shows the layer of the level shown and hides the other, when the level changes or the
/// screen was just spawned.
pub(super) fn show(level: Res<Level>, fresh: Query<(), Added<PlanetLayer>>, mut layers: Layers) {
    if !(level.is_changed() || !fresh.is_empty()) {
        return;
    }
    let planet = matches!(*level, Level::Planet(_));
    for (mut node, is_planet) in &mut layers {
        let display = [Display::None, Display::Flex][usize::from(planet == is_planet)];
        if node.display != display {
            node.display = display;
        }
    }
}

/// Asks for the picture of the planet shown and puts it on the map once it is painted. The
/// plain rectangle stands in until then, also after a switch to a planet not painted yet.
pub(super) fn picture(
    level: Res<Level>,
    mut rasters: ResMut<Rasters>,
    mut commands: Commands,
    pictures: Query<(Entity, Option<&ImageNode>), With<Picture>>,
) {
    let Level::Planet(body) = *level else {
        return;
    };
    let handle = rasters.image(body);
    if handle.is_none() {
        rasters.request(body);
    }
    for (entity, node) in &pictures {
        match (&handle, node) {
            (Some(handle), Some(node)) if node.image == *handle => {}
            (Some(handle), _) => {
                commands
                    .entity(entity)
                    .insert(ImageNode::new(handle.clone()));
            }
            (None, Some(_)) => {
                commands.entity(entity).remove::<ImageNode>();
            }
            (None, None) => {}
        }
    }
}

/// Returns the caption of the level: the planet shown with its climate at `time` seconds, or
/// the chart of the open tab.
pub fn title(level: Level, tab: Tab, (universe, time): (Option<&Universe>, f64)) -> String {
    match (level, universe) {
        (Level::Planet(body), Some(universe)) => {
            let planet = universe.planet(body);
            let climate = climate_line(&planet, crate::climate::season(&planet, time));
            format!("{} - {climate}", planet.name)
        }
        (Level::Planet(_), None) => "Planet".to_string(),
        (Level::Chart, _) => format!("{} chart", tab.label()),
    }
}

/// Writes the caption when the level or the tab changes.
pub(super) fn caption_text(
    (level, board): (Res<Level>, Res<Board>),
    (universe, clock): (Option<Res<Universe>>, Option<Res<Clock>>),
    mut texts: Query<&mut Text, With<Caption>>,
    mut last: Local<Option<(Level, Tab)>>,
) {
    let now = (*level, board.tab);
    if *last == Some(now) && texts.iter().all(|text| !text.0.is_empty()) {
        return;
    }
    *last = Some(now);
    let time = clock.map_or(0.0, |clock| clock.seconds);
    let said = title(now.0, now.1, (universe.as_deref(), time));
    for mut text in &mut texts {
        text.set_if_neq(Text(said.clone()));
    }
}

/// Writes the latitude and longitude under the pointer on a planet, and nothing elsewhere.
pub(super) fn coords_text(
    surface: Res<Surface>,
    level: Res<Level>,
    mut texts: Query<&mut Text, With<Coords>>,
) {
    if !(surface.is_changed() || level.is_changed()) {
        return;
    }
    let point = surface
        .point
        .filter(|_| matches!(*level, Level::Planet(_)) && surface.over);
    let said = point.map_or(String::new(), words);
    for mut text in &mut texts {
        text.set_if_neq(Text(said.clone()));
    }
}
