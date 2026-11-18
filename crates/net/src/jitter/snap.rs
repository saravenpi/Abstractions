//! One snapshot of a remote body, the state drawn from the buffer, and how one becomes the other.

use glam::{DVec3, Quat, Vec3};

/// Longest time in seconds a body is carried forward on its velocity past its newest snapshot.
/// After that it holds still.
pub const EXTRAPOLATE_MAX: f64 = 0.1;

/// One sample of a remote body as its record carried it.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Snap<E> {
    /// Universe time of the sample in seconds: the time of the record's head.
    pub time: f64,
    /// Position in the frame of the record, in metres.
    pub position: DVec3,
    /// Velocity in the frame of the record, or nothing for a body that sends none.
    pub velocity: Option<Vec3>,
    /// Orientation in the frame of the record.
    pub rotation: Quat,
    /// The rest of the body, which the caller blends.
    pub extra: E,
}

/// The state of a remote body at one time, ready to draw.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View<E> {
    /// Universe time the state is for, in seconds: `now` minus the delay.
    pub time: f64,
    /// Position in the frame of the snapshots, in metres.
    pub position: DVec3,
    /// Orientation in the frame of the snapshots.
    pub rotation: Quat,
    /// Extra of the snapshot before `time`.
    pub older: E,
    /// Extra of the snapshot after `time`. The same as `older` when the body holds or is
    /// carried forward.
    pub newer: E,
    /// Share of the way from `older` to `newer`, from 0 to 1.
    pub blend: f32,
}

impl<E: Copy> Snap<E> {
    /// Returns the snapshot as it stands, drawn at `time`.
    pub(super) fn hold(&self, time: f64) -> View<E> {
        View {
            time,
            position: self.position,
            rotation: self.rotation,
            older: self.extra,
            newer: self.extra,
            blend: 0.0,
        }
    }

    /// Returns the snapshot carried on its velocity to `time`, for at most [`EXTRAPOLATE_MAX`].
    pub(super) fn ahead(&self, time: f64) -> View<E> {
        let lead = (time - self.time).clamp(0.0, EXTRAPOLATE_MAX);
        let drift = self
            .velocity
            .map_or(DVec3::ZERO, |velocity| velocity.as_dvec3() * lead);
        View {
            position: self.position + drift,
            ..self.hold(time)
        }
    }

    /// Returns the state at `time` between this snapshot and the later `next`.
    pub(super) fn toward(&self, next: &Self, time: f64) -> View<E> {
        let span = next.time - self.time;
        let share = ((time - self.time) / span).clamp(0.0, 1.0);
        let position = match (self.velocity, next.velocity) {
            (Some(start), Some(end)) => hermite(
                [self.position, next.position],
                [start.as_dvec3() * span, end.as_dvec3() * span],
                share,
            ),
            _ => self.position.lerp(next.position, share),
        };
        View {
            time,
            position,
            rotation: self.rotation.slerp(next.rotation, share as f32),
            older: self.extra,
            newer: next.extra,
            blend: share as f32,
        }
    }
}

/// Returns the point at `share` of the cubic Hermite curve from `ends[0]` to `ends[1]` with the
/// tangents `tangents`, each a velocity times the span between the two ends.
fn hermite(ends: [DVec3; 2], tangents: [DVec3; 2], share: f64) -> DVec3 {
    let square = share * share;
    let cube = square * share;
    ends[0] * (2.0 * cube - 3.0 * square + 1.0)
        + tangents[0] * (cube - 2.0 * square + share)
        + ends[1] * (3.0 * square - 2.0 * cube)
        + tangents[1] * (cube - square)
}
