//! Laying out the labels: which animal in view takes which strip of the pool, how bright it is with
//! its distance, and which names step aside when a herd stands bunched on screen. Nothing is
//! allocated: the lists are kept from frame to frame.

use bevy::prelude::*;

use super::place::hung;
use super::{SLOTS, STRIP};

/// Metres within which an animal in view shows its label at its full brightness.
pub const FULL: f32 = 25.0;

/// Metres past which an animal shows no label: its label fades out from [`FULL`] to here.
pub const FAR: f32 = 60.0;

/// Brightness of the label of an animal neither fought nor looked at, at its fullest.
pub const DIM: f32 = 0.7;

/// Pixels of the name at the top of a strip, the part that steps aside for a nearer one.
const NAME: f32 = 16.0;

/// An animal in view this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Found {
    /// Index of its species in the life of the planet.
    pub species: u16,
    /// Share of its health left, 0 to 1.
    pub health: f32,
    /// Where its head shows, in interface pixels.
    pub pos: Vec2,
    /// Metres from the eyes.
    pub distance: f32,
    /// True for the animal fought or the one looked at, which show brightest and first.
    pub keen: bool,
}

/// What one strip of the pool shows.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Label {
    /// Index of the species named.
    pub species: u16,
    /// Share of its health left, 0 to 1.
    pub health: f32,
    /// Its left and its bottom from the centre of the screen, in pixels.
    pub at: (f32, f32),
    /// How bright it is, 0 to 1.
    pub alpha: f32,
    /// False when a nearer label covers its name: only its bar shows.
    pub named: bool,
}

/// Returns how bright the label of an animal `distance` metres away is: full under [`FULL`],
/// nothing past [`FAR`], and [`DIM`] of that unless it is `keen`.
pub fn faint(distance: f32, keen: bool) -> f32 {
    let fade = ((FAR - distance) / (FAR - FULL)).clamp(0.0, 1.0);
    fade * if keen { 1.0 } else { DIM }
}

/// Returns the rectangle the name of a strip hung at `at` takes on a `screen`.
fn name_box((left, bottom): (f32, f32), screen: Vec2) -> Rect {
    let (x, base) = (screen.x / 2.0 + left, screen.y / 2.0 - bottom);
    Rect::new(x, base - STRIP, x + 2.0 * super::BAR.0, base - STRIP + NAME)
}

/// Lays the animals of `found` into `labels`, one strip each in the pool of [`SLOTS`]: the keen
/// ones first, then the nearest, those past [`FAR`] left out, a name that a label laid before
/// covers left out while its bar shows. Unused strips are emptied. `taken` keeps the names laid.
pub fn lay(found: &mut [Found], screen: Vec2, labels: &mut [Option<Label>], taken: &mut Vec<Rect>) {
    found.sort_unstable_by(|a, b| b.keen.cmp(&a.keen).then(a.distance.total_cmp(&b.distance)));
    taken.clear();
    labels.iter_mut().for_each(|label| *label = None);
    let shown = found.iter().filter(|f| faint(f.distance, f.keen) > 0.0);
    for (slot, found) in shown.take(SLOTS.min(labels.len())).enumerate() {
        let at = hung(found.pos, screen);
        let name = name_box(at, screen);
        let named = !taken.iter().any(|other| !other.intersect(name).is_empty());
        if named {
            taken.push(name);
        }
        labels[slot] = Some(Label {
            species: found.species,
            health: found.health,
            at,
            alpha: faint(found.distance, found.keen),
            named,
        });
    }
}
