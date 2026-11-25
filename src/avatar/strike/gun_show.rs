//! Firearm wielding in first person, third person, and on remote player puppets.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use super::super::Avatar;
use super::super::animate::Anim;
use super::super::own::Own;
use super::super::rig::{Seg, globals};
use super::super::show::Shown;
use super::tune::HAND_GRIP;
use crate::craft::Pack;
use crate::craft::gun::mesh::gun_mesh;
use crate::craft::gun::{GunKind, GunState};
use crate::craft::matter::Mineral;
use crate::render::GameCamera;

/// Firearm carried by a remote player puppet for third-person rendering.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteGun {
    /// Pistol or rifle chassis.
    pub kind: GunKind,
    /// Primary barrel mineral.
    pub barrel: Mineral,
}

/// Child firearm mesh spawned on an avatar body.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub struct WieldedGun {
    /// Child entity carrying the procedural gun mesh.
    pub entity: Entity,
    /// Firearm chassis currently displayed.
    pub kind: GunKind,
    /// Barrel mineral currently displayed.
    pub barrel: Mineral,
}

/// Marker on the child entity carrying a rendered firearm mesh.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct GunModel;

/// Game camera transform and optional parent query.
type CameraQuery<'w, 's> = Query<
    'w,
    's,
    (&'static Transform, Option<&'static ChildOf>),
    (With<GameCamera>, Without<GunModel>),
>;

/// Mesh, material, and cached white material handle bundle for gun spawning.
type GunAssets<'w, 's> = (
    ResMut<'w, Assets<Mesh>>,
    ResMut<'w, Assets<StandardMaterial>>,
    Local<'s, Option<Handle<StandardMaterial>>>,
);

/// Avatar bodies carrying optional remote or wielded firearm markers.
type AvatarGuns<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        Option<&'static RemoteGun>,
        Option<&'static WieldedGun>,
    ),
    With<Avatar>,
>;

/// Animated bodies with active wielded firearms.
type PlacedBodies<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Anim,
        &'static WieldedGun,
        Option<&'static ChildOf>,
        &'static Transform,
    ),
    Without<GunModel>,
>;

/// Camera, view mode, and parent transforms read while placing wielded firearms.
#[derive(SystemParam)]
pub struct PlaceView<'w, 's> {
    /// Local player and avatar lookup.
    own: Own<'w, 's>,
    /// Current body visibility setting (first-person vs third-person).
    shown: Option<Res<'w, Shown>>,
    /// Local firearm runtime state updated with `muzzle_world`.
    state: Option<ResMut<'w, GunState>>,
    /// Active game camera local transform and optional parent.
    cameras: CameraQuery<'w, 's>,
    /// Parent transforms for resolving world-space camera and body frames.
    parents: Query<'w, 's, &'static Transform, (Without<GunModel>, Without<GameCamera>)>,
}

/// Returns the barrel muzzle tip offset in local firearm space.
pub fn muzzle_offset(kind: GunKind) -> Vec3 {
    match kind {
        GunKind::FlintlockPistol => Vec3::new(0.0, 0.032, -0.26),
        GunKind::MineralRifle => Vec3::new(0.0, 0.040, -0.48),
    }
}

/// Returns the camera-space transform for a firearm held in first person.
pub fn first_person_offset(ads: f32, recoil: f32) -> Transform {
    let hip = Vec3::new(0.22, -0.18, -0.38);
    let sight = Vec3::new(0.0, -0.075, -0.32);
    let mut pos = hip.lerp(sight, ads.clamp(0.0, 1.0));
    pos.z += recoil * 0.08;
    Transform::from_translation(pos).with_rotation(Quat::from_rotation_x(recoil * 0.22))
}

/// Mesh, material, and cache tuple used when spawning held firearm models.
type GunRes<'a> = (
    &'a mut Assets<Mesh>,
    &'a mut Assets<StandardMaterial>,
    &'a mut Option<Handle<StandardMaterial>>,
);

/// Replaces `wielded` on `body` with `want`, spawning or despawning child gun meshes as needed.
fn set_gun(
    commands: &mut Commands,
    (meshes, materials, cache): GunRes,
    body: Entity,
    wielded: Option<&WieldedGun>,
    want: Option<(GunKind, Mineral)>,
) {
    if let Some(old) = wielded {
        commands.entity(old.entity).despawn();
    }
    let Some((kind, barrel)) = want else {
        commands.entity(body).remove::<WieldedGun>();
        return;
    };
    let mat = cache
        .get_or_insert_with(|| {
            materials.add(StandardMaterial {
                base_color: Color::WHITE,
                perceptual_roughness: 0.5,
                ..default()
            })
        })
        .clone();
    let look = (
        Mesh3d(meshes.add(gun_mesh(kind, barrel))),
        MeshMaterial3d(mat),
    );
    let vis = (Transform::default(), Visibility::default());
    let entity = commands
        .spawn((Name::new("Gun"), GunModel, look, vis, ChildOf(body)))
        .id();
    commands.entity(body).insert(WieldedGun {
        entity,
        kind,
        barrel,
    });
}

/// Spawns, updates, or despawns the held firearm mesh on the local robot and remote puppets.
pub fn sync(
    mut commands: Commands,
    (pack, own, mut state): (Option<Res<Pack>>, Own, Option<ResMut<GunState>>),
    (mut meshes, mut materials, mut cache): GunAssets,
    bodies: AvatarGuns,
) {
    let mine = own.body().map(|(_, body)| body);
    let local = pack
        .as_deref()
        .and_then(Pack::held_gun)
        .map(|g| (g.kind, g.barrel));
    if local.is_none()
        && let Some(state) = state.as_deref_mut()
    {
        state.muzzle_world = None;
    }
    for (body, remote, wielded) in &bodies {
        let want = if Some(body) == mine {
            local
        } else {
            remote.map(|r| (r.kind, r.barrel))
        };
        if want != wielded.map(|w| (w.kind, w.barrel)) {
            set_gun(
                &mut commands,
                (&mut meshes, &mut materials, &mut cache),
                body,
                wielded,
                want,
            );
        }
    }
}

/// Resolves world-space `Transform` from a child `local` transform and optional `parent`.
fn resolve_world(
    local: &Transform,
    parent: Option<&ChildOf>,
    parents: &Query<&Transform, (Without<GunModel>, Without<GameCamera>)>,
) -> Transform {
    let base = parent
        .and_then(|c| parents.get(c.parent()).ok())
        .copied()
        .unwrap_or(Transform::IDENTITY);
    base * *local
}

/// Computes local gun transform in body space and optional world-space muzzle tip for one body.
fn place_one(
    view: &PlaceView,
    (is_local, shown): (bool, Shown),
    (anim, kind, body_world): (&Anim, GunKind, Transform),
    (ads, recoil): (f32, f32),
) -> (Transform, Option<Vec3>) {
    if is_local
        && shown == Shown::FirstPerson
        && let Ok((cam_tf, cam_of)) = view.cameras.single()
    {
        let cam_world = resolve_world(cam_tf, cam_of, &view.parents);
        let gun_world = cam_world * first_person_offset(ads, recoil);
        let in_body =
            GlobalTransform::from(gun_world).reparented_to(&GlobalTransform::from(body_world));
        return (
            in_body,
            Some(gun_world.transform_point(muzzle_offset(kind))),
        );
    }
    let hand = globals(&anim.pose)[Seg::HandR as usize];
    let grip = hand.transform_point(Vec3::new(0.0, -HAND_GRIP, 0.0));
    let mut in_body = Transform::from_translation(grip).with_rotation(hand.rotation);
    if is_local {
        let kick = Transform::from_xyz(0.0, 0.0, recoil * 0.06)
            .with_rotation(Quat::from_rotation_x(recoil * 0.18));
        in_body = in_body * kick;
    }
    let muzzle = is_local.then(|| (body_world * in_body).transform_point(muzzle_offset(kind)));
    (in_body, muzzle)
}

/// Places every wielded firearm in first-person camera space or attached to `Seg::HandR` in 3P.
pub fn place(
    mut view: PlaceView,
    bodies: PlacedBodies,
    mut guns: Query<(&mut Transform, &mut Visibility), With<GunModel>>,
) {
    let mine = view.own.body().map(|(_, body)| body);
    let shown = view.shown.as_deref().copied().unwrap_or(Shown::FirstPerson);
    let state_vals = view.state.as_deref().map(|s| (s.ads, s.recoil));
    let (ads, recoil) = state_vals.unwrap_or((0.0, 0.0));
    let mut muzzle_out = None;
    for (body, anim, wielded, of, body_tf) in &bodies {
        let Ok((mut gun_tf, mut vis)) = guns.get_mut(wielded.entity) else {
            continue;
        };
        let is_local = mine == Some(body);
        let hidden = is_local && shown == Shown::Hidden;
        vis.set_if_neq([Visibility::Inherited, Visibility::Hidden][usize::from(hidden)]);
        let body_world = resolve_world(body_tf, of, &view.parents);
        let spec = (anim, wielded.kind, body_world);
        let (in_body, muzzle) = place_one(&view, (is_local, shown), spec, (ads, recoil));
        *gun_tf = in_body;
        muzzle_out = muzzle.or(muzzle_out);
    }
    if let (Some(muzzle), Some(state)) = (muzzle_out, view.state.as_deref_mut()) {
        state.muzzle_world = Some(muzzle);
    }
}

/// Returns the systems that sync and pose wielded firearms in first and third person.
pub(super) fn systems() -> impl IntoScheduleConfigs<bevy::ecs::system::ScheduleSystem, ()> {
    (
        sync.after(crate::craft::wield::choose),
        place
            .after(super::super::AvatarSet::Animate)
            .after(sync)
            .run_if(any_with_component::<WieldedGun>),
    )
        .into_configs()
}
