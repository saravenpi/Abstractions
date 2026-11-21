//! Numbers of the cockpit: where the controls and the instruments stand in the axes of the
//! hull (the nose along -Z, the roof along +Y), how fast they follow, and the light.

use std::ops::Range;

use bevy::prelude::Vec3;

/// Height of the floor of the tub.
pub const FLOOR: f32 = -0.34;

/// Height of the top of the seat pan: the pilot sits on it.
pub const PAN: f32 = 0.06;

/// Where the front of the backrest and of the headrest is along z: the back of the pilot.
pub const BACK: f32 = -0.975;

/// Where the side stick hinges, on the front of the right console, where the hand of the pilot
/// reaches it and the eye sees its grip at the foot of the view.
pub const STICK: Vec3 = Vec3::new(0.3, 0.31, -1.74);

/// Length of the stick from its hinge to the top of its grip, and how far under the top the
/// hand holds it, in metres.
pub const STICK_LENGTH: (f32, f32) = (0.24, 0.045);

/// Tilt of the stick at full pitch or roll, and its twist at full yaw, in radians.
pub const STICK_REACH: (f32, f32) = (0.18, 0.3);

/// Where the throttle lever hinges, on the front of the left console.
pub const LEVER: Vec3 = Vec3::new(-0.3, 0.31, -1.72);

/// Length of the throttle lever from its hinge to the top of its handle, and how far under the
/// top the hand holds it, in metres.
pub const LEVER_LENGTH: (f32, f32) = (0.2, 0.025);

/// Tilt of the lever from upright, in radians: back by that much at idle, forward at full.
pub const LEVER_SWING: f32 = 0.35;

/// Where the gear lever hinges, on top of the right console.
pub const GEAR_LEVER: Vec3 = Vec3::new(0.29, 0.25, -1.45);

/// Where the warp switch hinges, on top of the right console.
pub const WARP_SWITCH: Vec3 = Vec3::new(0.29, 0.25, -1.25);

/// Tilt of the gear lever and of the warp switch from upright, in radians: back when the gear
/// is up and the drive off, forward when the gear is down and the drive engaged.
pub const THROW: (f32, f32) = (0.4, 0.45);

/// Length of the gear lever and of the warp switch from hinge to tip, in metres.
pub const THROW_LENGTH: (f32, f32) = (0.16, 0.07);

/// Where the heel of each rudder pedal hinges: x of the right pedal, height and z. The left
/// pedal mirrors it. The soles of the seated body stand on them, and the hinge is under the ankle
/// of the seated body, so that a boot and its pedal turn about the same axis.
pub const PEDAL: Vec3 = Vec3::new(0.095, -0.347, -1.62);

/// Length of a pedal from its heel to its toe, in metres.
pub const PEDAL_LENGTH: f32 = 0.26;

/// Tilt of a pedal at full yaw, in radians. The boot turns by the same angle.
pub const PEDAL_REACH: f32 = 0.2;

/// Where the loose strap hangs from, beside the seat, and its length in metres.
pub const STRAP: (Vec3, f32) = (Vec3::new(-0.21, 0.52, BACK - 0.012), 0.2);

/// Rate per second at which the controls follow what is asked of them, so that a key that
/// goes down in one frame moves the stick over a few. The time constant is 1/14 s.
pub const FOLLOW: f32 = 14.0;

/// Rate per second at which the needles follow what they measure: a first-order lag with a
/// time constant of 1/8 s, so a needle sweeps to a new value in about half a second.
pub const SETTLE: f32 = 8.0;

/// Natural pulsation in radians per second and damping ratio of the loose strap on its hinge,
/// a pendulum a hand long that swings a few times after a push.
pub const SWING: (f32, f32) = (11.0, 0.25);

/// Smallest pull in metres per second squared that the strap hangs along: under it, in free
/// fall, the strap floats at rest instead of whipping about.
pub const SLACK: f32 = 2.0;

/// Longest step in seconds the strap is integrated over, so a long frame cannot throw it.
pub const SPRING_STEP: f32 = 1.0 / 30.0;

/// Centre of the raked instrument panel, and its half width across and half length up the
/// slope, in metres. Its top edge is under the rim of the hull, so the nose shows over it, and
/// the panel faces the eye of the pilot.
pub const RAKE: (Vec3, f32, f32) = (Vec3::new(0.0, 0.331, -2.052), 0.31, 0.17);
/// Tilt of the raked panel from upright, in radians: its face looks up at the pilot.
pub const RAKE_TILT: f32 = 0.611;
/// Side of a pixel of the displays on the panel, in metres: about one pixel of the game image
/// at the distance of the eye, so a capital of the font is seven of them tall.
pub const TEXEL: f32 = 0.004;
/// The displays of the panel from left to right, the drive, the flight and the status: where
/// the middle of each is across the panel, and its width, in metres. All are as tall as the
/// flight display is wide.
pub const BAYS: [(f32, f32); 3] = [(-0.228, 0.128), (0.0, 0.288), (0.228, 0.128)];
/// Height in metres at which the altitude bar is full. It reads on a logarithmic scale, so a
/// hand over the ground and a high orbit both show.
pub const ALTITUDE_SPAN: f64 = 100_000.0;

/// Steps per unit of the natural logarithm of a speed or height shown as text: a text is
/// repainted when its value moved by half a percent, finer than the digits show.
pub const TEXT_STEPS: f64 = 200.0;

/// Step in radians of the attitude display and of the scope: a repaint when the horizon moved
/// by this much, about three degrees.
pub const ANGLE_STEP: f32 = 0.05;

/// Heat share of the glow of an entry at which the heat lamp lights.
pub const HOT_LAMP: f32 = 0.1;

/// Integrity of the frame under which the frame lamp lights, and under which the failing lamp
/// lights, from 0 to 1. They are the thresholds of the warning and the danger of the HUD.
pub const FRAME_LAMPS: (f32, f32) = (0.5, 0.25);

/// Luminous power in lumens of the panel glow and of the cabin light, and of the glow of an
/// entry at full heat. The lights cast no shadow.
pub const LUMENS: (f32, f32, f32) = (240.0, 700.0, 1200.0);

/// Range in metres of the panel glow and of the cabin light.
pub const REACH: (f32, f32) = (1.6, 3.2);

/// Share of the panel glow and of the cabin light that full day takes away.
pub const DAY_DIM: (f32, f32) = (0.8, 0.6);

/// Brightness of the lit faces of the instruments in full day and at night, as a share of
/// white. Unlit white would wash out in the light of the day.
pub const SCREEN_GLOW: (f32, f32) = (0.8, 1.0);

/// Distances in metres from the camera to the cockpit between which it is drawn: past the
/// second it is hidden, between them it fades out.
pub const DRAW: Range<f32> = 70.0..90.0;

/// Where the canopy hinges: across the back of the mouth, on the line of the roof of the hull.
pub const HINGE: Vec3 = Vec3::new(0.0, 1.27, -0.85);

/// How far the canopy opens, in radians, and how fast it swings, in radians per second.
pub const CANOPY_OPEN: (f32, f32) = (1.0, 0.7);
