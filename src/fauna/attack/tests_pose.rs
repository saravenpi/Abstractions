//! Tests of the body through a whole attack, posed as the game poses it: every chain keeps its
//! lengths and stays on its socket, every carried part stays on its parent, the head never sinks
//! into the torso, and the part that strikes is drawn back and thrown out.

use bevy::math::{Vec2, Vec3};

use super::brace::brace;
use super::kinds::Kind;
use super::limbs::Broken;
use super::timing::Attacking;
use crate::fauna::motion::frame::Role;
use crate::fauna::motion::rest::carry;
use crate::fauna::motion::{Detail, Drive, Motion, Placed};
use crate::life::body::{Body, Locomotion, PartKind};
use crate::life::{self, Species};
use crate::universe::Universe;

/// Seconds per frame.
const DT: f32 = 1.0 / 60.0;

/// Gravity of the tests, in metres per second squared.
const G: f32 = 9.81;

/// Returns the walking species with a torso and a head of the living homes of a few seeds.
fn walkers() -> Vec<Species> {
    let mut found = Vec::new();
    for seed in [3, 7, 57] {
        let universe = Universe::new(seed);
        let body = universe.home();
        let star = universe.system(body.star).star;
        let Some(life) = life::of(&universe.planet(body), &star) else {
            continue;
        };
        let fits = |s: &&Species| {
            let parts = &s.body.parts;
            let torso = parts.first().is_some_and(|p| p.kind == PartKind::Torso);
            let head = parts.iter().any(|p| p.kind == PartKind::Head);
            s.body.locomotion == Locomotion::Walk && torso && head
        };
        found.extend(life.species.iter().filter(fits).cloned());
    }
    found
}

/// Returns the drive of a body standing still and looking at a point ahead of it.
fn still(body: &Body) -> Drive {
    Drive {
        velocity: Vec3::ZERO,
        look: Some(Vec3::new(0.0, body.height, -2.0 - 3.0 * body.length)),
        gravity: G,
        excite: 1.0,
        detail: Detail::Full,
    }
}

/// Checks the chains of `body` in one pose by `m`: each keeps its lengths, and one that hangs from
/// the torso or a carried part starts on its socket. `name` names the case in a failure.
fn chained(body: &Body, m: &Motion, pose: &[Placed], name: &str) {
    let chains = [&body.neck, &body.tail, &body.spine];
    for chain in chains.into_iter().flatten() {
        for (i, pair) in chain.parts.windows(2).enumerate() {
            let gap = pose[usize::from(pair[0])]
                .0
                .distance(pose[usize::from(pair[1])].0);
            let want = chain.lengths[i];
            assert!(
                (gap - want).abs() < 2e-3,
                "{name}: a link {gap} m, not {want}"
            );
        }
        let first = usize::from(chain.parts[0]);
        let Some(parent) = body.parts[first].parent.map(usize::from) else {
            continue;
        };
        if matches!(m.frame.roles[parent], Role::Torso | Role::Free) {
            let (at, turn, _) = pose[parent];
            let part = &body.parts[first];
            let off = carry((at, turn), (part.socket, part.rest))
                .0
                .distance(pose[first].0);
            assert!(off < 2e-3, "{name}: a chain {off} m off its socket");
        }
    }
}

/// Checks one pose of `body` by `m` against its rest pose `rest`: its chains hold (see [`chained`]),
/// carried parts keep their distance to their parents, and the head does not sink deeper into the
/// torso than at rest. `name` names the case in a failure.
fn whole(body: &Body, m: &Motion, (pose, rest): (&[Placed], &[Placed]), name: &str) {
    chained(body, m, pose, name);
    for (i, part) in body.parts.iter().enumerate() {
        let carried = !matches!(m.frame.roles[i], Role::Link(..) | Role::Bone(..));
        if let Some(p) = part.parent.map(usize::from).filter(|_| carried) {
            let moved = pose[i].0.distance(pose[p].0) - rest[i].0.distance(rest[p].0);
            assert!(
                moved.abs() < 2e-3,
                "{name}: part {i} moved {moved} m off its parent"
            );
        }
    }
    let (inside, before) = (depth(body, pose), depth(body, rest));
    let floor = before.min(1.0) - 0.05;
    assert!(
        inside >= floor,
        "{name}: the head sank into the torso, {inside} from {before}"
    );
}

/// Returns where the head lies against the torso in `pose`: its distance from the middle of the torso
/// in half sizes of the torso, under 1 inside it.
fn depth(body: &Body, pose: &[Placed]) -> f32 {
    let head = body
        .parts
        .iter()
        .position(|p| p.kind == PartKind::Head)
        .unwrap_or(0);
    let (at, turn, _) = pose[0];
    let half = (body.parts[0].size * 0.5).max(Vec3::splat(1e-3));
    (turn.inverse() * (pose[head].0 - at) / half).length()
}

/// Returns how far ahead and to the right of the root, and how high, the part that makes an attack of
/// `kind` lies in `pose`: the head, the tip of the first front leg or the tip of the tail.
fn striker(body: &Body, m: &Motion, kind: Kind, pose: &[Placed]) -> Vec3 {
    let tip = |index: u8, length: f32| {
        let (at, turn, _) = pose[usize::from(index)];
        at + turn * (m.frame.axes[usize::from(index)] * length)
    };
    let at = match kind {
        Kind::Swipe | Kind::Trample => body.legs.iter().find(|l| l.rank == 0).map(|l| {
            let last = l.bones.len() - 1;
            tip(l.bones[last], l.lengths[last])
        }),
        Kind::Sweep => body
            .tail
            .as_ref()
            .and_then(|t| Some(tip(*t.parts.last()?, *t.lengths.last()?))),
        _ => body
            .parts
            .iter()
            .position(|p| p.kind == PartKind::Head)
            .map(|h| pose[h].0),
    };
    let rel = at.unwrap_or(m.root.at) - m.root.at;
    Vec3::new(rel.x.abs(), rel.y, -rel.z)
}

/// Runs an attack of `kind` by `species` standing still on flat ground, checking every pose, and
/// returns where its striking part lay at rest, nearest at the end of the wind-up and farthest out
/// or highest after it.
fn cycle(species: &Species, kind: Kind) -> (Vec3, Vec3, Vec3) {
    let body = &species.body;
    let ground = |_: Vec2| 0.0;
    let mut m = Motion::new(body, Vec3::ZERO, Vec3::NEG_Z, 5);
    for _ in 0..90 {
        m.step(body, &still(body), &ground, DT);
    }
    let rest = m.pose(body);
    let start = striker(body, &m, kind, &rest);
    let mut attack = Attacking::begin(kind, body.length, G);
    let (mut drawn, mut out) = (start, start);
    let name = format!("{} {kind:?}", species.name);
    while !attack.over() {
        m.brace = brace(&attack, body.length);
        m.step(body, &still(body), &ground, DT);
        let pose = m.pose(body);
        whole(body, &m, (&pose[..], &rest[..]), &name);
        let at = striker(body, &m, kind, &pose);
        if attack.t <= attack.wind {
            drawn = at;
        }
        out = out.max(at);
        attack.t += DT;
    }
    (start, drawn, out)
}

/// Through every attack each walker of a few homes can make, no part leaves its socket or its parent,
/// no chain stretches or shrinks and the head never sinks into the torso. A bite or a lunge draws the
/// head back, then throws it farther ahead than at rest; a charge drives the head ahead; a swipe
/// strikes a forefoot ahead; a trample lifts the forefeet; a sweep swings the tail aside.
#[test]
fn an_attack_bends_the_body_and_throws_the_striker() {
    let walkers = walkers();
    let mut cases = 0;
    for species in &walkers {
        let size = species.body.length.max(0.1);
        for kind in Kind::ORDER
            .into_iter()
            .filter(|k| k.can(species, Broken::default()))
        {
            let (start, drawn, out) = cycle(species, kind);
            let name = format!("{} {kind:?}", species.name);
            let gain = match kind {
                Kind::Bite | Kind::Lunge => {
                    assert!(drawn.z < start.z, "{name}: the head is not drawn back");
                    out.z - start.z
                }
                Kind::Charge | Kind::Swipe => out.z - start.z,
                Kind::Trample => out.y - start.y,
                Kind::Sweep => out.x - start.x,
            };
            assert!(
                gain > 0.02 * size,
                "{name}: the striker moves {gain} m, the body is {size} m"
            );
            cases += 1;
        }
    }
    println!("{cases} attacks of {} walkers", walkers.len());
    assert!(cases >= 3, "only {cases} attacks posed");
}
