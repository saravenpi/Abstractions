//! Multi-phase boss state and special-attack descriptors.

use bevy::prelude::*;

/// Base seconds between boss special attacks in Phase 1.
const BASE_CD: f32 = 7.5;

/// Special attack a boss titan can telegraph and unleash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BossSpecial {
    /// Windup warning ring followed by a ground shockwave dodged by jumping over 1.1 m.
    SeismicSlam,
    /// Brief telegraph followed by a 1.45x sprint charge and a 1.4 s recovery punish window.
    RampageCharge,
    /// Windup followed by 3 (Phase 1) or 5 (Phase 2) arcing bioluminescent projectiles.
    BioBarrage,
}

impl BossSpecial {
    /// Returns the uppercase HUD callout label for this special attack.
    pub fn label(self) -> &'static str {
        match self {
            BossSpecial::SeismicSlam => "SEISMIC SLAM",
            BossSpecial::RampageCharge => "RAMPAGE CHARGE",
            BossSpecial::BioBarrage => "BIO BARRAGE",
        }
    }

    /// Returns the telegraph windup duration in seconds before impact or launch.
    pub fn windup(self) -> f32 {
        match self {
            BossSpecial::SeismicSlam => 1.15,
            BossSpecial::RampageCharge => 0.9,
            BossSpecial::BioBarrage => 0.85,
        }
    }
}

/// Multi-phase combat state attached to every spawned boss beast.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct BossMind {
    /// Combat phase: 1 above half health, 2 once enraged below 50% HP.
    pub phase: u8,
    /// Seconds remaining before the next special attack may start.
    pub special_cd: f32,
    /// Currently executing special attack, if any.
    pub active: Option<BossSpecial>,
    /// Seconds elapsed in the active special attack.
    pub timer: f32,
    /// Locked horizontal direction of the active special in the beast's local frame.
    pub target_dir: Vec3,
    /// Origin of the active special in the beast's local frame.
    pub origin: Vec3,
}

impl BossMind {
    /// Returns a fresh Phase 1 boss mind.
    pub fn new() -> Self {
        Self {
            phase: 1,
            special_cd: 4.5,
            active: None,
            timer: 0.0,
            target_dir: Vec3::NEG_Z,
            origin: Vec3::ZERO,
        }
    }

    /// Transitions to Phase 2 enrage when `hp_share < 0.5`, shortening cooldown by 30%, and
    /// returns true on the transition frame.
    pub fn check_enrage(&mut self, hp_share: f32) -> bool {
        if self.phase == 1 && hp_share < 0.5 {
            self.phase = 2;
            self.special_cd *= 0.7;
            return true;
        }
        false
    }

    /// Returns the special cooldown in seconds for the current phase.
    pub fn cooldown(&self) -> f32 {
        if self.phase >= 2 {
            BASE_CD * 0.7
        } else {
            BASE_CD
        }
    }

    /// Returns the number of projectiles fired by [`BossSpecial::BioBarrage`] in the current phase.
    pub fn barrage_count(&self) -> usize {
        if self.phase >= 2 { 5 } else { 3 }
    }

    /// Returns the active special while it is still inside its telegraph windup window.
    pub fn winding(&self) -> Option<BossSpecial> {
        self.active.filter(|s| self.timer < s.windup())
    }
}

impl Default for BossMind {
    fn default() -> Self {
        Self::new()
    }
}
