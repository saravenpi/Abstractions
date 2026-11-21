//! The HUD shadow. The in-game HUD has no panel: every text and icon of an interface over the
//! live world carries [`theme::HUD_SHADOW`], one physical pixel down and to the right of its
//! glyph, so it reads over a bright ground or sky without a box that takes the player out of the
//! game. Warning text carries the deeper [`theme::HUD_SHADOW_WARN`]. Nothing drawn on a
//! picture, such as the minimap, carries one, and menus never do.
//!
//! Bevy 0.20 draws one `TextShadow` per text, an offset copy of its glyphs with no blur. A second
//! shadow toward the top left would need a second text node behind every label, and a light edge
//! on both sides reads as an outline, not as a shadow: the HUD keeps one.

use bevy::prelude::*;
use bevy::ui::UiSystems;

use super::theme;

/// Marks the root of an interface over the live world with no panel, the HUD and the hotbar:
/// every text and icon under it carries the HUD shadow.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct OverWorld;

/// Marks a node under an [`OverWorld`] root whose texts and icons carry no shadow: a picture,
/// such as the minimap, and everything drawn on it.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Unshaded;

/// Returns the physical pixels the HUD shadow lies from its glyph at `scale` physical pixels
/// per logical one: [`theme::HUD_SHADOW_REACH`] per logical pixel rounded down, one at least and
/// [`theme::HUD_SHADOW_MOST`] at most.
pub fn reach(scale: f32) -> f32 {
    (theme::HUD_SHADOW_REACH * scale)
        .floor()
        .clamp(1.0, theme::HUD_SHADOW_MOST)
}

/// Returns whether `color` is the warning or the danger colour, at any opacity.
fn warning(color: Color) -> bool {
    let rgb = |color: Color| {
        let color = color.to_srgba();
        [color.red, color.green, color.blue]
    };
    [theme::WARN, theme::DANGER]
        .into_iter()
        .any(|token| rgb(token) == rgb(color))
}

/// Returns the shadow of a HUD text drawn in `text` at `scale` physical pixels per logical one:
/// [`theme::HUD_SHADOW_WARN`] for a warning colour and [`theme::HUD_SHADOW`] for the rest, a
/// whole number of physical pixels down and to the right. A text fainter than `TEXT_2` gets a
/// shadow as much fainter, so a text that fades takes its shadow with it and a hidden one
/// casts none.
pub fn hud_shadow(text: Color, scale: f32) -> TextShadow {
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let base = [theme::HUD_SHADOW, theme::HUD_SHADOW_WARN][usize::from(warning(text))];
    let level = (text.alpha() / theme::TEXT_2.alpha()).clamp(0.0, 1.0);
    TextShadow {
        offset: Vec2::splat(reach(scale) / scale),
        color: base.with_alpha(base.alpha() * level),
    }
}

/// Returns the physical pixels per logical one of the interface `node` is laid out at.
fn scale_of(node: &ComputedNode) -> f32 {
    1.0 / node.inverse_scale_factor()
}

/// Returns whether `entity` carries the HUD shadow: the nearest of it and its ancestors that is
/// marked is an [`OverWorld`] root, not an [`Unshaded`] node.
fn shaded(
    entity: Entity,
    parents: &Query<&ChildOf>,
    marks: &Query<(Has<OverWorld>, Has<Unshaded>)>,
) -> bool {
    std::iter::once(entity)
        .chain(parents.iter_ancestors(entity))
        .find_map(|at| match marks.get(at) {
            Ok((_, true)) => Some(false),
            Ok((true, false)) => Some(true),
            _ => None,
        })
        .unwrap_or(false)
}

/// Gives the HUD shadow to every text added under an [`OverWorld`] root and outside an
/// [`Unshaded`] node. Each text is looked at once, on the frame after it is added.
fn adopt(
    added: Query<(Entity, &TextColor, &ComputedNode), (Added<Text>, Without<TextShadow>)>,
    parents: Query<&ChildOf>,
    marks: Query<(Has<OverWorld>, Has<Unshaded>)>,
    mut commands: Commands,
) {
    for (entity, color, node) in &added {
        if shaded(entity, &parents, &marks) {
            let shadow = hud_shadow(color.0, scale_of(node));
            commands.entity(entity).try_insert(shadow);
        }
    }
}

/// The shadows whose text changed its colour or its scale.
type Moved = Or<(Changed<ComputedNode>, Changed<TextColor>)>;

/// Keeps every HUD shadow at its reach and opacity when the scale of the interface or the
/// colour of its text changes. Every `TextShadow` of the game is a HUD shadow.
fn fit(mut shadows: Query<(&ComputedNode, &TextColor, &mut TextShadow), Moved>) {
    for (node, color, mut shadow) in &mut shadows {
        shadow.set_if_neq(hud_shadow(color.0, scale_of(node)));
    }
}

/// Registers the shadow of the HUD, after the layout so it reads the scale of this frame.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        PostUpdate,
        (adopt, fit)
            .chain()
            .after(UiSystems::Layout)
            .run_if(any_with_component::<OverWorld>),
    );
}
