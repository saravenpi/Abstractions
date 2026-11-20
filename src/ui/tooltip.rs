//! The tooltip: the name of an icon control, shown on a panel under it after the pointer rested
//! on it for [`motion::TOOLTIP_DELAY`] seconds, faded in over [`motion::FADE`]. Put a [`Tip`]
//! next to the `Hovered` of a button to give it one.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use super::motion;
use super::theme;

/// The text of the tooltip of a control.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tip(pub &'static str);

/// Marks the one tooltip panel.
#[derive(Component)]
pub struct Bubble;

/// Marks the text of the tooltip panel.
#[derive(Component)]
struct Words;

/// What the tooltip remembers between frames: the control the pointer rests on, how long it
/// has rested there, and how visible the panel is.
#[derive(Default)]
struct Rest {
    /// The control under the pointer that has a tip.
    over: Option<Entity>,
    /// Seconds the pointer has rested on it.
    seconds: f32,
    /// Opacity of the panel, from 0 to 1.
    alpha: f32,
}

/// Spawns the hidden tooltip panel above everything else.
pub(super) fn spawn(mut commands: Commands) {
    commands.spawn((
        Bubble,
        Node {
            position_type: PositionType::Absolute,
            padding: UiRect::axes(px(theme::S3), px(theme::S2)),
            ..default()
        },
        UiTransform::from_translation(Val2::percent(-50.0, 0.0)),
        BackgroundColor(Color::NONE),
        GlobalZIndex(i32::MAX - 1),
        Pickable::IGNORE,
        Visibility::Hidden,
        children![(
            Words,
            Text::default(),
            TextFont::from_font_size(theme::FONT_SMALL),
            TextLayout::no_wrap(),
            TextColor(Color::NONE),
        )],
    ));
}

/// The panel and its text.
type Parts<'w, 's> = (
    Query<
        'w,
        's,
        (
            &'static mut Node,
            &'static mut BackgroundColor,
            &'static mut Visibility,
        ),
        With<Bubble>,
    >,
    Query<'w, 's, (&'static mut Text, &'static mut TextColor), (With<Words>, Without<Bubble>)>,
);

/// Puts the panel under the control `tip` belongs to and writes its text.
fn move_under(
    (tip, size, at): (&Tip, &ComputedNode, &UiGlobalTransform),
    (panel, words): &mut Parts,
) {
    let scale = size.inverse_scale_factor;
    let below = (at.translation.y + size.size.y * 0.5) * scale + theme::S1;
    if let Ok((mut node, ..)) = panel.single_mut() {
        node.left = px(at.translation.x * scale);
        node.top = px(below);
    }
    if let Ok((mut text, _)) = words.single_mut() {
        text.set_if_neq(Text(tip.0.to_string()));
    }
}

/// Fades the panel to `alpha`, eased.
fn tint(alpha: f32, (panel, words): &mut Parts) {
    let eased = motion::ease_out(alpha);
    if let Ok((_, mut fill, mut visibility)) = panel.single_mut() {
        visibility
            .set_if_neq([Visibility::Hidden, Visibility::Inherited][usize::from(alpha > 0.0)]);
        fill.set_if_neq(BackgroundColor(
            theme::PANEL.with_alpha(theme::PANEL.alpha() * eased),
        ));
    }
    if let Ok((_, mut color)) = words.single_mut() {
        color.set_if_neq(TextColor(theme::TEXT.with_alpha(eased)));
    }
}

/// Moves the tooltip under the control the pointer rests on, once it rested long enough, and
/// fades it away when the pointer leaves.
pub(super) fn update(
    time: Res<Time>,
    tips: Query<(Entity, &Hovered, &Tip, &ComputedNode, &UiGlobalTransform)>,
    mut parts: Parts,
    mut rest: Local<Rest>,
) {
    let over = tips.iter().find(|(_, hovered, ..)| hovered.get());
    let entity = over.map(|(entity, ..)| entity);
    if entity != rest.over {
        (rest.over, rest.seconds) = (entity, 0.0);
    }
    rest.seconds += time.delta_secs();
    let shown = entity.is_some() && rest.seconds >= motion::TOOLTIP_DELAY;
    let alpha = motion::approach(rest.alpha, shown, time.delta_secs());
    if alpha == 0.0 && rest.alpha == 0.0 {
        return;
    }
    rest.alpha = alpha;
    if let Some((_, _, tip, size, at)) = over.filter(|_| shown) {
        move_under((tip, size, at), &mut parts);
    }
    tint(alpha, &mut parts);
}
