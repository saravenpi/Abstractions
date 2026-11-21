//! The camera rig: one orbit around a pivot, described by distance, yaw and pitch relative to
//! its target and turned into a pose at the very end.

use bevy::prelude::*;

use super::aim::Aim;
use super::basis::{self, Stance};
use super::blend::Blend;
use super::curve::{self, Bend};
use super::io::{BodyFade, CameraPose, Heading, Kind, Moment, Occluder, Shown, Target};
use super::occlude::{self, Probe};
use super::sane;
use super::shot::Shot;
use super::sway::Sway;
use super::tune::{
    CEILING_BAND, CEILING_REACH, DT_RANGE, HEAD_CENTRE, HEAD_RADIUS, HEAD_REACH, INDOOR_RATE,
    RADIUS, SKIN,
};
use crate::render::smoothstep;

#[cfg(test)]
#[path = "rig_tests.rs"]
mod tests;

/// The state of the camera between frames.
///
/// Every part of it is an angle, a length or a vector in the axes of the target. Nothing is a
/// position in render space, so moving and turning the target and the world together, as a
/// rebase does, changes nothing here.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct Rig {
    /// The kind of target of the last step.
    seen: Option<Kind>,
    /// Whether a cut was asked for since the last step.
    cut: bool,
    /// Blends between the views and between the rigs.
    blend: Blend,
    /// The look angles.
    aim: Aim,
    /// Lag, steps, roll, zoom and widening.
    sway: Sway,
    /// Turn of the camera away from the axes of the target: the lag behind the rotation of the ship.
    rel: Quat,
    /// How much shorter than asked the boom is, because something blocks the view.
    short: f32,
    /// How far indoors the rig is, from 0 to 1.
    indoor: f32,
}

impl Rig {
    /// Makes the next step a cut: the camera goes straight to where it belongs, with no smoothing.
    pub fn cut(&mut self) {
        self.cut = true;
    }

    /// Sets the look angles, as after a spawn or when the player leaves the ship.
    /// Returns the roll of the camera around where it looks, as last shown, positive to the
    /// right.
    pub fn roll(&self) -> f32 {
        self.sway.roll.angle
    }

    pub fn face(&mut self, heading: Heading) {
        self.aim = Aim::settled(heading);
    }

    /// Returns true when this step is a cut, and puts the state at rest if so. A step is a cut
    /// when one was asked for, on the first step, when the target changes to or from the ship,
    /// and when a value of the state is not a finite number: the rig then heals in one frame,
    /// on the look it had when that look is still a number.
    fn begin(&mut self, moment: &Moment) -> bool {
        let ship = moment.target.kind == Kind::Ship;
        let crossed = self.seen.is_some_and(|was| (was == Kind::Ship) != ship);
        let mine = self.rel.is_finite() && Vec2::new(self.short, self.indoor).is_finite();
        let sound = mine && self.aim.sound() && self.sway.sound() && self.blend.sound();
        let fresh = self.cut || crossed || !sound || self.seen.is_none();
        if fresh {
            let held = Vec2::new(self.aim.heading.yaw, self.aim.heading.pitch).is_finite();
            let heading = match (crossed && ship) || !held {
                true => Heading::default(),
                false => self.aim.heading,
            };
            self.aim = Aim::settled(heading);
            self.blend = Blend::settled(moment.target.kind, &moment.prefs);
            self.sway = Sway::settled(moment.prefs.zoom);
            (self.rel, self.short, self.indoor) = (Quat::IDENTITY, 0.0, 0.0);
        }
        (self.seen, self.cut) = (Some(moment.target.kind), false);
        fresh
    }

    /// Returns the orientation of the orbit in the axes of the target, and lets the camera
    /// catch up with the rotation of the target.
    fn look(&mut self, moment: &Moment, stance: &Stance, shot: &Shot) -> Quat {
        let follow = moment.prefs.auto_follow;
        let goal = stance.frame.inverse() * basis::chase(&moment.target, shot.third, follow);
        let swept = stance.spin * stance.dt;
        self.rel = match stance.fresh {
            true => goal,
            false => basis::carried(self.rel, swept, goal, stance.gain(shot.turn)),
        };
        let (yaw, pitch) = self.aim.shown(shot.third);
        self.rel * Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch)
    }

    /// Returns what bends the shot this step, and looks for a ceiling above a body.
    fn bend(
        &mut self,
        target: &Target,
        stance: &Stance,
        third: f32,
        world: &impl Occluder,
    ) -> Bend {
        let roof = match target.kind {
            Kind::Ship => None,
            Kind::OnFoot | Kind::Flying => world.cast(
                target.position,
                stance.frame * Vec3::Y,
                CEILING_REACH,
                RADIUS,
            ),
        };
        let goal = roof.map_or(0.0, |gap| {
            1.0 - smoothstep(CEILING_BAND.0, CEILING_BAND.1, gap)
        });
        self.indoor += (goal - self.indoor) * stance.gain(INDOOR_RATE);
        Bend {
            tilt: curve::tilt(self.aim.shown(third).1),
            indoor: self.indoor,
            zoom: self.sway.zoom,
            rush: self.sway.rush,
        }
    }

    /// Returns where the camera goes on its way from `pivot` along `swing`, and how far from
    /// the pivot that is. The boom pulls in fast and eases out slowly. A boom that stays inside
    /// the body, as the one from the neck to the eyes does, is cast with the small sphere of
    /// the head and never eased: the body already keeps it off the walls.
    fn place(&mut self, probe: Option<Probe>, stance: &Stance, world: &impl Occluder) -> f32 {
        let Some(probe) = probe else {
            self.short = 0.0;
            return 0.0;
        };
        if probe.max <= HEAD_REACH {
            self.short = 0.0;
            let free = world.cast(probe.origin, probe.dir, probe.max, HEAD_RADIUS);
            return free.map_or(probe.max, |hit| (hit - SKIN).max(0.0));
        }
        let hard = occlude::reach(world, &probe);
        let goal = probe.max - hard.min(occlude::feel(world, &probe));
        let short = match stance.fresh {
            true => goal,
            false => occlude::ease(self.short.min(probe.max), goal, stance.dt),
        };
        let gap = (probe.max - short).min(hard);
        self.short = probe.max - gap;
        gap
    }

    /// Runs the rig for one frame and returns the camera. What it reads is first brought within
    /// bounds, see [`sane::moment`].
    pub fn step(&mut self, moment: &Moment, world: &impl Occluder) -> Shown {
        let moment = &sane::moment(moment);
        let target = &moment.target;
        let dt = moment.dt.clamp(DT_RANGE.0, DT_RANGE.1);
        let stance = Stance::of(target, dt, self.begin(moment));
        self.blend.step(target.kind, &moment.prefs, dt);
        let shot = Shot::of(target, self.blend.person(), self.blend.flight());
        self.aim.flight = self.blend.flight();
        self.aim.step(moment, &stance, shot.third);
        let look = self.look(moment, &stance, &shot);
        self.sway.step(moment, &stance, &shot, look);
        let bend = self.bend(target, &stance, shot.third, world);
        let seat = stance.frame * curve::seat(&shot, &bend);
        let pivot = occlude::pivot(world, target.position, seat);
        let swing = stance.frame * (self.sway.offset() + look * curve::boom(&shot, &bend));
        let probe = Probe::along(pivot, swing, stance.frame * Vec3::Y);
        let gap = self.place(probe, &stance, world);
        let lead =
            Quat::from_rotation_y(self.sway.lead.x) * Quat::from_rotation_x(self.sway.lead.y);
        let position = pivot + swing.normalize_or_zero() * gap;
        let head = target.position + stance.frame * Vec3::Y * HEAD_CENTRE;
        let pose = CameraPose {
            position,
            rotation: stance.frame * look * lead * Quat::from_rotation_z(-self.sway.roll.angle),
            fov: curve::fov(moment.fov, &shot, &bend),
        };
        Shown {
            pose,
            fade: BodyFade(curve::fade(shot.third, position.distance(head))),
            heading: self.aim.view,
        }
    }
}
