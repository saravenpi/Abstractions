//! Opening a placed chest on interact (`F`) and closing the overlay on Escape, Tab, or Pack.

use abstractions_net::chest::PlaceableKind;
use bevy::prelude::*;

use super::show::PropEntity;
use super::{ChestScreen, ChestWorld, OpenChest};
use crate::audio::AudioCue;
use crate::menus::Menu;
use crate::player::{Interact, Player, Verb};
use crate::render::GameCamera;
use crate::screens::Screen;
use crate::settings::{Persistent, Settings};

/// Maximum player distance in metres to open a placed chest.
const OPEN_REACH: f32 = 4.0;

/// Maximum lateral distance in metres from the camera aim ray to select a chest.
const OPEN_AIM: f32 = 0.75;

/// Returns the placed chest id closest to the camera sight line within [`OPEN_REACH`].
fn aimed_chest(
    eye: &GlobalTransform,
    me: Vec3,
    props: &Query<(&PropEntity, &GlobalTransform)>,
) -> Option<u64> {
    let (from, dir) = (eye.translation(), eye.forward().as_vec3());
    let miss = |at: Vec3| {
        let along = (at - from).dot(dir);
        (along > 0.0).then(|| (at - from - dir * along).length())
    };
    props
        .iter()
        .filter(|(p, at)| {
            p.kind == PlaceableKind::Chest && at.translation().distance(me) <= OPEN_REACH
        })
        .filter_map(|(p, at)| Some((miss(at.translation())?, p.id)))
        .filter(|(m, _)| *m <= OPEN_AIM)
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, id)| id)
}

/// Camera and player transform queries for chest aiming.
type Viewers<'w, 's> = (
    Query<'w, 's, &'static GlobalTransform, With<GameCamera>>,
    Query<'w, 's, &'static Transform, With<Player>>,
);

/// Active gameplay screen and menu states.
type AppScreens<'w> = (Option<Res<'w, State<Screen>>>, Option<Res<'w, State<Menu>>>);

/// Offers [`Verb::Open`] on the aimed chest and opens [`ChestScreen::Open`] when interact is pressed.
pub(super) fn aim_and_open(
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    (mut interact, mut open, mut next): (
        ResMut<Interact>,
        ResMut<OpenChest>,
        ResMut<NextState<ChestScreen>>,
    ),
    (camera, player): Viewers,
    props: Query<(&PropEntity, &GlobalTransform)>,
    mut cues: MessageWriter<AudioCue>,
) {
    let (Ok(eye), Ok(me)) = (camera.single(), player.single()) else {
        return;
    };
    let Some(id) = aimed_chest(eye, me.translation, &props) else {
        return;
    };
    interact.offer(Verb::Open, "chest");
    let key = settings.controls.keys.ship.board;
    if !interact.pressed(Verb::Open, &keys, key) {
        return;
    }
    open.id = Some(id);
    open.selected = None;
    next.set(ChestScreen::Open);
    cues.write(AudioCue::UiConfirm);
}

/// Closes [`ChestScreen::Open`] on Escape, Tab, Pack key, pause menu, or if the chest vanished.
pub(super) fn close(
    (keys, settings): (Res<ButtonInput<KeyCode>>, Res<Persistent<Settings>>),
    (screen, menu): AppScreens,
    (world, open): (Res<ChestWorld>, Res<OpenChest>),
    mut next: ResMut<NextState<ChestScreen>>,
    mut cues: MessageWriter<AudioCue>,
) {
    let playing = screen.is_some_and(|s| *s.get() == Screen::Gameplay)
        && menu.is_none_or(|m| *m.get() == Menu::None);
    let gone = open.id.is_none_or(|id| world.chests.find(id).is_none());
    let key = keys.just_pressed(KeyCode::Escape)
        || keys.just_pressed(KeyCode::Tab)
        || keys.just_pressed(settings.controls.keys.pack);
    if !playing || gone || key {
        next.set(ChestScreen::Closed);
        cues.write(AudioCue::UiCancel);
    }
}

/// Clears the active chest handle and slot selection when leaving [`ChestScreen::Open`].
pub(super) fn clear(mut open: ResMut<OpenChest>) {
    open.id = None;
    open.selected = None;
}
