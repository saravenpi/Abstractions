//! The waves a source leaves in the sea itself: one packet in the ring buffer of the wakes for
//! every [`PERIOD`] a moving source sprays, and one for every ripple ring of a hovering one.

use super::throw::Spot;
use super::tune::RIPPLE_PERIOD;
use super::wash::Flow;
use crate::space::Anchor;
use crate::world::wake::{MIN_AMPLITUDE, MOVING, PERIOD, Packet, Wake, Wakes, amplitude};

/// Returns the height in metres of the waves `spot` raises, from the dynamic pressure of its wash
/// on the sea and, when it touches the surface, from its speed. `air` is the density of the air
/// around it and `liquid` the density of the sea, both in kilograms per cubic metre.
fn raised(spot: &Spot, flow: &Flow, (air, liquid): (f32, f32)) -> f32 {
    let pressure = 0.5 * air * flow.surface * flow.surface;
    let touches = spot.height <= 0.5 * spot.size;
    let speed = spot.velocity.length();
    amplitude(pressure, liquid, spot.gravity, (speed, touches))
}

/// Writes the packet of `spot` into the buffer once `due` seconds, counted here, reach the period
/// of its kind: [`PERIOD`] while it moves and sprays, the ripple period while it hovers with a
/// wash that rings the sea. Anything else restarts the count.
///
/// `cause` is the spot, its flow and the strength of its spray, `density` the air and the liquid
/// and `dt` the seconds of the frame.
pub fn lay(
    wakes: &mut Wakes,
    anchor: &Anchor,
    due: &mut f32,
    cause: (&Spot, &Flow, f32),
    (density, dt): ((f32, f32), f32),
) {
    let (spot, flow, strength) = cause;
    let speed = spot.velocity.length();
    let moving = speed > MOVING;
    let (live, period) = match moving {
        true => (strength > 0.0, PERIOD),
        false => (flow.ripple > 0.0, RIPPLE_PERIOD),
    };
    *due = [0.0, *due + dt][usize::from(live)];
    if *due < period {
        return;
    }
    *due %= period;
    let amp = raised(spot, flow, density);
    if amp < MIN_AMPLITUDE {
        return;
    }
    let packet = match moving {
        true => Packet::moving(speed, spot.gravity, spot.size, (amp, spot.now)),
        false => Packet::hover(spot.gravity, spot.size, (amp, spot.now)),
    };
    let on = spot.at - spot.up * spot.height;
    let ahead = spot.velocity.normalize_or(spot.up.any_orthonormal_vector());
    wakes.push(Wake {
        at: anchor.to_frame(on),
        ahead: anchor.to_frame(on + ahead),
        over: anchor.to_frame(on + spot.up),
        packet,
    });
}
