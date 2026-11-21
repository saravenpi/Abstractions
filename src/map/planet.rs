//! The planet level of the map: the flat map of one planet on its sheet, its picture tiled so
//! the map wraps east and west.
//!
//! The flat map is the coloured picture of the planet the raster paints, from
//! [`super::raster::Rasters`], clear where nobody has been, so a dim tone of the panel shows
//! there. Until a picture is ready the same dim tone stands in for it. A sentence over the map
//! says why it is dim: the picture is being drawn, or nothing is explored yet.

use bevy::prelude::*;

use super::explored::Explored;
use super::level::Level;
use super::raster::{Rasters, tile_layer};
use super::surface::PlanetSheet;
use crate::computer::ChartLayer;
use crate::ui::{text, theme};

/// Tone of the flat map where its picture is clear: while it is not painted yet, and wherever
/// nobody has been. A faint light on the translucent panel, so the shape of the planet shows
/// and the blurred game still shows through.
const BLANK: Color = Color::srgba(1.0, 1.0, 1.0, 0.08);

/// What the map says while its picture is not painted yet.
pub const DRAWING: &str = "Drawing the map...";

/// What the map says while the picture is there and nothing of the planet is explored.
pub const UNEXPLORED: &str = "Nothing explored here yet. Walk or fly over the ground to chart it.";

/// Marks the box over the flat map that holds the sentence about its picture.
#[derive(Component)]
pub struct MapNote;

/// Marks the sentence about the picture of the flat map.
#[derive(Component)]
pub struct NoteText;

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

/// Returns what the map says about its picture: it is being drawn, nothing is explored, or
/// nothing at all.
pub fn note_words(drawn: bool, explored: bool) -> &'static str {
    match (drawn, explored) {
        (false, _) => DRAWING,
        (true, false) => UNEXPLORED,
        (true, true) => "",
    }
}

/// The box over the middle of the flat map that holds its sentence on a panel, hidden while the
/// sentence is empty. It lets the pointer through.
fn note() -> impl Bundle {
    let cover = Node {
        display: Display::None,
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };
    let words = Node {
        max_width: px(crate::computer::ROOM),
        padding: UiRect::axes(px(theme::S3), px(theme::S2)),
        ..default()
    };
    let said = (NoteText, words, BackgroundColor(theme::PANEL));
    (
        MapNote,
        cover,
        children![(said, text("", theme::FONT_SMALL, theme::TEXT_2))],
    )
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
        children![
            (
                PlanetSheet,
                cover,
                children![tile(-1), tile(0), tile(1), tile_layer()]
            ),
            note()
        ],
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

/// Writes the sentence about the picture of the planet shown, and shows its box only while it
/// says something.
pub(super) fn say(
    level: Res<Level>,
    (rasters, explored): (Res<Rasters>, Option<Res<Explored>>),
    mut boxes: Query<&mut Node, With<MapNote>>,
    mut texts: Query<&mut Text, With<NoteText>>,
) {
    let Level::Planet(body) = *level else {
        return;
    };
    let seen = explored.is_some_and(|explored| explored.version(body) > 0);
    let said = note_words(rasters.image(body).is_some(), seen);
    for mut text in &mut texts {
        text.set_if_neq(Text(said.to_string()));
    }
    let display = [Display::Flex, Display::None][usize::from(said.is_empty())];
    for mut node in &mut boxes {
        if node.display != display {
            node.display = display;
        }
    }
}
