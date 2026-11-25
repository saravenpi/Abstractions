//! Tests of colossal boss species, cave-adapted species, oriented spine & under-belly collision,
//! multi-phase boss AI, and closest-part melee hit selection.

use bevy::camera::primitives::Aabb;
use bevy::ecs::system::SystemState;
use bevy::math::{DVec3, Vec2, Vec3};
use bevy::prelude::*;

use super::attack::{BossMind, BossSpecial};
use super::bulk::capsule;
use super::collide::math::push_from_robot;
use super::place::Spot;
use super::react::Temper;
use super::rig::planet;
use super::senses::Health;
use super::senses::stance::{Foe, Me, Stance, choose};
use super::senses::tune::ROBOT_KG;
use super::{Creature, Live, Part};
use crate::avatar::strike::sense::{Parts, first};
use crate::life::{Activity, Guild, Rung, SpeciesId};
use crate::player::BODY_RADIUS;

/// Living skeletal worlds promote solitary boss titans with boosted health, 120 s grudge and
/// fearless combat stances, plus compact subterranean cave species.
#[test]
fn boss_and_cave_species_emerge_with_expected_traits() {
    let (_, _, life) = planet(true);
    let life = life.expect("living planet");
    assert!(life.rung >= Rung::Shelled);
    let boss = life.species.iter().find(|s| s.boss).expect("boss species");
    assert_eq!((boss.history.group_min, boss.history.group_max), (1, 1));
    assert!(boss.history.mass >= 7_000.0 && boss.body.length >= 8.0);
    assert!(boss.glow.is_some() && !boss.epithet.is_empty());
    assert!((Temper::of(boss).grudge - 120.0).abs() < 1e-4);
    let normal_hp = Health::of(boss.history.mass).max;
    assert!((Health::of_species(boss).max / normal_hp - 2.5).abs() < 1e-4);
    let cave = life.species.iter().find(|s| s.cave).expect("cave species");
    assert!(cave.body.height <= 3.2 && cave.role.activity == Activity::Night);
    assert!(cave.glow.is_some());
}

/// Boss stances never flee even at low health, and boss predators actively hunt robots.
#[test]
fn boss_stance_never_flees_and_hunts_players() {
    let robot = Foe {
        mass: ROBOT_KG,
        robot: true,
    };
    let hurt_grazer_boss = Me {
        mass: 20_000.0,
        health: 0.1,
        guild: Guild::Grazer,
        hungry: false,
        cornered: false,
        guarding: false,
        grace: false,
        provoked: false,
        boss: true,
    };
    assert_eq!(choose(1.0, &hurt_grazer_boss, &robot), Stance::Threaten);
    let predator_boss = Me {
        guild: Guild::Hunter,
        ..hurt_grazer_boss
    };
    assert_eq!(choose(0.4, &predator_boss, &robot), Stance::Attack);
}

/// Oriented spine collision keeps nose clearance at half-length, flank clearance at torso radius,
/// and skips horizontal push when the robot fits cleanly under a tall giant's belly.
#[test]
fn oriented_spine_flank_nose_and_under_belly_collision() {
    let (_, globe, life) = planet(true);
    let life = life.expect("living planet");
    let spot = Spot::at(DVec3::Y, globe.radius());
    let mut body = life.species.iter().find(|s| s.boss).unwrap().body.clone();
    body.length = 12.0;
    body.height = 1.8;
    let (radius, _) = capsule(body.length, body.height);
    let nose_robot = spot.planet(Vec3::new(0.0, 0.8, 0.0));
    let nose_at = Vec3::new(5.5, 0.0, 0.0);
    let nose_push = push_from_robot(&spot, (nose_at, Vec2::NEG_X), &body, nose_robot)
        .expect("nose overlaps robot");
    let resolved_x = nose_at.x + nose_push.x;
    assert!((resolved_x - (0.5 * body.length + BODY_RADIUS)).abs() < 1e-3);
    let flank_robot = spot.planet(Vec3::new(0.0, 0.8, radius + BODY_RADIUS + 0.4));
    assert!(
        push_from_robot(&spot, (Vec3::ZERO, Vec2::X), &body, flank_robot).is_none(),
        "flank outside torso radius must not push"
    );
    body.height = 6.0;
    let under_robot = spot.planet(Vec3::new(0.0, 0.9, 0.0));
    assert!(
        push_from_robot(&spot, (Vec3::ZERO, Vec2::X), &body, under_robot).is_none(),
        "robot under tall torso underside must pass freely"
    );
}

/// Phase 2 enrage triggers below 50% HP, shortens special cooldown by 30%, upgrades BioBarrage
/// from 3 to 5 projectiles, and exposes active windup labels.
#[test]
fn boss_mind_phase_two_enrage_and_specials() {
    let mut mind = BossMind::new();
    assert_eq!(mind.phase, 1);
    assert_eq!(mind.barrage_count(), 3);
    let base_cd = mind.cooldown();
    assert!(!mind.check_enrage(0.6));
    assert!(mind.check_enrage(0.49));
    assert_eq!(mind.phase, 2);
    assert_eq!(mind.barrage_count(), 5);
    assert!((mind.cooldown() - base_cd * 0.7).abs() < 1e-4);
    mind.active = Some(BossSpecial::SeismicSlam);
    mind.timer = 0.5;
    assert_eq!(mind.winding().map(|s| s.label()), Some("SEISMIC SLAM"));
    mind.timer = 1.2;
    assert!(mind.winding().is_none());
}

/// Melee `sense::first` selects the closest part contact along the swing segment, not whichever
/// entity happened to iterate first.
#[test]
fn strike_sense_first_picks_closest_part() {
    let mut world = World::new();
    let root = world
        .spawn(Creature {
            species: SpeciesId {
                planet: 1,
                index: 0,
            },
            herd: 1,
            slot: 0,
        })
        .id();
    let far = world
        .spawn((
            Part,
            ChildOf(root),
            GlobalTransform::from_translation(Vec3::new(0.0, 0.0, -3.0)),
            Aabb::from_min_max(Vec3::splat(-0.4), Vec3::splat(0.4)),
        ))
        .id();
    let near = world
        .spawn((
            Part,
            ChildOf(root),
            GlobalTransform::from_translation(Vec3::new(0.0, 0.0, -1.2)),
            Aabb::from_min_max(Vec3::splat(-0.4), Vec3::splat(0.4)),
        ))
        .id();
    let mut state: SystemState<(Parts, Query<(), Live>)> = SystemState::new(&mut world);
    let (parts_q, live_q) = state.get(&world).expect("query state");
    let was = [Vec3::ZERO, Vec3::new(0.0, 0.0, -0.5)];
    let now = [Vec3::new(0.0, 0.0, -4.0), Vec3::new(0.0, 0.0, -4.5)];
    let (hit_part, hit_root, _) = first(&parts_q, &live_q, (was, now), 0.1).expect("hit part");
    assert_eq!(hit_root, root);
    assert_eq!(hit_part, near);
    assert_ne!(hit_part, far);
}
