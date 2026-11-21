//! The animals the robot faces: the name of each over a bar of its health, straight over the world
//! above its head. The animal fought carries one while the fight lasts, and the living animal the
//! crosshair points at within a short range, or one standing close by, carries another. Each fades
//! in and out over [`FADE`] seconds, follows its animal on screen, hides when it is off screen or
//! behind the camera, and never sits over the crosshair.

mod pick;
mod place;
#[cfg(test)]
mod tests_pick;
#[cfg(test)]
mod tests_shown;

pub use pick::{labels, looked};
pub use place::{hung, over};

use bevy::prelude::*;

use super::context::Element;
use super::fade::gate;
use crate::avatar::strike::Mark;
use crate::fauna::{Beast, Live};
use crate::ui::{OverWorld, text, theme};
use crate::world::Terrain;
use pick::RANGE;
use place::{Bounds, Lens, sight, top};

/// Width of the health bar and its height, in pixels.
const BAR: (f32, f32) = (64.0, 4.0);

/// Height of the strip, the name over the bar, in pixels.
const STRIP: f32 = 28.0;

/// Strips at most on screen: the animal fought and the one looked at.
pub const SLOTS: usize = 2;

/// Seconds a strip takes to fade in or out.
pub const FADE: f32 = 0.2;

/// Marks the strip of slot `0`.
#[derive(Component)]
pub(super) struct Foe(usize);

/// Marks the name in the strip of slot `0`.
#[derive(Component)]
pub(super) struct FoeName(usize);

/// Marks the track of the health bar in the strip of slot `0`.
#[derive(Component)]
pub(super) struct FoeTrack(usize);

/// Marks the fill of the health bar in the strip of slot `0`.
#[derive(Component)]
pub(super) struct FoeBar(usize);

/// Returns the layer of the strips: its own root over the whole screen, gated by
/// [`Element::Labels`] and not by the reticle, so it shows on foot with or without a tool, and a
/// point at the centre of the screen the strips hang from.
pub(super) fn layer() -> impl Bundle {
    let whole = Node {
        position_type: PositionType::Absolute,
        width: percent(100),
        height: percent(100),
        ..default()
    };
    let centre = Node {
        position_type: PositionType::Absolute,
        left: percent(50),
        top: percent(50),
        width: px(0),
        height: px(0),
        ..default()
    };
    let strips = (0..SLOTS).map(strip);
    (
        (gate(Element::Labels), OverWorld, whole, Pickable::IGNORE),
        children![(centre, Pickable::IGNORE, Children::spawn(SpawnIter(strips)))],
    )
}

/// Returns the strip of slot `slot`, to hang from the centre of the screen: a name over a health
/// bar, with no panel, hidden.
fn strip(slot: usize) -> impl Bundle {
    let node = Node {
        display: Display::None,
        position_type: PositionType::Absolute,
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(theme::S1),
        width: px(2.0 * BAR.0),
        height: px(STRIP),
        ..default()
    };
    let track = Node {
        width: px(BAR.0),
        height: px(BAR.1),
        ..default()
    };
    let fill = (FoeBar(slot), Node::default(), BackgroundColor(theme::TEXT));
    let name = text("", theme::FONT_SMALL, theme::TEXT);
    let track = (FoeTrack(slot), track, BackgroundColor(theme::HAIRLINE));
    (
        (Foe(slot), node),
        children![
            (FoeName(slot), name, TextLayout::no_wrap()),
            (track, children![fill])
        ],
    )
}

/// How far one strip has faded in, and the animal it shows.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Fade {
    /// The animal it shows, kept while it fades out.
    pub animal: Option<Entity>,
    /// How far it is in, 0 to 1.
    pub alpha: f32,
}

/// Returns `fade` after `dt` seconds of being asked to show `wanted`: in toward it, or out of the
/// animal it showed when it is asked for none.
pub fn faded(fade: Fade, wanted: Option<Entity>, dt: f32) -> Fade {
    let step = dt / FADE;
    match wanted {
        Some(animal) => Fade {
            animal: Some(animal),
            alpha: (fade.alpha + step).min(1.0),
        },
        None => {
            let alpha = (fade.alpha - step).max(0.0);
            Fade {
                animal: fade.animal.filter(|_| alpha > 0.0),
                alpha,
            }
        }
    }
}

/// The strips, the fills of their bars, their names and the tracks of their bars.
type Strips<'w, 's> = (
    Query<'w, 's, (&'static Foe, &'static mut Node), Without<FoeBar>>,
    Query<
        'w,
        's,
        (
            &'static FoeBar,
            &'static mut Node,
            &'static mut BackgroundColor,
        ),
        Without<Foe>,
    >,
    Query<'w, 's, (&'static FoeName, &'static mut Text, &'static mut TextColor)>,
    Query<'w, 's, (&'static FoeTrack, &'static mut BackgroundColor), Without<FoeBar>>,
);

/// What a strip shows: the share of health left, the name and where it hangs from the centre.
type Shown<'a> = (f32, &'a str, (f32, f32));

/// Draws the strip of `slot` among `strips` showing `shown`, faded to `alpha`, or hides it.
fn draw((nodes, bars, names, tracks): &mut Strips, slot: usize, shown: Option<Shown>, alpha: f32) {
    let display = [Display::None, Display::Flex][usize::from(shown.is_some() && alpha > 0.0)];
    let (left, bottom) = shown.map_or((0.0, 0.0), |(_, _, at)| at);
    for (_, mut node) in nodes.iter_mut().filter(|(foe, _)| foe.0 == slot) {
        (node.display, node.left, node.bottom) = (display, px(left), px(bottom));
    }
    let Some((health, name, _)) = shown else {
        return;
    };
    for (_, mut node, mut back) in bars.iter_mut().filter(|(bar, ..)| bar.0 == slot) {
        node.width = percent(100.0 * health);
        back.set_if_neq(BackgroundColor(theme::TEXT.with_alpha(alpha)));
    }
    let hairline = theme::HAIRLINE.with_alpha(theme::HAIRLINE.alpha() * alpha);
    for (_, mut back) in tracks.iter_mut().filter(|(track, _)| track.0 == slot) {
        back.set_if_neq(BackgroundColor(hairline));
    }
    for (_, mut words, mut colour) in names.iter_mut().filter(|(named, ..)| named.0 == slot) {
        if words.0 != name {
            words.0 = name.to_string();
        }
        colour.set_if_neq(TextColor(theme::TEXT.with_alpha(alpha)));
    }
}

/// The last blow, the living animals, the terrain and the clock.
type Field<'w, 's> = (
    Res<'w, Mark>,
    Query<'w, 's, (Entity, &'static Beast, &'static GlobalTransform), Live>,
    Option<Res<'w, Terrain>>,
    Res<'w, Time>,
);

/// Shows over their heads the animal fought and the animal looked at, each faded in and out.
pub(super) fn show(
    (mark, beasts, terrain, time): Field,
    (parts, lens): (Bounds, Lens),
    mut strips: Strips,
    mut fades: Local<[Fade; SLOTS]>,
) {
    let Some((camera, screen, frustum)) = sight(&lens) else {
        return;
    };
    let (eye, dir) = (camera.translation(), camera.forward().as_vec3());
    let near = beasts
        .iter()
        .filter(|(_, _, placed)| placed.translation().distance(eye) <= RANGE + 3.0)
        .map(|(animal, _, placed)| {
            (
                animal,
                (placed.translation() + top((animal, placed), &parts)) / 2.0,
            )
        });
    let fought = mark.victim.filter(|victim| beasts.contains(*victim));
    let wanted = labels(fought, looked(near, (eye, dir)));
    let life = terrain
        .as_deref()
        .and_then(|terrain| terrain.globe().life());
    for slot in 0..SLOTS {
        fades[slot] = faded(fades[slot], wanted[slot], time.delta_secs());
        let shown = fades[slot].animal.and_then(|animal| {
            let (_, beast, placed) = beasts.get(animal).ok()?;
            let name = &life?.species.get(usize::from(beast.index))?.name;
            let pos = over(top((animal, placed), &parts), &camera, (frustum, screen))?;
            Some((beast.wits.health.share(), name.as_str(), hung(pos, screen)))
        });
        draw(&mut strips, slot, shown, fades[slot].alpha);
    }
}

/// Registers the strips: they follow the animals every frame while the player controls the game,
/// after the eye of the HUD is captured.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        show.after(super::eye::capture)
            .run_if(any_with_component::<Beast>)
            .in_set(super::Step::Playing),
    );
}
