//! A stage that isolates what draws the flecks of the sky. Every shot starts from a full restore, changes one
//! thing, waits, logs what is drawn and shoots. Things are hidden by taking their `Mesh3d` off, which no system
//! of the game puts back, and never through `Visibility`, which the cloud layer and the far bodies rewrite
//! every frame.

use std::collections::BTreeMap;

use abstractions::render::{CloudCover, CloudMaterial, CloudShell, Palette, Vantage};
use abstractions::weather::Weather;
use bevy::camera::visibility::{NoFrustumCulling, RenderLayers};
use bevy::light::NotShadowCaster;
use bevy::pbr::Material;

use super::*;

/// Frames each shot gets: the restore and the change on the first, the shot five before the last.
const TURN: u32 = 45;

/// Pitch in radians every shot looks at unless it changes the pitch: the one of `L1a-all`.
const PITCH: f32 = 0.3;

/// Cover every shot forces unless it changes the cover: a light cover, the kind the flecks came from. The
/// cover of the weather moves with the time of the run, so a shot under it proves nothing about another.
const COVER: f32 = 0.3;

/// What the stage took off the world and must put back before the next shot.
#[derive(Resource, Default)]
pub struct Isolated {
    /// The meshes taken off their entities.
    meshes: Vec<(Entity, Mesh3d)>,
    /// The red copy of the cloud shell, while it stands in for it.
    twin: Option<Entity>,
}

/// The one thing a shot changes.
#[derive(Debug, Clone, Copy)]
enum Change {
    /// Nothing: the baseline under the forced reference cover.
    Nothing,
    /// The cover is given back to the weather.
    Weather,
    /// The cover of the cloud shell is forced to this value.
    Cover(f32),
    /// The cloud shell is drawn as its bare mesh in flat red of this opacity, by a standard unlit material.
    Twin(f32),
    /// Every mesh whose material kind holds this text is taken off.
    Hide(&'static str),
    /// Every mesh is taken off but those whose material kind holds one of these texts.
    Keep(&'static [&'static str]),
    /// The cloud shadows are switched off in the settings.
    NoShade,
    /// The camera looks toward the star at this pitch in radians.
    Pitch(f32),
    /// The player is held this many metres over the ground.
    High(f64),
    /// The cloud shell is despawned. Nothing brings it back, so it comes last.
    Despawn,
}

/// The shots in order: name and change.
const STEPS: [(&str, Change); 22] = [
    ("I00-weather-cover", Change::Weather),
    ("I00-baseline", Change::Nothing),
    ("I01-cover-0", Change::Cover(0.0)),
    ("I02-cover-045", Change::Cover(0.45)),
    ("I03-cover-1", Change::Cover(1.0)),
    ("I04-shell-red-opaque", Change::Twin(1.0)),
    ("I05-shell-red-blend", Change::Twin(0.5)),
    ("I06-no-cloud-mesh", Change::Hide("CloudMaterial")),
    ("I07-no-body-mesh", Change::Hide("BodyMaterial")),
    ("I08-no-halo-mesh", Change::Hide("HaloMaterial")),
    ("I09-no-ring-mesh", Change::Hide("RingMaterial")),
    ("I10-no-star-mesh", Change::Hide("StarMaterial")),
    ("I11-no-sea-mesh", Change::Hide("WaterMaterial")),
    ("I12-no-ground-mesh", Change::Hide("Daylit")),
    ("I13-dome-only", Change::Keep(&["SkyMaterial"])),
    (
        "I14-dome-and-clouds",
        Change::Keep(&["SkyMaterial", "CloudMaterial"]),
    ),
    ("I15-no-cloud-shadows", Change::NoShade),
    ("I16-zenith", Change::Pitch(1.5)),
    ("I17-level", Change::Pitch(0.0)),
    ("I18-3km", Change::High(3000.0)),
    ("I19-shell-despawned", Change::Despawn),
    ("I20-after", Change::Nothing),
];

/// Returns the kind of material `entity` is drawn with: the last word of the type of its material
/// component, `none` without one. The names need the `debug` feature of Bevy, which `dev` switches on.
fn kind(world: &World, entity: Entity) -> String {
    let Ok(parts) = world.inspect_entity(entity) else {
        return "gone".into();
    };
    let name = parts
        .map(|(_, info)| info.name().to_string())
        .find(|name| name.contains("MeshMaterial3d<"));
    let Some(name) = name else {
        return "none".into();
    };
    let last = name.rsplit("::").next().unwrap_or_default();
    last.trim_end_matches('>').to_string()
}

/// Takes the mesh off every entity whose material kind `pick` accepts, and keeps it for the restore.
fn hide(world: &mut World, pick: impl Fn(&str) -> bool) {
    let all: Vec<Entity> = world
        .query_filtered::<Entity, With<Mesh3d>>()
        .iter(world)
        .collect();
    let chosen: Vec<Entity> = all
        .into_iter()
        .filter(|entity| pick(&kind(world, *entity)))
        .collect();
    let mut taken = Vec::new();
    for entity in chosen {
        if let Some(mesh) = world.entity_mut(entity).take::<Mesh3d>() {
            taken.push((entity, mesh));
        }
    }
    eprintln!("PROBE isolate: took {} meshes off", taken.len());
    world.resource_mut::<Isolated>().meshes.extend(taken);
}

/// Draws the mesh of the cloud shell in flat red of opacity `alpha` with a standard unlit material, in
/// place of the shell, whose own mesh is taken off.
fn twin(world: &mut World, alpha: f32) {
    let found = world
        .query_filtered::<(&Mesh3d, &Transform), With<CloudShell>>()
        .iter(world)
        .next()
        .map(|(mesh, pose)| (mesh.clone(), *pose));
    let Some((mesh, pose)) = found else {
        eprintln!("PROBE isolate: no cloud shell with a mesh to copy");
        return;
    };
    let material = StandardMaterial {
        base_color: Color::srgba(1.0, 0.0, 0.0, alpha),
        unlit: true,
        fog_enabled: false,
        double_sided: true,
        cull_mode: None,
        alpha_mode: [AlphaMode::Opaque, AlphaMode::Blend][usize::from(alpha < 1.0)],
        ..default()
    };
    let handle = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(material);
    let red = (
        mesh,
        MeshMaterial3d(handle),
        pose,
        NoFrustumCulling,
        NotShadowCaster,
    );
    let twin = world.spawn(red).id();
    hide(world, |kind| kind.contains("CloudMaterial"));
    world.resource_mut::<Isolated>().twin = Some(twin);
    eprintln!(
        "PROBE isolate: red twin at {:?} scale {:?}",
        pose.translation, pose.scale
    );
}

/// Puts back every mesh taken off, removes the red twin, forces the reference cover, switches
/// the clouds and their shadows on, and looks toward the star at [`PITCH`] from the ground.
fn restore(world: &mut World) {
    let mut state = std::mem::take(&mut *world.resource_mut::<Isolated>());
    let back = state.meshes.len();
    for (entity, mesh) in state.meshes.drain(..) {
        if let Ok(mut found) = world.get_entity_mut(entity) {
            found.insert(mesh);
        }
    }
    if let Some(found) = state
        .twin
        .take()
        .and_then(|twin| world.get_entity_mut(twin).ok())
    {
        found.despawn();
    }
    world.resource_mut::<CloudCover>().0 = Some(COVER);
    let graphics = &world.resource::<Persistent<Settings>>().graphics;
    if !graphics.clouds || !graphics.lighting.cloud_shade {
        let mut settings = world.resource_mut::<Persistent<Settings>>();
        settings.graphics.clouds = true;
        settings.graphics.lighting.cloud_shade = true;
    }
    gaze(world, PITCH, true);
    eprintln!("PROBE isolate: restored {back} meshes");
}

/// Makes the one change of a shot.
fn apply(world: &mut World, change: Change) {
    match change {
        Change::Nothing | Change::High(_) => {}
        Change::Weather => world.resource_mut::<CloudCover>().0 = None,
        Change::Cover(value) => world.resource_mut::<CloudCover>().0 = Some(value),
        Change::Twin(alpha) => twin(world, alpha),
        Change::Hide(part) => hide(world, |kind| kind.contains(part)),
        Change::Keep(parts) => hide(world, |kind| !parts.iter().any(|part| kind.contains(part))),
        Change::NoShade => {
            let mut settings = world.resource_mut::<Persistent<Settings>>();
            settings.graphics.lighting.cloud_shade = false;
        }
        Change::Pitch(pitch) => gaze(world, pitch, true),
        Change::Despawn => {
            let shells: Vec<Entity> = world
                .query_filtered::<Entity, With<CloudShell>>()
                .iter(world)
                .collect();
            eprintln!("PROBE isolate: despawning {} cloud shells", shells.len());
            shells.into_iter().for_each(|shell| {
                world.despawn(shell);
            });
        }
    }
}

/// Holds the player `high` metres over the ground under it, looking toward the east a little down.
fn aloft(world: &mut World, high: f64) {
    let at = robot(world);
    let feet = ground(world, at, high);
    let up = feet.normalize();
    let east = up.cross(DVec3::Y).normalize_or(DVec3::X);
    stand(world, feet, feet + east * 100.0, -0.15);
}

/// What the report reads of a cloud shell.
type ShellRow = (
    Option<&'static Mesh3d>,
    &'static Visibility,
    &'static ViewVisibility,
    &'static Transform,
    Option<&'static RenderLayers>,
    &'static MeshMaterial3d<CloudMaterial>,
);

/// Logs where the camera is, the colours of the sky, and every cloud shell: whether it has a mesh, is
/// shown and is in view, its pose, its render layers, its alpha mode and its shader values.
fn report(world: &mut World, name: &str) {
    let (vantage, palette) = (*world.resource::<Vantage>(), *world.resource::<Palette>());
    let cover = world.resource::<CloudCover>().0;
    let weather = world.get_resource::<Weather>().map(|weather| weather.cloud);
    eprintln!(
        "PROBE isolate {name}: altitude {:.0} m, air {:.3}, cloud band {:.0} to {:.0} m, forced cover {cover:?}, weather cover {weather:?}, \
         cloud {:?}, horizon {:?}, zenith {:?}",
        vantage.altitude,
        vantage.air,
        vantage.cloud_base,
        vantage.cloud_top,
        palette.cloud,
        palette.horizon,
        palette.zenith
    );
    let shells: Vec<_> = world
        .query_filtered::<ShellRow, With<CloudShell>>()
        .iter(world)
        .map(|(mesh, shown, view, pose, layers, material)| {
            let state = (mesh.is_some(), *shown, view.get());
            (state, *pose, layers.cloned(), material.0.clone())
        })
        .collect();
    if shells.is_empty() {
        eprintln!("PROBE isolate {name}: no cloud shell entity");
    }
    for ((meshed, shown, view), pose, layers, handle) in shells {
        let materials = world.resource::<Assets<CloudMaterial>>();
        let read = materials
            .get(&handle)
            .map(|material| (material.alpha_mode(), material.clouds));
        eprintln!(
            "PROBE isolate {name}: shell mesh {meshed}, {shown:?}, in view {view}, at {:?} scale {:.0}, \
             layers {layers:?}, alpha mode and uniform {read:?}",
            pose.translation, pose.scale.x
        );
    }
}

/// Logs how many entities with a mesh each kind of material has, how many are in view and how many are
/// hidden, then the [`report`].
fn tally(world: &mut World, name: &str) {
    let rows: Vec<(Entity, bool, bool)> = world
        .query_filtered::<(Entity, &ViewVisibility, &Visibility), With<Mesh3d>>()
        .iter(world)
        .map(|(entity, view, shown)| (entity, view.get(), *shown == Visibility::Hidden))
        .collect();
    let mut kinds: BTreeMap<String, [usize; 3]> = BTreeMap::new();
    for (entity, view, hidden) in rows {
        let row = kinds.entry(kind(world, entity)).or_default();
        row[0] += 1;
        row[1] += usize::from(view);
        row[2] += usize::from(hidden);
    }
    let taken = world.resource::<Isolated>().meshes.len();
    eprintln!(
        "PROBE isolate {name}: {taken} meshes off; kind [with a mesh, in view, hidden]: {kinds:?}"
    );
    report(world, name);
}

/// Shoots the sky once per entry of [`STEPS`], each from a full restore with one change, and logs what is
/// drawn at each shot. The cloud shell is despawned for the last two shots and stays gone: run this stage
/// alone or last.
pub fn look_isolate(world: &mut World, _: &mut Notes, frame: u32) -> Act {
    let index = (frame.saturating_sub(1) / TURN) as usize;
    let at = frame.saturating_sub(1) % TURN;
    let Some((name, change)) = STEPS.get(index).copied() else {
        return Act::Next;
    };
    if at == 0 {
        world.init_resource::<Isolated>();
        restore(world);
        apply(world, change);
        eprintln!("PROBE isolate {name}: restored, then {change:?}");
    }
    if let Change::High(high) = change {
        aloft(world, high);
    }
    if at != TURN - 5 {
        return Act::Wait;
    }
    tally(world, name);
    if index + 1 == STEPS.len() {
        return Act::Shoot(name.into());
    }
    shot(world, name);
    Act::Wait
}
