//! State tracking and per-frame updates for the classic framed top-centre boss health bar.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::avatar::strike::Mark;
use crate::fauna::{Beast, BossMind, Live};
use crate::life::Life;
use crate::player::Player;
use crate::space::Anchor;
use crate::world::Terrain;

/// Maximum distance in metres from the player at which an untargeted boss shows its bar.
const RANGE: f64 = 95.0;

/// Rate per second at which the delayed white/gold damage bar catches up to current HP.
const TRAIL_RATE: f32 = 0.45;

/// Marks the outer framed boss HUD container.
#[derive(Component)]
pub(super) struct BossFrame;

/// Marks the header text line of the boss HUD.
#[derive(Component)]
pub(super) struct BossTitle;

/// Marks the `PHASE II` badge shown below 50% HP.
#[derive(Component)]
pub(super) struct BossPhase;

/// Marks the white/gold trailing delayed-damage fill node.
#[derive(Component)]
pub(super) struct BossTrail;

/// Marks the crimson current-HP fill node.
#[derive(Component)]
pub(super) struct BossFill;

/// Marks the special-attack telegraph callout beneath the bar.
#[derive(Component)]
pub(super) struct BossWarn;

/// Smoothed trailing health share kept across frames for the active boss.
#[derive(Default)]
pub(super) struct TrailState {
    /// Entity whose health bar is currently tracked.
    entity: Option<Entity>,
    /// Delayed health share in `0..=1`.
    share: f32,
}

/// Snapshot of the boss currently displayed by the HUD.
struct Shown {
    /// Entity of the boss.
    entity: Entity,
    /// Header string: name, epithet and length in metres.
    header: String,
    /// Current HP share in `0..=1`.
    hp: f32,
    /// Current combat phase (1 or 2).
    phase: u8,
    /// Special-attack telegraph label during windup, if any.
    warn: Option<&'static str>,
}

/// Read-only world queries used to pick the active boss for the HUD.
#[derive(SystemParam)]
pub(super) struct BossRead<'w, 's> {
    time: Res<'w, Time>,
    mark: Res<'w, Mark>,
    terrain: Option<Res<'w, Terrain>>,
    anchor: Option<Res<'w, Anchor>>,
    player: Query<'w, 's, &'static Transform, With<Player>>,
    bosses: Query<'w, 's, (Entity, &'static Beast, &'static BossMind), Live>,
}

/// Disjoint query filter for boss frame nodes.
type FrameOnly = (
    With<BossFrame>,
    Without<BossTrail>,
    Without<BossFill>,
    Without<BossPhase>,
    Without<BossWarn>,
);

/// Disjoint query filter for Phase II badge nodes.
type PhaseOnly = (
    With<BossPhase>,
    Without<BossFrame>,
    Without<BossTrail>,
    Without<BossFill>,
    Without<BossWarn>,
);

/// Disjoint query filter for trailing damage bar nodes.
type TrailOnly = (
    With<BossTrail>,
    Without<BossFrame>,
    Without<BossFill>,
    Without<BossPhase>,
    Without<BossWarn>,
);

/// Disjoint query filter for crimson health fill nodes.
type FillOnly = (
    With<BossFill>,
    Without<BossFrame>,
    Without<BossTrail>,
    Without<BossPhase>,
    Without<BossWarn>,
);

/// Disjoint query filter for special-attack warning callout nodes.
type WarnOnly = (
    With<BossWarn>,
    Without<BossFrame>,
    Without<BossTrail>,
    Without<BossFill>,
    Without<BossPhase>,
    Without<BossTitle>,
);

/// Mutable UI node queries written by [`update`].
#[derive(SystemParam)]
pub(super) struct BossUi<'w, 's> {
    frame: Query<'w, 's, &'static mut Node, FrameOnly>,
    title: Query<'w, 's, &'static mut Text, (With<BossTitle>, Without<BossWarn>)>,
    phase: Query<'w, 's, &'static mut Node, PhaseOnly>,
    trail: Query<'w, 's, &'static mut Node, TrailOnly>,
    fill: Query<'w, 's, &'static mut Node, FillOnly>,
    warn: Query<'w, 's, (&'static mut Node, &'static mut Text), WarnOnly>,
}

/// Picks the boss to display: the struck victim if it is a living boss, else the nearest living
/// boss within [`RANGE`] metres of the player.
fn pick(read: &BossRead, life: &Life) -> Option<Shown> {
    let anchor = read.anchor.as_deref()?;
    let player_pos = anchor.to_frame(read.player.single().ok()?.translation);
    let mut best: Option<(f64, Entity, &Beast, &BossMind)> = None;
    for (entity, beast, mind) in &read.bosses {
        let dist = beast.spot.planet(beast.motion.root.at).distance(player_pos);
        let targeted = read.mark.victim == Some(entity);
        if !targeted && dist > RANGE {
            continue;
        }
        let rank = if targeted { -1.0 } else { dist };
        if best.is_none_or(|b| rank < b.0) {
            best = Some((rank, entity, beast, mind));
        }
    }
    let (_, entity, beast, mind) = best?;
    let sp = life.species.get(usize::from(beast.index))?;
    let header = format!(
        "{}  ·  {}  ·  {} M",
        sp.name.to_uppercase(),
        sp.epithet.to_uppercase(),
        sp.body.length.round() as u32
    );
    Some(Shown {
        entity,
        header,
        hp: beast.wits.health.share(),
        phase: mind.phase,
        warn: mind.winding().map(|s| s.label()),
    })
}

/// Updates the top-centre boss health bar every frame.
pub(super) fn update(read: BossRead, mut ui: BossUi, mut trail: Local<TrailState>) {
    let life = read.terrain.as_deref().and_then(|t| t.globe().life());
    let shown = life.and_then(|l| pick(&read, l));
    let Ok(mut frame) = ui.frame.single_mut() else {
        return;
    };
    let Some(shown) = shown else {
        frame.display = Display::None;
        trail.entity = None;
        return;
    };
    frame.display = Display::Flex;
    if trail.entity != Some(shown.entity) || shown.hp > trail.share {
        trail.entity = Some(shown.entity);
        trail.share = shown.hp;
    } else {
        let dt = read.time.delta_secs().min(0.1);
        trail.share = (trail.share - TRAIL_RATE * dt).max(shown.hp);
    }
    write_ui(&mut ui, &shown, trail.share);
}

/// Writes header, Phase II badge, trailing/current fills and special callout to UI nodes.
fn write_ui(ui: &mut BossUi, shown: &Shown, trail_share: f32) {
    if let Ok(mut title) = ui.title.single_mut()
        && title.0 != shown.header
    {
        title.0.clone_from(&shown.header);
    }
    if let Ok(mut phase) = ui.phase.single_mut() {
        phase.display = [Display::None, Display::Flex][usize::from(shown.phase >= 2)];
    }
    if let Ok(mut fill) = ui.fill.single_mut() {
        fill.width = percent(100.0 * shown.hp);
    }
    if let Ok(mut bar) = ui.trail.single_mut() {
        bar.width = percent(100.0 * trail_share);
    }
    if let Ok((mut node, mut text)) = ui.warn.single_mut() {
        node.display = [Display::None, Display::Flex][usize::from(shown.warn.is_some())];
        if let Some(label) = shown.warn
            && text.0 != label
        {
            text.0.clear();
            text.0.push_str(label);
        }
    }
}
