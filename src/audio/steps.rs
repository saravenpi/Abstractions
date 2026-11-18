//! Footsteps: one cue per stride while the player walks on the ground.

use bevy::prelude::*;

use super::cue::{AudioCue, StepSurface};
use crate::player::{BODY_LENGTH, BODY_RADIUS, Player};
use crate::screens::playing;
use crate::space::{Anchor, Location, Travel, up};
use crate::world::structures::{FloorKind, Surroundings};
use crate::world::{Surface, Terrain};

/// Metres walked between two footsteps at a standstill start.
pub const STRIDE: f32 = 1.7;

/// Metres added to the stride for every metre per second of speed: a run takes longer steps.
pub const STRIDE_PER_SPEED: f32 = 0.11;

/// Largest height in metres of the feet above the ground that still counts as walking.
const CONTACT: f32 = 0.6;

/// Depth in metres of water above which steps are no longer heard.
const WADE_DEPTH: f32 = 0.5;

/// Largest move in metres in one frame that counts as walking. Anything longer is a jump of
/// the frame of reference or a teleport.
const MAX_MOVE: f32 = 2.0;

/// What the footstep system remembers between frames.
#[derive(Debug, Default)]
struct Walk {
    /// Position of the body on the previous frame.
    last: Option<Vec3>,
    /// Metres walked since the last footstep.
    walked: f32,
}

/// Returns the footstep sound of a ground cover, or nothing for a sea floor.
pub(super) fn step_surface(surface: Surface) -> Option<StepSurface> {
    match surface {
        Surface::Sand | Surface::Shallows | Surface::Pond => Some(StepSurface::Sand),
        Surface::Grass => Some(StepSurface::Grass),
        Surface::Rock => Some(StepSurface::Rock),
        Surface::Snow => Some(StepSurface::Snow),
        Surface::Water | Surface::Reef => None,
    }
}

/// Returns the footstep sound of a built floor: planks knock, masonry rings like rock, and
/// sandstone and packed earth are as dull as sand.
pub(super) fn floor_surface(floor: FloorKind) -> StepSurface {
    match floor {
        FloorKind::Wood => StepSurface::Wood,
        FloorKind::Stone => StepSurface::Rock,
        FloorKind::Sand | FloorKind::Earth => StepSurface::Sand,
    }
}

/// Returns the ground cover under a body at chart position `at`, when its feet touch dry or
/// shallow ground. Where the globe has no sea, ground below sea level is as dry as any.
pub(super) fn footing(terrain: &Terrain, at: Vec3) -> Option<StepSurface> {
    let feet = at.y - 0.5 * BODY_LENGTH - BODY_RADIUS;
    let ground = terrain.height(at.x, at.z);
    let touching = feet - ground < CONTACT && feet - terrain.globe().sea_level() > -WADE_DEPTH;
    step_surface(terrain.surface(at.x, at.z)).filter(|_| touching)
}

/// Adds the distance walked on the ground or on a built floor this frame and writes a cue at
/// every stride.
fn footsteps(
    time: Res<Time>,
    world: (
        Res<Location>,
        Res<Anchor>,
        Res<Terrain>,
        Option<Res<Surroundings>>,
    ),
    players: Query<&Transform, With<Player>>,
    mut walk: Local<Walk>,
    mut cues: MessageWriter<AudioCue>,
) {
    let (location, anchor, terrain, floors) = world;
    let (Ok(body), Travel::OnFoot) = (players.single(), location.travel) else {
        walk.last = None;
        return;
    };
    let pos = body.translation;
    let moved = pos - walk.last.replace(pos).unwrap_or(pos);
    let flat = moved - up(&anchor, pos) * moved.dot(up(&anchor, pos));
    let feet = pos - up(&anchor, pos) * (0.5 * BODY_LENGTH + BODY_RADIUS);
    let built = floors.and_then(|floors| floors.floor_at(anchor.to_frame(feet)));
    let ground = || footing(&terrain, terrain.locate(anchor.to_frame(pos)));
    let surface = built.map(floor_surface).or_else(ground);
    let (Some(surface), true) = (surface, flat.length() < MAX_MOVE) else {
        return;
    };
    walk.walked += flat.length();
    let speed = flat.length() / time.delta_secs().max(1e-4);
    if walk.walked >= STRIDE + STRIDE_PER_SPEED * speed {
        walk.walked = 0.0;
        cues.write(AudioCue::Footstep(surface));
    }
}

/// Registers the footstep system, which runs while the game is played.
pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        footsteps
            .run_if(playing)
            .run_if(resource_exists::<Location>)
            .run_if(resource_exists::<Anchor>)
            .run_if(resource_exists::<Terrain>),
    );
}
