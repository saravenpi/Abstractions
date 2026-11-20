//! Fading of whole HUD elements when the context changes: a [`Gate`] on the root of an element
//! lets it fade in or out over [`motion::FADE`] seconds when its element enters or leaves the
//! set of `context::shown`, and hides it once it is gone, so nothing pops.
//!
//! A fade multiplies the opacity of every text and background under the root. The colours the
//! writers of the HUD wrote are remembered while it lasts: when a writer writes a new colour
//! mid-fade, that colour becomes the one faded.

use std::collections::HashMap;

use bevy::prelude::*;

use super::context::{Element, Where};
use crate::ui::motion;

/// Makes the root of an element fade with its context.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(super) struct Gate {
    /// The element it belongs to.
    pub element: Element,
    /// How visible it is, from 0 (gone) to 1 (whole).
    pub alpha: f32,
}

/// Returns a gate for `element`, closed until its context shows it.
pub(super) const fn gate(element: Element) -> Gate {
    Gate {
        element,
        alpha: 0.0,
    }
}

/// The colours remembered while a fade lasts: the whole colour and the one written last.
type Memory = HashMap<Entity, (Color, Color)>;

/// Returns `now` at `level` of its opacity, remembering the whole colour in `memory`, and
/// forgetting it once the level is whole.
fn dim(memory: &mut Memory, entity: Entity, now: Color, level: f32) -> Color {
    let entry = memory.entry(entity).or_insert((now, now));
    if now != entry.1 {
        entry.0 = now;
    }
    let whole = entry.0;
    if level >= 1.0 {
        memory.remove(&entity);
        return whole;
    }
    let shown = whole.with_alpha(whole.alpha() * level);
    entry.1 = shown;
    shown
}

/// What a fade reads and writes under a root: the nodes below it and their colours.
type Under<'w, 's> = (
    Query<'w, 's, &'static Children>,
    Query<'w, 's, &'static mut TextColor>,
    Query<'w, 's, &'static mut BackgroundColor>,
);

/// Sets the opacity of the root and everything under it to `level`.
fn dim_under(root: Entity, level: f32, under: &mut Under, memory: &mut (Memory, Memory)) {
    let (kids, texts, fills) = under;
    for below in std::iter::once(root).chain(kids.iter_descendants(root)) {
        if let Ok(mut color) = texts.get_mut(below) {
            let out = dim(&mut memory.0, below, color.0, level);
            color.set_if_neq(TextColor(out));
        }
        if let Ok(mut color) = fills.get_mut(below) {
            let out = dim(&mut memory.1, below, color.0, level);
            color.set_if_neq(BackgroundColor(out));
        }
    }
}

/// Moves every gate toward what its element's context wants, and fades what is under it.
pub(super) fn step(
    (time, now): (Res<Time>, Res<Where>),
    mut gates: Query<(Entity, &mut Gate, &mut Visibility)>,
    mut under: Under,
    mut memory: Local<(Memory, Memory)>,
) {
    for (root, mut gate, mut visibility) in &mut gates {
        let before = gate.alpha;
        let after = motion::approach(before, now.has(gate.element), time.delta_secs());
        let shown = [Visibility::Hidden, Visibility::Inherited][usize::from(after > 0.0)];
        visibility.set_if_neq(shown);
        if after != before {
            gate.alpha = after;
            dim_under(root, motion::ease_out(after), &mut under, &mut memory);
        }
    }
}
