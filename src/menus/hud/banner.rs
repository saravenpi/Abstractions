//! The banner: the name of a planet and of its system, shown briefly on arrival, and the line
//! that says a player joined or left the session, or what the network has to say.

use bevy::prelude::*;

use crate::net::{PlayerJoined, PlayerLeft, SessionNotice};
use crate::space::Location;
use crate::ui::{caps, text, theme};
use crate::universe::{BodyId, Universe};

/// Seconds the banner stays on screen.
const SECONDS: f32 = 4.5;

/// Seconds the banner takes to fade in, and to fade out.
const FADE: f32 = 0.7;

/// Which planet the banner last announced and how long it still shows.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub(super) struct Banner {
    /// The planet announced last.
    shown: Option<BodyId>,
    /// Seconds left on screen.
    left: f32,
}

/// Marks the banner block.
#[derive(Component)]
pub(super) struct BannerBlock;

/// Marks the dark strip behind the names, which keeps them readable on any sky.
#[derive(Component)]
pub(super) struct BannerStrip;

/// Marks the planet name of the banner.
#[derive(Component)]
pub(super) struct PlanetName;

/// Marks the system name of the banner.
#[derive(Component)]
pub(super) struct SystemName;

/// Returns the opacity of the banner with `left` seconds to go: it fades in, holds, fades out.
pub fn opacity(left: f32) -> f32 {
    let since = SECONDS - left;
    (since / FADE).min(left / FADE).clamp(0.0, 1.0)
}

/// The hidden banner, high in the middle of the screen.
pub(super) fn block() -> impl Bundle {
    (
        BannerBlock,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            width: percent(100),
            top: percent(14),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: px(theme::GAP_XS),
            ..default()
        },
        children![(
            BannerStrip,
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(theme::GAP_XS),
                padding: UiRect::axes(px(theme::GAP_L), px(theme::GAP_S)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            children![
                (PlanetName, text("", theme::FONT_HEADING, Color::NONE)),
                (SystemName, caps("")),
            ],
        )],
    )
}

/// The two names of the banner.
type Names<'w, 's> = (
    Query<'w, 's, &'static mut Text, (With<PlanetName>, Without<SystemName>)>,
    Query<'w, 's, &'static mut Text, (With<SystemName>, Without<PlanetName>)>,
);

/// Starts the banner when the player arrives at a planet it did not announce last.
pub(super) fn watch(
    location: Option<Res<Location>>,
    universe: Option<Res<Universe>>,
    mut banner: ResMut<Banner>,
    (mut planets, mut systems): Names,
) {
    let (Some(location), Some(universe)) = (location, universe) else {
        return;
    };
    if banner.shown == Some(location.body) {
        return;
    }
    *banner = Banner {
        shown: Some(location.body),
        left: SECONDS,
    };
    let planet = universe.planet(location.body).name;
    let star = universe.system(location.body.star).star.name;
    for mut text in &mut planets {
        text.set_if_neq(Text(planet.clone()));
    }
    for mut text in &mut systems {
        text.set_if_neq(Text(format!("{star} system").to_uppercase()));
    }
}

/// Returns the banner line of a player who joined, or left when `joined` is false.
pub fn presence(name: &str, joined: bool) -> String {
    match joined {
        true => format!("{name} joined"),
        false => format!("{name} left"),
    }
}

/// The players who joined and left since the last frame, and the lines of the network.
type Presence<'w, 's> = (
    MessageReader<'w, 's, PlayerJoined>,
    MessageReader<'w, 's, PlayerLeft>,
    MessageReader<'w, 's, SessionNotice>,
);

/// Shows the last player who joined or left, or the last line of the network, in the banner,
/// in place of the planet name.
pub(super) fn announce(
    (mut joins, mut leaves, mut notices): Presence,
    mut banner: ResMut<Banner>,
    (mut planets, mut systems): Names,
) {
    let joined = joins.read().map(|player| presence(&player.name, true));
    let lines: Vec<String> = joined
        .chain(leaves.read().map(|player| presence(&player.name, false)))
        .chain(notices.read().map(|notice| notice.line.clone()))
        .collect();
    let Some(line) = lines.last() else {
        return;
    };
    banner.left = SECONDS;
    for mut text in &mut planets {
        text.set_if_neq(Text(line.clone()));
    }
    for mut text in &mut systems {
        text.set_if_neq(Text(String::new()));
    }
}

/// The colours of the two names.
type Tints<'w, 's> = (
    Query<'w, 's, &'static mut TextColor, (With<PlanetName>, Without<SystemName>)>,
    Query<'w, 's, &'static mut TextColor, (With<SystemName>, Without<PlanetName>)>,
);

/// Fades the banner in and out while it shows, then hides it.
pub(super) fn fade(
    time: Res<Time>,
    mut banner: ResMut<Banner>,
    mut blocks: Query<&mut Node, With<BannerBlock>>,
    mut strips: Query<&mut BackgroundColor, With<BannerStrip>>,
    (mut planets, mut systems): Tints,
) {
    banner.left = (banner.left - time.delta_secs()).max(0.0);
    let alpha = opacity(banner.left);
    for mut node in &mut blocks {
        let display = if banner.left > 0.0 {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
    }
    for mut strip in &mut strips {
        strip.set_if_neq(BackgroundColor(theme::PANEL.with_alpha(0.5 * alpha)));
    }
    for mut tint in &mut planets {
        tint.set_if_neq(TextColor(theme::TEXT.with_alpha(alpha)));
    }
    for mut tint in &mut systems {
        tint.set_if_neq(TextColor(theme::TEXT_2.with_alpha(0.65 * alpha)));
    }
}

/// Run condition: the banner is on screen.
pub(super) fn showing(banner: Res<Banner>) -> bool {
    banner.left > 0.0
}
