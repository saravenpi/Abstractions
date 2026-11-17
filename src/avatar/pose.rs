//! A pose: every joint angle of the body as plain numbers, which blend by plain weighted sums.

/// The joints of the trunk and the head.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Core {
    /// Height of the pelvis above its rest height, in meters.
    Lift,
    /// Sideways shift of the pelvis toward the right, in meters.
    Sway,
    /// Forward lean of the whole body: 0 upright, a quarter turn prone.
    Pitch,
    /// Roll of the whole body about its direction of travel: positive drops the right side.
    Roll,
    /// Forward bend of the torso over the pelvis.
    Spine,
    /// Turn of the torso to the left over the pelvis.
    Twist,
    /// Turn of the head to the left.
    HeadYaw,
    /// Nod of the head, positive up.
    HeadPitch,
}

/// The joints of one side of the body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Limb {
    /// Swing of the upper arm, positive forward. Half a turn points it overhead.
    Shoulder,
    /// Lift of the upper arm away from the body, to the side.
    ArmOut,
    /// Bend of the elbow, never negative: the hand comes forward.
    Elbow,
    /// Swing of the thigh, positive forward.
    Hip,
    /// Lift of the thigh away from the other leg, to the side.
    LegOut,
    /// Bend of the knee, never negative: the shin folds back.
    Knee,
    /// Pitch of the boot, positive points the toes down.
    Ankle,
}

/// Number of trunk joints.
pub const CORES: usize = 8;

/// Number of joints per side.
pub const LIMBS: usize = 7;

/// Lowest and highest value of every trunk joint, in the order of [`Core`]. No pose leaves them,
/// and the blended pose is held inside them whatever the input.
pub const CORE_LIMITS: [(f32, f32); CORES] = [
    (-0.45, 0.1),
    (-0.05, 0.05),
    (-0.6, 2.3),
    (-1.1, 1.1),
    (-0.5, 0.6),
    (-0.5, 0.5),
    (-1.3, 1.3),
    (-1.0, 1.3),
];

/// Lowest and highest value of every side joint, in the order of [`Limb`]. No pose leaves them:
/// elbows and knees never bend backwards.
pub const LIMB_LIMITS: [(f32, f32); LIMBS] = [
    (-1.3, 3.2),
    (-0.1, 1.8),
    (0.0, 2.4),
    (-1.0, 1.9),
    (-0.2, 1.0),
    (0.0, 2.5),
    (-1.2, 1.3),
];

/// The value of every joint. Side 0 is the left of the body (-X), side 1 the right (+X).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Pose {
    /// Trunk joints, indexed by [`Core`].
    pub core: [f32; CORES],
    /// Side joints, indexed by side then by [`Limb`].
    pub sides: [[f32; LIMBS]; 2],
}

impl Pose {
    /// Returns the value of a trunk joint.
    pub fn core(&self, joint: Core) -> f32 {
        self.core[joint as usize]
    }

    /// Returns the value of a joint of `side`.
    pub fn limb(&self, joint: Limb, side: usize) -> f32 {
        self.sides[side][joint as usize]
    }

    /// Returns the pose with a trunk joint set.
    pub fn with(mut self, joint: Core, value: f32) -> Self {
        self.core[joint as usize] = value;
        self
    }

    /// Returns the pose with a joint set on the left and on the right.
    pub fn pair(mut self, joint: Limb, left: f32, right: f32) -> Self {
        self.sides[0][joint as usize] = left;
        self.sides[1][joint as usize] = right;
        self
    }

    /// Returns the pose with a joint set to the same value on both sides.
    pub fn both(self, joint: Limb, value: f32) -> Self {
        self.pair(joint, value, value)
    }

    /// Returns every value in one row: the trunk, the left side, the right side.
    #[cfg(test)]
    pub fn values(&self) -> [f32; CORES + 2 * LIMBS] {
        let mut row = [0.0; CORES + 2 * LIMBS];
        row[..CORES].copy_from_slice(&self.core);
        row[CORES..CORES + LIMBS].copy_from_slice(&self.sides[0]);
        row[CORES + LIMBS..].copy_from_slice(&self.sides[1]);
        row
    }

    /// Returns this pose plus `other` weighted by `weight`, joint by joint.
    pub fn plus(mut self, other: &Pose, weight: f32) -> Self {
        for (mine, theirs) in self.core.iter_mut().zip(other.core) {
            *mine += theirs * weight;
        }
        for (mine, theirs) in self
            .sides
            .iter_mut()
            .flatten()
            .zip(other.sides.as_flattened())
        {
            *mine += theirs * weight;
        }
        self
    }

    /// Returns the pose with every joint inside its limits, and zero for a value that is not a number.
    pub fn bounded(mut self) -> Self {
        let keep = |value: f32, (low, high): (f32, f32)| match value.is_finite() {
            true => value.clamp(low, high),
            false => 0.0_f32.clamp(low, high),
        };
        for (value, limits) in self.core.iter_mut().zip(CORE_LIMITS) {
            *value = keep(*value, limits);
        }
        for side in &mut self.sides {
            for (value, limits) in side.iter_mut().zip(LIMB_LIMITS) {
                *value = keep(*value, limits);
            }
        }
        self
    }
}
