//! The climate of the selected planet in the card: the climate line under the readout and the
//! small climate map in the top right corner.
//!
//! Both follow the planet under the cursor and the step of its year, and nothing else: the
//! line is written and the map painted once per planet and step, the map on a worker thread
//! because sampling the ground of a whole planet takes a few milliseconds.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, futures_lite::future::poll_once};

use super::board::Board;
use super::climap::{BYTES, COLS, ROWS, grid, mapped};
use super::season::{climate_line, orbit_season};
use crate::space::Clock;
use crate::ui::{text, theme};
use crate::universe::BodyId;

/// Steps a year is cut into: the map and the line change at most this often in a year.
pub const STEPS: u32 = 12;

/// Pixels on the screen for one cell of the map.
const CELL: f32 = 2.0;

/// What the climate of the card is drawn for: one planet at one step of its year.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Key {
    /// The planet or moon.
    pub body: BodyId,
    /// The step of its year, from 0 to [`STEPS`].
    pub step: u32,
}

impl Key {
    /// Returns the share of the year in the middle of the step, which the map is painted for.
    pub fn season(self) -> f32 {
        (self.step as f32 + 0.5) / STEPS as f32
    }
}

/// What the card shows of the climate and what it is painting.
#[derive(Resource, Default)]
pub struct Inset {
    /// The planet and step the line is written for and the last map was started for.
    pub wanted: Option<Key>,
    /// The planet and step the map shows.
    pub shown: Option<Key>,
    /// The map being painted and what it is for.
    pub job: Option<(Key, Task<Vec<u8>>)>,
    /// The image of the map.
    pub image: Handle<Image>,
}

/// Marks the text that holds the climate line.
#[derive(Component)]
pub struct ClimateLine;

/// Marks the image of the climate map.
#[derive(Component)]
pub struct ClimateMap;

/// The climate line, under the readout.
pub fn line() -> impl Bundle {
    (
        ClimateLine,
        text("", theme::FONT_SMALL, theme::TEXT),
        TextLayout::no_wrap(),
    )
}

/// The climate map, in the top right corner of the card, hidden until it is painted.
pub fn map() -> impl Bundle {
    (
        ClimateMap,
        ImageNode::default(),
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: px(theme::GAP_S),
            right: px(0),
            width: px(COLS as f32 * CELL),
            height: px(ROWS as f32 * CELL),
            ..default()
        },
    )
}

/// Returns an empty image of the size of the map, drawn cell by cell.
fn blank() -> Image {
    let size = Extent3d {
        width: COLS as u32,
        height: ROWS as u32,
        depth_or_array_layers: 1,
    };
    let pixels = vec![0; COLS * ROWS * BYTES];
    let (format, usage) = (TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    let mut image = Image::new(size, TextureDimension::D2, pixels, format, usage);
    image.sampler = ImageSampler::nearest();
    image
}

/// Gives a new map its image and forgets what the card showed, so that it is drawn again.
pub fn fresh(
    mut inset: ResMut<Inset>,
    mut images: Option<ResMut<Assets<Image>>>,
    mut maps: Query<&mut ImageNode, Added<ClimateMap>>,
) {
    for mut node in &mut maps {
        if let Some(images) = images.as_deref_mut() {
            inset.image = images.add(blank());
        }
        node.image = inset.image.clone();
        (inset.wanted, inset.shown, inset.job) = (None, None, None);
    }
}

/// Writes the climate line and starts painting the map when the planet under the cursor or the
/// step of its year changed. A planet whose system was not visited gets neither.
pub fn plan(
    board: Res<Board>,
    clock: Option<Res<Clock>>,
    mut inset: ResMut<Inset>,
    mut lines: Query<&mut Text, With<ClimateLine>>,
) {
    let time = clock.map_or(0.0, |clock| clock.seconds);
    let world = board.worlds.get(board.cursor).and_then(Option::as_ref);
    let key = world.map(|spec| Key {
        body: spec.id,
        step: (orbit_season(spec, time) * STEPS as f32) as u32 % STEPS,
    });
    if key == inset.wanted {
        return;
    }
    inset.wanted = key;
    let world = world.zip(key);
    let written = world.map_or_else(String::new, |(spec, key)| climate_line(spec, key.season()));
    for mut text in &mut lines {
        text.set_if_neq(Text(written.clone()));
    }
    inset.job = world.filter(|(spec, _)| mapped(spec)).map(|(spec, key)| {
        let spec = spec.clone();
        (
            key,
            AsyncComputeTaskPool::get().spawn(async move { grid(&spec, key.season()) }),
        )
    });
}

/// Takes the map a worker finished into the image, and shows the map while it is of the planet
/// under the cursor.
pub fn collect(
    mut inset: ResMut<Inset>,
    images: Option<ResMut<Assets<Image>>>,
    mut maps: Query<&mut Node, With<ClimateMap>>,
) {
    let inset = inset.bypass_change_detection();
    let done = inset
        .job
        .as_mut()
        .and_then(|job| block_on(poll_once(&mut job.1)));
    if let Some(pixels) = done {
        inset.shown = inset.job.take().map(|job| job.0);
        let image = images.and_then(|images| images.into_inner().get_mut(&inset.image));
        if let Some(mut image) = image {
            image.data = Some(pixels);
        }
    }
    let same = inset
        .shown
        .zip(inset.wanted)
        .is_some_and(|(a, b)| a.body == b.body);
    let display = if same { Display::Flex } else { Display::None };
    for mut node in &mut maps {
        if node.display != display {
            node.display = display;
        }
    }
}
