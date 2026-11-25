//! Multi-phase boss AI and special attacks: seismic slams, rampage charges and bioluminescent
//! barrages, with a Phase 2 enrage below 50% health.

use bevy::ecs::system::SystemParam;
use bevy::math::Vec3Swizzles;
use bevy::prelude::*;

use super::Dead;
use super::boss_fx::{rumble, spawn_barrage, spawn_ring};
pub use super::boss_mind::{BossMind, BossSpecial};
use crate::audio::{AudioCue, Impact};
use crate::avatar::blow::Blow;
use crate::fauna::Frozen;
use crate::fauna::beast::Beast;
use crate::fauna::senses::Stance;
use crate::fauna::tune::MAX_STEP;
use crate::life::{Reaction, Species};
use crate::player::Player;
use crate::space::Anchor;
use crate::world::Terrain;

/// World context read and written by the boss special-attack system.
#[derive(SystemParam)]
pub struct BossCtx<'w, 's> {
    commands: Commands<'w, 's>,
    time: Res<'w, Time>,
    terrain: Res<'w, Terrain>,
    anchor: Res<'w, Anchor>,
    robot: Query<'w, 's, &'static Transform, With<Player>>,
    meshes: ResMut<'w, Assets<Mesh>>,
    materials: ResMut<'w, Assets<StandardMaterial>>,
    blows: MessageWriter<'w, Blow>,
    sounds: MessageWriter<'w, Impact>,
    cues: MessageWriter<'w, AudioCue>,
}

/// Living, unfrozen boss beasts.
type Alive = (Without<Dead>, Without<Frozen>);

/// Selects the next boss special from horizontal distance `dist` and `length`.
pub fn pick_special(dist: f32, length: f32) -> BossSpecial {
    if dist <= 1.35 * length {
        BossSpecial::SeismicSlam
    } else if dist <= 28.0 + length {
        BossSpecial::RampageCharge
    } else {
        BossSpecial::BioBarrage
    }
}

/// Runs one frame of multi-phase boss AI for every living boss beast.
pub fn think(mut ctx: BossCtx, mut bosses: Query<(&mut Beast, &mut BossMind), Alive>) {
    let globe = ctx.terrain.globe().clone();
    let (Some(life), Some(robot_pos)) =
        (globe.life(), ctx.robot.single().ok().map(|t| t.translation))
    else {
        return;
    };
    let dt = ctx.time.delta_secs().min(MAX_STEP);
    let robot_planet = ctx.anchor.to_frame(robot_pos);
    for (mut beast, mut mind) in &mut bosses {
        let Some(species) = life.species.get(usize::from(beast.index)) else {
            continue;
        };
        let boss_render = ctx
            .anchor
            .to_render(beast.spot.planet(beast.motion.root.at));
        if mind.check_enrage(beast.wits.health.share()) {
            rumble(&mut ctx.sounds, boss_render, 1200.0);
            ctx.cues.write(AudioCue::Land(1.0));
        }
        let robot_local = beast.spot.local(robot_planet);
        let gap = robot_local - beast.motion.root.at;
        if mind.active.is_none() {
            maybe_start(
                &mut ctx,
                (&beast, &mut mind),
                species,
                (gap, boss_render, dt),
            );
        } else {
            step_special(
                &mut ctx,
                (&mut beast, &mut mind),
                species,
                (robot_local, robot_pos, boss_render, dt),
            );
        }
    }
}

/// Counts down cooldown and starts a boss special when engaged and within range.
fn maybe_start(
    ctx: &mut BossCtx,
    (beast, mind): (&Beast, &mut BossMind),
    species: &Species,
    (gap, boss_render, dt): (Vec3, Vec3, f32),
) {
    mind.special_cd = (mind.special_cd - dt).max(0.0);
    let dist = gap.xz().length();
    let engaged = matches!(beast.wits.stance, Stance::Attack | Stance::Threaten);
    if !engaged || mind.special_cd > 0.0 || dist > 60.0 + species.body.length {
        return;
    }
    let special = pick_special(dist, species.body.length);
    mind.active = Some(special);
    mind.timer = 0.0;
    mind.origin = beast.motion.root.at;
    mind.target_dir = gap.with_y(0.0).normalize_or(Vec3::NEG_Z);
    if special == BossSpecial::SeismicSlam {
        let up = ctx.anchor.rotation_to_render(beast.spot.rot) * Vec3::Y;
        spawn_ring(
            &mut ctx.commands,
            (&mut ctx.meshes, &mut ctx.materials),
            (boss_render, up),
            (special.windup(), 1.35 * species.body.length),
        );
    }
}

/// Advances the active boss special attack through windup, execution and recovery.
fn step_special(
    ctx: &mut BossCtx,
    (beast, mind): (&mut Beast, &mut BossMind),
    species: &Species,
    (robot_local, robot_render, boss_render, dt): (Vec3, Vec3, Vec3, f32),
) {
    mind.timer += dt;
    match mind.active {
        Some(BossSpecial::SeismicSlam) if mind.timer >= BossSpecial::SeismicSlam.windup() => {
            fire_slam(
                ctx,
                (beast, mind),
                species,
                (robot_local, robot_render, boss_render),
            );
        }
        Some(BossSpecial::RampageCharge) => {
            step_charge(ctx, (beast, mind), species, (robot_local, robot_render));
        }
        Some(BossSpecial::BioBarrage) if mind.timer >= BossSpecial::BioBarrage.windup() => {
            fire_barrage(ctx, (beast, mind), species, boss_render);
        }
        _ => {}
    }
}

/// Detonates [`BossSpecial::SeismicSlam`], damaging grounded players within shockwave radius.
fn fire_slam(
    ctx: &mut BossCtx,
    (beast, mind): (&Beast, &mut BossMind),
    species: &Species,
    (robot_local, robot_render, boss_render): (Vec3, Vec3, Vec3),
) {
    rumble(&mut ctx.sounds, boss_render, 1500.0);
    let flat = (robot_local - beast.motion.root.at).xz().length();
    let grounded = robot_local.y <= beast.motion.root.at.y + 1.1;
    if flat <= 1.35 * species.body.length && grounded {
        let away = (robot_render - boss_render).normalize_or(Vec3::Y);
        ctx.blows.write(Blow {
            at: robot_render,
            joules: 420.0,
            cut: false,
            dir: away,
            momentum: 320.0,
        });
    }
    finish_special(mind);
}

/// Launches [`BossSpecial::BioBarrage`] arcing projectiles tinted by primary tissue reaction.
fn fire_barrage(
    ctx: &mut BossCtx,
    (beast, mind): (&Beast, &mut BossMind),
    species: &Species,
    boss_render: Vec3,
) {
    let rot = ctx.anchor.rotation_to_render(beast.spot.rot);
    let (dir, up) = (rot * mind.target_dir, rot * Vec3::Y);
    let launch = boss_render + up * (species.body.height * 0.9);
    let reaction = species
        .tissues
        .first()
        .map_or(Reaction::Inert, |t| t.reaction);
    spawn_barrage(
        &mut ctx.commands,
        (&mut ctx.meshes, &mut ctx.materials),
        (launch, dir, up),
        (mind.barrage_count(), reaction, species.history.mass),
    );
    finish_special(mind);
}

/// Drives [`BossSpecial::RampageCharge`] through sprint and 1.4 s recovery punish window.
fn step_charge(
    ctx: &mut BossCtx,
    (beast, mind): (&mut Beast, &mut BossMind),
    species: &Species,
    (robot_local, robot_render): (Vec3, Vec3),
) {
    let wind = BossSpecial::RampageCharge.windup();
    let charge_end = wind + 1.5;
    if mind.timer >= charge_end + 1.4 {
        finish_special(mind);
    } else if mind.timer >= charge_end {
        beast.motion.root.vel.at = Vec3::ZERO;
        beast.motion.root.vel.vel = Vec3::ZERO;
    } else if mind.timer >= wind {
        beast.motion.root.vel.at = mind.target_dir * (species.history.speed * 1.45);
        beast.motion.root.vel.vel = Vec3::ZERO;
        let flat = (robot_local - beast.motion.root.at).xz().length();
        if mind.origin.y > -1e4 && flat <= 0.65 * species.body.length + 1.5 {
            mind.origin.y = -1e5;
            let away = ctx.anchor.rotation_to_render(beast.spot.rot) * mind.target_dir;
            rumble(&mut ctx.sounds, robot_render, 1400.0);
            ctx.blows.write(Blow {
                at: robot_render,
                joules: 520.0,
                cut: false,
                dir: away,
                momentum: 420.0,
            });
        }
    }
}

/// Clears the active special and arms the phase-scaled cooldown.
fn finish_special(mind: &mut BossMind) {
    mind.active = None;
    mind.timer = 0.0;
    mind.special_cd = mind.cooldown();
}
