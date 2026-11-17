//! Checks of the generated window icon and of its plugin.

use super::*;

/// Columns and rows that hold a white pixel, as (left, right, top, bottom, count).
fn white_bounds(rgba: &[u8]) -> (u32, u32, u32, u32, u32) {
    let mut bounds = (SIZE, 0, SIZE, 0, 0);
    for (index, pixel) in rgba.as_chunks::<4>().0.iter().enumerate() {
        if *pixel == [255; 4] {
            let (x, y) = (index as u32 % SIZE, index as u32 / SIZE);
            bounds = (
                bounds.0.min(x),
                bounds.1.max(x),
                bounds.2.min(y),
                bounds.3.max(y),
                bounds.4 + 1,
            );
        }
    }
    bounds
}

/// The buffer holds one RGBA pixel per cell of the tile.
#[test]
fn icon_has_the_documented_dimensions() {
    assert_eq!(pixels().len(), (SIZE * SIZE * 4) as usize);
}

/// Only opaque black and opaque white appear.
#[test]
fn icon_uses_exactly_two_colours() {
    let rgba = pixels();
    let mut colours = rgba.as_chunks::<4>().0.to_vec();
    colours.sort_unstable();
    colours.dedup();
    assert_eq!(colours, [[0, 0, 0, 255], [255, 255, 255, 255]]);
}

/// The white pixels form one filled square with the same black margin on all four sides.
#[test]
fn icon_square_is_centred() {
    let (left, right, top, bottom, count) = white_bounds(&pixels());
    assert_eq!((left, top), (PADDING, PADDING));
    assert_eq!((SIZE - 1 - right, SIZE - 1 - bottom), (PADDING, PADDING));
    assert_eq!(count, SQUARE * SQUARE);
}

/// Every side keeps at least the documented share of the width in black.
#[test]
fn icon_padding_is_at_least_the_documented_share() {
    let (left, right, top, bottom, _) = white_bounds(&pixels());
    let smallest = left.min(top).min(SIZE - 1 - right).min(SIZE - 1 - bottom);
    assert!(
        smallest as f32 >= SIZE as f32 * PADDING_SHARE,
        "padding is {smallest} px"
    );
}

/// The pixels equal the 64 px bitmap stored in the Windows icon file.
#[test]
fn icon_matches_the_packaged_windows_tile() {
    let ico = include_bytes!("../../packaging/windows/abstractions.ico");
    let count = u16::from_le_bytes([ico[4], ico[5]]) as usize;
    let entry = (0..count)
        .map(|n| &ico[6 + n * 16..22 + n * 16])
        .find(|entry| u32::from(entry[0]) == SIZE);
    let entry = entry.expect("the ico holds a 64 px image");
    let offset = u32::from_le_bytes([entry[12], entry[13], entry[14], entry[15]]) as usize + 40;
    assert_eq!(
        &ico[offset..offset + (SIZE * SIZE * 4) as usize],
        pixels().as_slice()
    );
}

/// Without a window the plugin runs and sets nothing.
#[test]
fn icon_plugin_does_nothing_without_a_window() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins).add_plugins(plugin);
    (0..3).for_each(|_| app.update());
    assert!(
        app.world_mut()
            .query::<&Window>()
            .iter(app.world())
            .next()
            .is_none()
    );
}
