//! The wake-up sequence: the robot's boot screen, shown over the loaded world before it fades in.
//!
//! A new world gets the full sequence. A resumed one gets a single line. Any key skips it.

use bevy::input::mouse::MouseButton;
use bevy::prelude::*;
use bevy::text::LetterSpacing;

use super::loading::Arrival;
use super::wake::{FADE, STATUS_DELAY, line_time, script, timeline};
use super::{Screen, WarmUp};
use crate::render::Warmed;
use crate::ui::{cell, root_on, text, theme, warm_up};
use crate::world::Calm;

/// Width of the label column of the boot lines, in pixels.
const LABEL_WIDTH: f32 = 250.0;

/// Seconds after which the boot screen lifts even if the world around the player is not complete.
const HOLD_LIMIT: f32 = 20.0;

/// The clock of the boot screen.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Boot {
    /// Seconds since the boot screen appeared.
    clock: f32,
    /// Second at which the fade to the world starts.
    fade_at: f32,
}

/// Marks the boot screen.
#[derive(Component)]
struct BootScreen;

/// Marks the block of lines and the skip hint, which go dark when the fade starts.
#[derive(Component)]
struct Lines;

/// A text that appears this many seconds after the boot screen.
#[derive(Component, Debug, Clone, Copy)]
struct Appears(f32);

/// One boot line: its label, a dotted lead and its status word, each hidden until its time.
fn line((index, (label, status)): (usize, (&'static str, String))) -> impl Bundle {
    let at = line_time(index);
    let spaced = || (TextLayout::no_wrap(), LetterSpacing::Px(2.0));
    (
        Node {
            column_gap: px(theme::GAP_M),
            ..default()
        },
        Appears(at),
        Visibility::Hidden,
        children![
            cell(
                LABEL_WIDTH,
                (text(label, theme::FONT_BODY, theme::TEXT_2), spaced())
            ),
            text("...", theme::FONT_BODY, theme::TEXT_2),
            (
                text(status, theme::FONT_BODY, theme::TEXT),
                spaced(),
                Appears(at + STATUS_DELAY),
                Visibility::Hidden,
            ),
        ],
    )
}

/// The hint that any key skips the sequence, in the bottom-left corner.
fn hint() -> impl Bundle {
    (
        Lines,
        text("Press any key to skip", theme::FONT_SMALL, theme::TEXT_2),
        Node {
            position_type: PositionType::Absolute,
            left: percent(5),
            bottom: px(theme::EDGE),
            ..default()
        },
    )
}

/// Covers the world with the boot screen and starts its clock.
fn enter(mut commands: Commands, arrival: Option<Res<Arrival>>) {
    let fallback = Arrival::of(None, 0);
    let lines = script(arrival.as_deref().unwrap_or(&fallback));
    let (fade_at, _) = timeline(lines.len());
    commands.insert_resource(Boot {
        clock: 0.0,
        fade_at,
    });
    commands.spawn((
        BootScreen,
        DespawnOnExit(WarmUp::Hidden),
        root_on(theme::BACKGROUND, JustifyContent::Center),
        GlobalZIndex(50),
        children![
            (
                Lines,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: px(theme::GAP_S),
                    ..default()
                },
                Children::spawn(SpawnIter(lines.into_iter().enumerate().map(line))),
            ),
            hint(),
            warm_up(),
        ],
    ));
}

/// The keyboard and the mouse buttons, either of which skips the sequence.
type Buttons<'w> = (
    Res<'w, ButtonInput<KeyCode>>,
    Option<Res<'w, ButtonInput<MouseButton>>>,
);

/// What the fade waits for: every pipeline drawn once and the world around the player complete.
type Readiness<'w> = (Option<Res<'w, Warmed>>, Option<Res<'w, Calm>>);

/// Advances the clock, jumps to the fade on any key or click, and hands the world over when
/// the fade ends. The fade does not start before the warm-up has run and the streaming has caught
/// up, or [`HOLD_LIMIT`] seconds have passed.
fn advance(
    time: Res<Time>,
    (keys, mouse): Buttons,
    (warmed, calm): Readiness,
    mut boot: ResMut<Boot>,
    mut warm: ResMut<NextState<WarmUp>>,
) {
    let clicked = mouse.is_some_and(|mouse| mouse.get_just_pressed().next().is_some());
    let skip = keys.get_just_pressed().next().is_some() || clicked;
    boot.clock += time.delta_secs();
    if skip && boot.clock < boot.fade_at {
        boot.fade_at = boot.clock;
    }
    let ready = warmed.is_none_or(|w| w.done) && calm.is_none_or(|c| c.0);
    if !ready && boot.clock < HOLD_LIMIT && boot.clock >= boot.fade_at {
        boot.fade_at = boot.clock;
    }
    if boot.clock >= boot.fade_at + FADE {
        warm.set(WarmUp::Idle);
    }
}

/// Shows each text once its time has come, and hides them all when the fade starts.
fn reveal(
    boot: Res<Boot>,
    mut texts: Query<(&Appears, &mut Visibility), Without<Lines>>,
    mut blocks: Query<&mut Visibility, With<Lines>>,
) {
    for (appears, mut visibility) in &mut texts {
        if boot.clock >= appears.0 {
            visibility.set_if_neq(Visibility::Inherited);
        }
    }
    if boot.clock >= boot.fade_at {
        for mut visibility in &mut blocks {
            visibility.set_if_neq(Visibility::Hidden);
        }
    }
}

/// Fades the dark background out over the world.
fn fade(boot: Res<Boot>, mut screens: Query<&mut BackgroundColor, With<BootScreen>>) {
    let done = ((boot.clock - boot.fade_at) / FADE).clamp(0.0, 1.0);
    for mut background in &mut screens {
        background.set_if_neq(BackgroundColor(theme::BACKGROUND.with_alpha(1.0 - done)));
    }
}

/// Uncovers the world at once when the title screen returns.
fn abort(mut warm: ResMut<NextState<WarmUp>>) {
    warm.set(WarmUp::Idle);
}

/// Registers the boot screen systems.
pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Boot>()
        .add_systems(OnEnter(WarmUp::Hidden), enter)
        .add_systems(OnEnter(Screen::Title), abort)
        .add_systems(
            Update,
            (advance, reveal, fade)
                .chain()
                .run_if(in_state(WarmUp::Hidden)),
        );
}
