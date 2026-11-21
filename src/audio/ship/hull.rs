//! One ship as a voice: every source it has, driven by its real state, mixed twice. A listener
//! outside hears what it radiates into the air, after the path the sound takes to reach them. A
//! listener in the seat hears that same air through the canopy, and with it what the airframe
//! carries: the rumble of the engines, the shafts of the pumps, the mechanisms, the wind.
//!
//! In a vacuum nothing is radiated, so outside there is silence and inside only the airframe.

use super::airframe::{Airframe, Flow};
use super::cabin::Cabin;
use super::drive::Drive;
use super::feed::{CHANNELS, Ch, HullSound};
use super::jet::{Jet, JetCtl};
use super::law::{ground, lobe, pressure};
use super::path::Path;
use super::pump::Pump;
use super::tune::{
    AIR_CAP, CANOPY_HZ, DIAMETERS, FULL_THRUST, IDLE_THRUST, IMPINGE, LEAK, LIFT_BODY, LIFT_FEED,
    LIFT_LOBE, LIGHT, OUTRUN, PUMP_IDLE, SHOCK, SHOCK_NOISE, TRANSONIC,
};
use super::works::Works;
use crate::audio::dsp::filter::{Lag, glide, pole};
use crate::render::smoothstep;
use crate::speed::mach;

/// Seconds a value of the ship takes to cover 63 percent of a change. This removes zipper noise.
const SMOOTH_SECONDS: f32 = 0.04;

/// What the voice works out a few hundred times per second.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Mix {
    /// How far the chambers are lit, from 0 to 1.
    lit: f32,
    /// Speed asked of the pumps while the engines run.
    spin: f32,
    /// Coefficient of the low-pass the canopy makes.
    wall: f32,
    /// Gain of the air outside into the cabin.
    leak: f32,
}

/// The voice of one ship.
#[derive(Debug, Clone)]
pub struct Hull {
    /// What the game says of the ship.
    goal: HullSound,
    /// The same values, smoothed.
    now: [Lag; CHANNELS],
    /// The main engines.
    main: Jet,
    /// The lift nozzles.
    lift: Jet,
    /// The pumps.
    pump: Pump,
    /// The air on the airframe.
    flow: Airframe,
    /// The warp drive.
    drive: Drive,
    /// The mechanisms.
    pub(super) works: Works,
    /// The cabin bed.
    cabin: Cabin,
    /// The way to a listener outside.
    path: Path,
    /// The low-pass of the canopy.
    wall: Lag,
    /// What the last tuning worked out.
    mix: Mix,
    /// Coefficient of the smoothing.
    k: f32,
    /// Sample rate in hertz.
    rate: f32,
}

impl Hull {
    /// Builds a silent voice with its own noise `seed`, tuned every `dt` seconds.
    pub fn new(rate: f32, seed: u32, dt: f32) -> Self {
        let seed = seed.wrapping_mul(0x9E37_79B9) ^ 0x51A7_E0B3;
        Self {
            goal: HullSound::default(),
            now: HullSound::default().0.map(Lag),
            main: Jet::new(rate, seed ^ 0x0A11, DIAMETERS.0),
            lift: Jet::new(rate, seed ^ 0x11F7, DIAMETERS.1),
            pump: Pump::new(rate, seed ^ 0x9C3B),
            flow: Airframe::new(rate, seed ^ 0xA1F0),
            drive: Drive::new(rate, seed ^ 0xD21E, dt),
            works: Works::new(rate, seed ^ 0x3EC4, dt),
            cabin: Cabin::new(rate, seed ^ 0xCAB1),
            path: Path::new(rate),
            wall: Lag::default(),
            mix: Mix::default(),
            k: glide(SMOOTH_SECONDS, rate),
            rate,
        }
    }

    /// Returns true while the ship is there or still fading out.
    fn live(&self) -> bool {
        self.goal.at(Ch::Present) > 0.5 || self.now[Ch::Present as usize].0 > 0.0
    }

    /// Starts from the state of a ship that has just appeared, silent, so that nothing glides
    /// in from the state of the ship this voice played before.
    fn wake(&mut self) {
        self.now = self.goal.0.map(Lag);
        self.now[Ch::Present as usize] = Lag(0.0);
        self.works.rest(&self.goal);
        self.path.clear(&self.goal);
    }

    /// Aims the two kinds of jet: the main engines by where the listener stands around the
    /// exhaust, with the noise of their shock cells in air, and the lift nozzles by the ground.
    fn aim(&mut self, hull: &HullSound, lit: f32) {
        let (air, sound) = (hull.at(Ch::Air), hull.at(Ch::Sound));
        let toward = lobe(hull.at(Ch::Aspect));
        let shock = smoothstep(SHOCK.0, SHOCK.1, air) * (1.0 - SHOCK_NOISE.2 * toward);
        self.main.tune(&JetCtl {
            level: hull.at(Ch::Thrust).max(IDLE_THRUST * lit),
            air,
            sound,
            lobe: toward,
            extra_hz: SHOCK_NOISE.0 * sound / DIAMETERS.0,
            extra: shock * SHOCK_NOISE.1,
        });
        let near = ground(hull.at(Ch::Height));
        self.lift.tune(&JetCtl {
            level: hull.at(Ch::Lift),
            air,
            sound,
            lobe: LIFT_LOBE * (1.0 + IMPINGE.0 * near),
            extra_hz: IMPINGE.1,
            extra: near * IMPINGE.2,
        });
    }

    /// Sets what the air does to the airframe and to the cabin: the wind and the buffet, rougher
    /// around the speed of sound, the pumps, the canopy as a low-pass that opens with it, and the
    /// sound of the engines in the air, which a cabin faster than sound leaves behind.
    fn ride(&mut self, hull: &HullSound, lit: f32) {
        let (air, speed) = (hull.at(Ch::Air), hull.at(Ch::Speed));
        let fast = mach(speed, hull.at(Ch::Sound));
        let band = (fast - 1.0) / TRANSONIC.1;
        let sonic = TRANSONIC.0 * (-band * band).exp() * air.min(1.0);
        let feed = hull.at(Ch::Thrust).max(LIFT_FEED * hull.at(Ch::Lift)) / FULL_THRUST;
        let open = hull.at(Ch::Canopy);
        let behind = 1.0 - smoothstep(OUTRUN.0, OUTRUN.1, fast);
        self.flow.tune(&Flow {
            speed,
            pressure: pressure(air, speed),
            rough: hull.at(Ch::Rough).max(sonic),
            roar: hull.at(Ch::Roar),
        });
        self.mix = Mix {
            lit,
            spin: feed.min(1.0).sqrt().max(PUMP_IDLE),
            wall: pole(
                CANOPY_HZ.0 * (CANOPY_HZ.1 / CANOPY_HZ.0).powf(open),
                self.rate,
            ),
            leak: (LEAK.0 + (LEAK.1 - LEAK.0) * open) * behind,
        };
    }

    /// Takes what the game says of the ship and retunes every layer, `dt` seconds after the
    /// last call. Call it a few hundred times per second.
    ///
    /// The way to a listener outside follows where the game puts them, not the smoothed place,
    /// and only while they are outside: one who gets in hears the outside fade from where they
    /// stood, and one who gets out is heard at once from where they stand, while the outside
    /// is still silent. The seat sits at no distance, so a smoothed place would sweep the delay
    /// through it.
    pub fn tune(&mut self, goal: HullSound, dt: f32) {
        let was = self.goal;
        self.goal = goal;
        if was.at(Ch::Present) <= 0.5 && goal.at(Ch::Present) > 0.5 {
            self.wake();
        }
        if !self.live() {
            return;
        }
        let hull = HullSound(self.now.map(|lag| lag.0));
        let lit = smoothstep(LIGHT.0, LIGHT.1, self.pump.speed());
        if self.mix.lit < 0.5 && lit >= 0.5 {
            self.works.light();
        }
        self.aim(&hull, lit);
        self.ride(&hull, lit);
        self.drive.tune();
        self.works.tune(&hull, lit, dt);
        match (goal.at(Ch::Inside) > 0.5, was.at(Ch::Inside) > 0.5) {
            (true, _) => {}
            (false, true) => self.path.jump(&goal),
            (false, false) => self.path.tune(&goal),
        }
    }

    /// Renders one sample of what a listener in the seat hears: the `air` outside through the
    /// canopy, what the airframe carries in `body`, the wind, the drive and the cabin bed.
    fn seat(&mut self, hull: &HullSound, air: f32, body: f32) -> f32 {
        let through = self.wall.to(air * self.mix.leak, self.mix.wall);
        let ride = self.flow.frame() + self.drive.frame(hull.at(Ch::Warp));
        through + body + ride + self.cabin.frame(hull.at(Ch::Power))
    }

    /// Renders one stereo frame.
    pub fn frame(&mut self) -> [f32; 2] {
        if !self.live() {
            return [0.0; 2];
        }
        let k = self.k;
        for (now, goal) in self.now.iter_mut().zip(self.goal.0) {
            now.settle(goal, k);
        }
        let hull = HullSound(self.now.map(|lag| lag.0));
        let (jet, roar) = self.main.frame();
        let (wash, shake) = self.lift.frame();
        let (whine, shaft) = self.pump.frame(self.mix.spin * hull.at(Ch::Power));
        let (works, knocks) = self.works.frame();
        let air = (jet + wash + whine + works) * hull.at(Ch::Air).min(AIR_CAP);
        self.path.write(air);
        let inside = hull.at(Ch::Inside);
        let seat = match inside > 0.0 {
            true => self.seat(&hull, air, roar + LIFT_BODY * shake + shaft + knocks),
            false => 0.0,
        };
        let outside = match inside < 1.0 {
            true => self.path.read(),
            false => [0.0; 2],
        };
        let level = hull.at(Ch::Present);
        outside.map(|side| (side * (1.0 - inside) + seat * inside) * level)
    }
}
