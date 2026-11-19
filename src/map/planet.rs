//! The planet level of the map: the flat map of one planet on its sheet, its picture tiled so
//! the map wraps east and west.
//!
//! The flat map is the coloured picture of the planet the raster paints, black where nobody
//! has been, from [`super::raster::Rasters`]. Until a picture is ready a plain dark rectangle
//! stands in for it.

use bevy::prelude::*;

use super::level::Level;
use super::raster::Rasters;
use super::surface::PlanetSheet;
use crate::computer::ChartLayer;

/// Colour of the flat map while its picture is not painted yet.
const BLANK: Color = Color::srgb(0.08, 0.09, 0.11);

/// Marks the layer of the planet, shown only on the planet level.
#[derive(Component)]
pub struct PlanetLayer;

/// Marks the picture of the planet on its sheet.
#[derive(Component)]
pub struct Picture;

/// Which copy of the picture a node is, counted in turns east of the copy under the lens.
#[derive(Component, Debug, Clone, Copy)]
pub struct Tile(pub i8);

/// One copy of the picture, `east` turns east of the one under the lens, in the middle half
/// of the sheet.
fn tile(east: i8) -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        left: percent(f32::from(east) * 100.0),
        top: percent(25),
        width: percent(100),
        height: percent(50),
        ..default()
    };
    (Picture, Tile(east), node, BackgroundColor(BLANK))
}

/// The hidden layer of the planet: the sheet the lens moves, holding the picture in its middle
/// half, twice as wide as it is high, with a copy on each side so the map wraps east and west.
pub fn planet_layer() -> impl Bundle {
    let cover = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    let layer = Node {
        display: Display::None,
        ..cover.clone()
    };
    (
        PlanetLayer,
        layer,
        children![(PlanetSheet, cover, children![tile(-1), tile(0), tile(1)])],
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
