//! Showing the tiles on the sheet of the flat map, over the picture of the planet.

use bevy::prelude::*;

use super::tile::TileId;
use super::tiles::Tiles;

/// Marks the layer of the sheet that holds the tiles, laid over the picture.
#[derive(Component)]
pub struct TileLayer;

/// Marks a tile shown, with the tile and the picture it shows.
#[derive(Component)]
pub(super) struct Shown(i64, TileId, Handle<Image>);

/// Returns the layer that holds the tiles: as large as the picture and moving as its copy under
/// the lens does.
pub fn tile_layer() -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        top: percent(25),
        width: percent(100),
        height: percent(50),
        ..default()
    };
    (TileLayer, crate::map::planet::Tile(0), node)
}

/// Returns the node of one tile, laid on its place in the picture.
fn node(column: i64, id: TileId) -> Node {
    let (cols, rows) = (TileId::cols(id.level) as f32, TileId::rows(id.level) as f32);
    Node {
        position_type: PositionType::Absolute,
        left: percent(column as f32 / cols * 100.0),
        top: percent(id.row as f32 / rows * 100.0),
        width: percent(100.0 / cols),
        height: percent(100.0 / rows),
        ..default()
    }
}

/// Shows the tiles that are ready and hides the others.
pub(super) fn show(
    tiles: Res<Tiles>,
    layers: Query<Entity, With<TileLayer>>,
    shown: Query<(Entity, &Shown)>,
    mut commands: Commands,
) {
    if !tiles.is_changed() {
        return;
    }
    let ready = tiles.ready();
    let wanted = |shown: &Shown| {
        ready
            .iter()
            .any(|(col, id, image)| (*col, *id, image) == (shown.0, shown.1, &shown.2))
    };
    for (entity, tile) in &shown {
        if !wanted(tile) {
            commands.entity(entity).despawn();
        }
    }
    let held: Vec<&Shown> = shown.iter().map(|(_, tile)| tile).collect();
    for layer in &layers {
        for (col, id, image) in &ready {
            let drawn = held
                .iter()
                .any(|tile| (tile.0, tile.1, &tile.2) == (*col, *id, image));
            if !drawn {
                let tile = (Shown(*col, *id, image.clone()), node(*col, *id));
                commands
                    .entity(layer)
                    .with_child((tile, ImageNode::new(image.clone())));
            }
        }
    }
}
