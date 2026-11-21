//! Drawing the pool of strips: each node of each kind written once a frame from the label of its
//! strip, nothing written that did not change.

use bevy::prelude::*;

use super::{Foe, FoeBar, FoeName, FoeTrack, Label};
use crate::life::Life;
use crate::ui::theme;

/// The strips, the fills of their bars, their names and the tracks of their bars.
pub(super) type Strips<'w, 's> = (
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

/// Returns the label of strip `slot` among `labels`, when it shows one.
fn of(labels: &[Option<Label>], slot: usize) -> Option<Label> {
    labels
        .get(slot)
        .copied()
        .flatten()
        .filter(|label| label.alpha > 0.0)
}

/// Writes every strip of `strips` from its label among `labels`, named from `life`: placed and
/// shown, or hidden when it has none.
pub(super) fn write(
    (nodes, bars, names, tracks): &mut Strips,
    labels: &[Option<Label>],
    life: Option<&Life>,
) {
    hang(nodes, labels);
    let shown = |slot: usize| of(labels, slot);
    for (bar, node, back) in bars.iter_mut() {
        if let Some(label) = shown(bar.0) {
            fill((node, back), &label);
        }
    }
    for (track, mut back) in tracks.iter_mut() {
        let alpha = shown(track.0).map_or(0.0, |label| label.alpha);
        let hairline = theme::HAIRLINE.with_alpha(theme::HAIRLINE.alpha() * alpha);
        back.set_if_neq(BackgroundColor(hairline));
    }
    for (named, words, colour) in names.iter_mut() {
        if let Some(label) = shown(named.0) {
            name((words, colour), &label, life);
        }
    }
}

/// Places every strip of `nodes` from its label among `labels`, or hides it.
fn hang(
    nodes: &mut Query<(&'static Foe, &'static mut Node), Without<FoeBar>>,
    labels: &[Option<Label>],
) {
    for (foe, mut node) in nodes.iter_mut() {
        let label = of(labels, foe.0);
        let display = [Display::None, Display::Flex][usize::from(label.is_some())];
        let (left, bottom) = label.map_or((0.0, 0.0), |label| label.at);
        if node.display != display || node.left != px(left) || node.bottom != px(bottom) {
            (node.display, node.left, node.bottom) = (display, px(left), px(bottom));
        }
    }
}

/// Writes the fill of a bar, its node and its colour, from `label`.
fn fill((mut node, mut back): (Mut<Node>, Mut<BackgroundColor>), label: &Label) {
    let width = percent(100.0 * label.health);
    if node.width != width {
        node.width = width;
    }
    back.set_if_neq(BackgroundColor(theme::TEXT.with_alpha(label.alpha)));
}

/// Writes a name, its words and its colour, from `label` and the species of `life`: clear when a
/// nearer name covers it.
fn name((mut words, mut colour): (Mut<Text>, Mut<TextColor>), label: &Label, life: Option<&Life>) {
    let species = life.and_then(|life| life.species.get(usize::from(label.species)));
    let said = species.map_or("", |species| species.name.as_str());
    if words.0 != said {
        words.0.clear();
        words.0.push_str(said);
    }
    let alpha = if label.named { label.alpha } else { 0.0 };
    colour.set_if_neq(TextColor(theme::TEXT.with_alpha(alpha)));
}
