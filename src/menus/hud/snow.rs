//! Static over the view: a hurt head loses the picture to bars of noise and, now and then, to a
//! dropout that blacks the whole screen for a moment.

use bevy::prelude::*;

use crate::avatar::hurt::eye;
use crate::condition::{Condition, Part, luck};

/// Number of bars of noise.
const BARS: usize = 8;

/// Noise changes this many times a second.
const RATE: f32 = 20.0;

/// Share of the bars a head at zero health shows at once, unitless.
const SHOWN: f32 = 0.9;

/// Opacity of a dropout, which blacks the screen almost whole.
const BLACKOUT: f32 = 0.92;

/// Brightest a bar of noise gets, as opacity.
const BAR: f32 = 0.35;

/// Dimmest a dropout of the eye gets: under this share of light the screen counts as dropped out.
const DROPPED: f32 = 0.1;

/// Marks the cover that blacks the screen in a dropout.
#[derive(Component)]
pub(super) struct Cover;

/// Marks a bar of noise, by number.
#[derive(Component)]
pub(super) struct Bar(usize);

/// The full screen cover and the bars of noise over it, all clear while the head is whole.
pub(super) fn screen() -> impl Bundle {
    (
        Cover,
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            ..default()
        },
        BackgroundColor(Color::NONE),
        Pickable::IGNORE,
        Children::spawn(bevy::ecs::spawn::SpawnIter((0..BARS).map(bar))),
    )
}

/// One clear bar of noise.
fn bar(at: usize) -> impl Bundle {
    (
        Bar(at),
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(2),
            ..default()
        },
        BackgroundColor(Color::NONE),
        Pickable::IGNORE,
    )
}

/// Returns the colour of the bar `at` of the noise at `time` seconds for a head of health `head`:
/// white or black at a random strength, or clear.
pub(super) fn tone(at: usize, head: f32, time: f32) -> Color {
    let key = ((time * RATE) as u32)
        .wrapping_mul(BARS as u32)
        .wrapping_add(at as u32);
    let weak = 1.0 - head.clamp(0.0, 1.0);
    if luck(key) >= weak * SHOWN {
        return Color::NONE;
    }
    let grey = if luck(key ^ 0x51) < 0.5 { 0.0 } else { 1.0 };
    Color::srgba(grey, grey, grey, BAR * luck(key ^ 0xa7))
}

/// Draws the noise and the dropouts of the head over the screen.
pub(super) fn update(
    (time, condition): (Res<Time>, Res<Condition>),
    mut cover: Query<&mut BackgroundColor, (With<Cover>, Without<Bar>)>,
    mut bars: Query<(&Bar, &mut Node, &mut BackgroundColor)>,
) {
    let (head, now) = (condition.of(Part::Head), time.elapsed_secs());
    for mut shade in &mut cover {
        let out = eye::dim(head, now) < DROPPED;
        shade.set_if_neq(BackgroundColor(match out {
            true => Color::srgba(0.0, 0.0, 0.0, BLACKOUT),
            false => Color::NONE,
        }));
    }
    for (Bar(at), mut node, mut shade) in &mut bars {
        let key = ((now * RATE) as u32)
            .wrapping_mul(BARS as u32)
            .wrapping_add(*at as u32);
        node.top = percent(luck(key ^ 0x33) * 98.0);
        shade.set_if_neq(BackgroundColor(tone(*at, head, now)));
    }
}
