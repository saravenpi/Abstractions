//! Showing the tiles over the picture of the planet. Each tile is laid in the frame of the map
//! itself, from the gap of its corners to the centre of the view worked out in f64, never in the
//! frame of the sheet: at the closest zoom the sheet is tens of millions of pixels wide, and a
//! place on it in f32 is off by pixels.

use bevy::math::DVec2;
use bevy::prelude::*;

use super::cover::Ready;
use super::tile::TileId;
use super::tiles::Tiles;
use crate::computer::SIZE;
use crate::map::Surface;
use crate::map::sight::Sight;

/// Marks the layer of the map that holds the tiles, laid over the picture.
#[derive(Component)]
pub struct TileLayer;

/// Marks a tile shown, with the tile and the picture it shows.
#[derive(Component)]
pub(super) struct Shown(i64, TileId, Handle<Image>);

/// Returns the layer that holds the tiles: the size of the map, over the sheet of the picture.
pub fn tile_layer() -> impl Bundle {
    let node = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    (TileLayer, node)
}

/// Returns the chart point of the north-west corner of the tile in `column`, counted without
/// wrapping, and `row` of `level`.
pub fn corner(level: u8, column: i64, row: u32) -> DVec2 {
    let (cols, rows) = (
        f64::from(TileId::cols(level)),
        f64::from(TileId::rows(level)),
    );
    DVec2::new(
        column as f64 / cols * 2.0 - 1.0,
        0.5 - f64::from(row) / rows,
    )
}

/// Returns the node of one tile, laid where `lens` draws its place on the map.
pub fn node(column: i64, id: TileId, lens: Sight) -> Node {
    let west = lens.to_map(corner(id.level, column, id.row)) / SIZE * 100.0;
    let east = lens.to_map(corner(id.level, column + 1, id.row + 1)) / SIZE * 100.0;
    Node {
        position_type: PositionType::Absolute,
        left: percent(west.x),
        top: percent(west.y),
        width: percent(east.x - west.x),
        height: percent(east.y - west.y),
        ..default()
    }
}

/// Returns true when `tile` shows the tile `ready`.
fn same((col, id, image): &Ready, tile: &Shown) -> bool {
    (*col, *id, image) == (tile.0, tile.1, &tile.2)
}

/// Moves every tile shown to where `lens` draws it, and despawns those no longer `ready`.
fn follow(
    ready: &[Ready],
    lens: Sight,
    shown: &mut Query<(Entity, &Shown, &mut Node)>,
    commands: &mut Commands,
) {
    for (entity, tile, mut place) in shown.iter_mut() {
        if !ready.iter().any(|ready| same(ready, tile)) {
            commands.entity(entity).despawn();
            continue;
        }
        let wanted = node(tile.0, tile.1, lens);
        if *place != wanted {
            *place = wanted;
        }
    }
}

/// Shows the tiles that cover the view where the lens draws them, each finer level over the
/// coarser ones, moves them with the lens, and hides the others.
pub(super) fn show(
    (tiles, surface): (Res<Tiles>, Res<Surface>),
    layers: Query<Entity, With<TileLayer>>,
    mut shown: Query<(Entity, &Shown, &mut Node)>,
    mut commands: Commands,
) {
    if !(tiles.is_changed() || surface.is_changed()) {
        return;
    }
    let lens = surface.lens;
    let (ready, _) = tiles.ready(lens);
    follow(&ready, lens, &mut shown, &mut commands);
    let missing: Vec<&Ready> = ready
        .iter()
        .filter(|ready| !shown.iter().any(|(_, tile, _)| same(ready, tile)))
        .collect();
    for layer in &layers {
        for (col, id, image) in missing.iter().copied() {
            let tile = (Shown(*col, *id, image.clone()), node(*col, *id, lens));
            let above = ZIndex(i32::from(id.level));
            commands
                .entity(layer)
                .with_child((tile, above, ImageNode::new(image.clone())));
        }
    }
}
