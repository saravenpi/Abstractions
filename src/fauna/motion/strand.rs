//! A body chain in motion: its joints and how each kind of chain follows the body.
//!
//! A spine is pulled along behind its root, so a wave put into its root travels down it. A neck
//! reaches by FABRIK for where its tip rests, lagging a little. Tails and tentacles lag on springs
//! toward their rest shape, swayed by a travelling sine whose amplitude grows to the tip. Necks
//! and tails are carried along with their anchor first, so they lag only when the body turns,
//! bobs or changes speed; tentacles trail in the open. Every chain ends with a follow pass, which
//! keeps its lengths and its bend limit.

use std::f32::consts::TAU;

use bevy::math::Vec3;

use super::chain::{follow, lag};
use super::frame::Kind;
use super::ik::fabrik;
use super::spring::Spring;
use crate::life::body::Chain;

/// The joints of one chain and their velocities.
#[derive(Debug, Clone, PartialEq)]
pub struct Strand {
    /// Which chain it is.
    pub kind: Kind,
    /// Joints from root to tip.
    pub points: Vec<Vec3>,
    /// Velocity of each joint, for the springs.
    vels: Vec<Vec3>,
}

/// What a strand follows this step.
#[derive(Debug, Clone, Copy)]
pub struct Pull<'a> {
    /// Rest joints under the body now, the first one where the chain is attached.
    pub rest: &'a [Vec3],
    /// Axis of the part the chain hangs from, which its first segment bends from.
    pub base: Vec3,
    /// Direction and size in metres of the sway at the tip.
    pub sway: Vec3,
    /// Phase of the sway in turns.
    pub phase: f32,
    /// Spring of the lag.
    pub spring: Spring,
}

/// Returns `rest` swayed sideways along `sway` by a sine travelling from root to tip at `phase`.
fn swayed(rest: &[Vec3], sway: Vec3, phase: f32) -> Vec<Vec3> {
    let last = rest.len().saturating_sub(1).max(1) as f32;
    let wave = |(i, p): (usize, &Vec3)| {
        let along = i as f32 / last;
        *p + sway * (along * along * (TAU * (phase - 0.6 * along)).sin())
    };
    rest.iter().enumerate().map(wave).collect()
}

impl Strand {
    /// Returns a strand of `kind` resting at `points`.
    pub fn new(kind: Kind, points: Vec<Vec3>) -> Self {
        let vels = vec![Vec3::ZERO; points.len()];
        Self { kind, points, vels }
    }

    /// Moves the strand along `chain` toward `pull` by `dt` seconds.
    pub fn step(&mut self, chain: &Chain, pull: &Pull, dt: f32) {
        let Some(&anchor) = pull.rest.first() else {
            return;
        };
        if self.points.len() < 2 || pull.rest.len() != self.points.len() {
            return;
        }
        if matches!(self.kind, Kind::Neck | Kind::Tail) {
            let shift = anchor - self.points[0];
            self.points.iter_mut().for_each(|p| *p += shift);
        }
        match self.kind {
            Kind::Spine => {}
            Kind::Neck => self.reach(chain, pull, dt),
            Kind::Tail | Kind::Tentacle => {
                let targets = swayed(pull.rest, pull.sway, pull.phase);
                lag(&mut self.points, &mut self.vels, &targets, &pull.spring, dt);
            }
        }
        follow(
            &mut self.points,
            &chain.lengths,
            (anchor, pull.base),
            chain.limit,
        );
    }

    /// Lags the tip toward its rest place and bends the chain by FABRIK to reach it from its anchor.
    fn reach(&mut self, chain: &Chain, pull: &Pull, dt: f32) {
        let tip = self.points.len() - 1;
        let targets = [Vec3::ZERO, pull.rest[tip]];
        lag(
            &mut self.points[tip - 1..],
            &mut self.vels[tip - 1..],
            &targets,
            &pull.spring,
            dt,
        );
        let reach = self.points[tip];
        self.points[0] = pull.rest[0];
        fabrik(&mut self.points, &chain.lengths, reach, 3);
    }
}
