//! Icons, drawn as glyphs of the Material Symbols Sharp icon font embedded in the binary.
//!
//! The look follows the owner's direction of 2026-11-20: square corners, no borders, no round
//! caps. The font is a subset of `MaterialSymbolsSharp[FILL,GRAD,opsz,wght].ttf`, instanced at
//! weight 400, FILL 0, GRAD 0, optical size 24, cut to the glyphs below. The exact commands
//! are in `packaging/icons.md`. The licence (Apache-2.0) is in `LICENSES/`. Icons are real
//! pictures with a name on screen somewhere: an icon never stands alone without a text, and
//! no black box sits behind it. Colour comes from the tokens of `ui::theme`: `ACCENT` for the
//! course, `PIN` for pins, `WARN` and `DANGER` for trouble, `TEXT` for home and the ship.

use bevy::asset::uuid_handle;
use bevy::prelude::*;

use super::style::Tone;
use super::theme;

/// Name of the font instance the embedded file comes from.
pub const ICONS_VERSION: &str = "Material Symbols Sharp, instance wght 400 FILL 0 opsz 24";

/// Bytes of the icon font file.
pub(super) const ICONS_TTF: &[u8] = include_bytes!("../../assets/fonts/material-symbols-sharp-subset.ttf");

/// Handle of the icon font asset, filled at startup.
pub const ICON_FONT: Handle<Font> = uuid_handle!("6c1f0a52-3d0e-4b6e-9a57-4f1d2c7b8e90");

/// The icons the game uses. Each doc line gives the Material Symbols Sharp name of the glyph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    /// `play_arrow`
    Play,
    /// `pause`
    Pause,
    /// `planet`
    Planet,
    /// `dark_mode`
    Moon,
    /// `sunny`
    Sun,
    /// `cyclone`
    Galaxy,
    /// `add`
    Plus,
    /// `tune`
    Settings,
    /// `logout`
    LogOut,
    /// `power_settings_new`
    Power,
    /// `edit`
    Pencil,
    /// `content_copy`
    Copy,
    /// `delete`
    Trash,
    /// `swap_vert`
    Sort,
    /// `arrow_back`
    Back,
    /// `check`
    Check,
    /// `close`
    Close,
    /// `monitor`
    Monitor,
    /// `keyboard`
    Keyboard,
    /// `volume_up`
    Volume,
    /// `info`
    Info,
    /// `star`
    Star,
    /// `send`
    Send,
    /// `location_on`
    MapPin,
    /// `shuffle`
    Shuffle,
    /// `ink_eraser`
    Eraser,
    /// `chevron_left`
    ChevronLeft,
    /// `chevron_right`
    ChevronRight,
    /// `home`
    Home,
    /// `map`
    Map,
    /// `group`
    Users,
    /// `person`
    User,
    /// `dns`
    Server,
    /// `wifi`
    Wifi,
    /// `refresh`
    Reload,
    /// `link`
    Link,
    /// `pets`
    Animal,
    /// `rocket_launch`
    Go,
    /// `stop_circle`
    Stop,
    /// `arrow_drop_up`
    Arrow,
    /// `straighten`
    Radius,
    /// `scale`
    Gravity,
    /// `thermostat`
    Temperature,
    /// `compress`
    Pressure,
    /// `air`
    Air,
    /// `wb_twilight`
    Day,
    /// `timer`
    Year,
    /// `water_drop`
    Water,
    /// `eco`
    Life,
    /// `rocket`
    ShipParked,
    /// `navigation`
    ShipFlying,
    /// `fast_forward`
    ShipWarp,
    /// `build`
    ShipDamaged,
}

/// Every icon with its codepoint in the icon font, in declaration order.
pub(super) const CODEPOINTS: [(Icon, char); 53] = [
    (Icon::Play, '\u{e037}'),
    (Icon::Pause, '\u{e034}'),
    (Icon::Planet, '\u{f387}'),
    (Icon::Moon, '\u{e51c}'),
    (Icon::Sun, '\u{e81a}'),
    (Icon::Galaxy, '\u{ebd5}'),
    (Icon::Plus, '\u{e145}'),
    (Icon::Settings, '\u{e429}'),
    (Icon::LogOut, '\u{e9ba}'),
    (Icon::Power, '\u{f8c7}'),
    (Icon::Pencil, '\u{f097}'),
    (Icon::Copy, '\u{e14d}'),
    (Icon::Trash, '\u{e92e}'),
    (Icon::Sort, '\u{e8d5}'),
    (Icon::Back, '\u{e5c4}'),
    (Icon::Check, '\u{e668}'),
    (Icon::Close, '\u{e5cd}'),
    (Icon::Monitor, '\u{ef5b}'),
    (Icon::Keyboard, '\u{e312}'),
    (Icon::Volume, '\u{e050}'),
    (Icon::Info, '\u{e88e}'),
    (Icon::Star, '\u{f09a}'),
    (Icon::Send, '\u{e163}'),
    (Icon::MapPin, '\u{f1db}'),
    (Icon::Shuffle, '\u{e043}'),
    (Icon::Eraser, '\u{e6d0}'),
    (Icon::ChevronLeft, '\u{e5cb}'),
    (Icon::ChevronRight, '\u{e5cc}'),
    (Icon::Home, '\u{e9b2}'),
    (Icon::Map, '\u{e55b}'),
    (Icon::Users, '\u{ea21}'),
    (Icon::User, '\u{f0d3}'),
    (Icon::Server, '\u{e875}'),
    (Icon::Wifi, '\u{e63e}'),
    (Icon::Reload, '\u{e5d5}'),
    (Icon::Link, '\u{e250}'),
    (Icon::Animal, '\u{e91d}'),
    (Icon::Go, '\u{eb9b}'),
    (Icon::Stop, '\u{ef71}'),
    (Icon::Arrow, '\u{e5c7}'),
    (Icon::Radius, '\u{e41c}'),
    (Icon::Gravity, '\u{eb5f}'),
    (Icon::Temperature, '\u{f076}'),
    (Icon::Pressure, '\u{e94d}'),
    (Icon::Air, '\u{efd8}'),
    (Icon::Day, '\u{e1c6}'),
    (Icon::Year, '\u{e425}'),
    (Icon::Water, '\u{e798}'),
    (Icon::Life, '\u{ea35}'),
    (Icon::ShipParked, '\u{eba5}'),
    (Icon::ShipFlying, '\u{e55d}'),
    (Icon::ShipWarp, '\u{e01f}'),
    (Icon::ShipDamaged, '\u{f8cd}'),
];

impl Icon {
    /// Every icon, in declaration order.
    pub const ALL: [Icon; 53] = [
        Icon::Play,
        Icon::Pause,
        Icon::Planet,
        Icon::Moon,
        Icon::Sun,
        Icon::Galaxy,
        Icon::Plus,
        Icon::Settings,
        Icon::LogOut,
        Icon::Power,
        Icon::Pencil,
        Icon::Copy,
        Icon::Trash,
        Icon::Sort,
        Icon::Back,
        Icon::Check,
        Icon::Close,
        Icon::Monitor,
        Icon::Keyboard,
        Icon::Volume,
        Icon::Info,
        Icon::Star,
        Icon::Send,
        Icon::MapPin,
        Icon::Shuffle,
        Icon::Eraser,
        Icon::ChevronLeft,
        Icon::ChevronRight,
        Icon::Home,
        Icon::Map,
        Icon::Users,
        Icon::User,
        Icon::Server,
        Icon::Wifi,
        Icon::Reload,
        Icon::Link,
        Icon::Animal,
        Icon::Go,
        Icon::Stop,
        Icon::Arrow,
        Icon::Radius,
        Icon::Gravity,
        Icon::Temperature,
        Icon::Pressure,
        Icon::Air,
        Icon::Day,
        Icon::Year,
        Icon::Water,
        Icon::Life,
        Icon::ShipParked,
        Icon::ShipFlying,
        Icon::ShipWarp,
        Icon::ShipDamaged,
    ];

    /// Returns the codepoint of the icon in the icon font.
    pub const fn codepoint(self) -> char {
        CODEPOINTS[self as usize].1
    }

    /// Returns the icon as the text of a node that uses [`ICON_FONT`].
    pub fn glyph(self) -> String {
        self.codepoint().to_string()
    }
}

/// One icon on its 24 pixel grid, whose colour follows `tone` and the button around it.
pub fn icon(icon: Icon, tone: Tone) -> impl Bundle {
    icon_sized(icon, tone, theme::ICON)
}

/// One icon drawn `size` pixels high, for marks that must stay out of the way.
pub fn icon_sized(icon: Icon, tone: Tone, size: f32) -> impl Bundle {
    (
        Text::new(icon.glyph()),
        TextFont::from_font_size(size).with_font(ICON_FONT),
        TextLayout::no_wrap(),
        tone,
        Node {
            min_width: px(size),
            flex_shrink: 0.0,
            ..default()
        },
    )
}

/// The room of one icon, holding `symbol` when there is one. A label without an icon keeps
/// the room, so that it lines up with labels that have one.
pub fn icon_slot(symbol: Option<Icon>, tone: Tone) -> impl Bundle {
    (
        Node {
            width: px(theme::ICON),
            height: px(theme::ICON),
            flex_shrink: 0.0,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Children::spawn(SpawnWith(move |parent: &mut ChildSpawner| {
            if let Some(symbol) = symbol {
                parent.spawn(icon(symbol, tone));
            }
        })),
    )
}

/// Puts the embedded font behind [`ICON_FONT`]. Does nothing in an app without text.
pub(super) fn load(fonts: Option<ResMut<Assets<Font>>>) {
    if let Some(mut fonts) = fonts
        && let Err(error) = fonts.insert(ICON_FONT.id(), Font::from_bytes(ICONS_TTF.to_vec()))
    {
        warn!("could not register the icon font: {error}");
    }
}
