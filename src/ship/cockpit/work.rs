//! Moving the parts of every cockpit from its own state.

use bevy::prelude::*;

use super::part::Part;
use super::state::Cockpit;

/// Moves the parts of every cockpit from the state of its own ship: what is shown follows what
/// is asked and read, and the levers, the pedals, the strap, the bars and the lamps follow what
/// is shown. A part is written only when it moved.
pub fn work(
    time: Res<Time>,
    mut cockpits: Query<&mut Cockpit>,
    mut parts: Query<(&Part, &ChildOf, &mut Transform)>,
) {
    let dt = time.delta_secs();
    for mut cockpit in &mut cockpits {
        let mut next = *cockpit;
        next.step(dt);
        cockpit.set_if_neq(next);
    }
    for (part, parent, mut transform) in &mut parts {
        let Ok(cockpit) = cockpits.get(parent.parent()) else {
            continue;
        };
        let placed = part.placed(cockpit, *transform);
        transform.set_if_neq(placed);
    }
}
