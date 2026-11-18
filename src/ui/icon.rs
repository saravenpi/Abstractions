//! Icons, drawn as glyphs of the pixelarticons icon font embedded in the binary.
//!
//! The font is `fonts/pixelart-icons-font.ttf` of the npm package `pixelarticons`, version
//! [`ICONS_VERSION`], unmodified. Its licence is in `LICENSES/pixelarticons-MIT.txt`. Codepoints
//! come from `fonts/pixelart-icons-font.css` of the same package: when the font is updated,
//! check every one again. Only icons of that pack are used. An action with no fitting icon in
//! the pack has a text label and no icon.

use bevy::asset::uuid_handle;
use bevy::prelude::*;

use super::style::Tone;
use super::theme;

/// Version of `pixelarticons` the embedded font comes from.
pub const ICONS_VERSION: &str = "2.4.2";

/// Bytes of the icon font file.
pub(super) const ICONS_TTF: &[u8] = include_bytes!("../../assets/fonts/pixelart-icons-font.ttf");

/// Handle of the icon font asset, filled at startup.
pub const ICON_FONT: Handle<Font> = uuid_handle!("6c1f0a52-3d0e-4b6e-9a57-4f1d2c7b8e90");

/// The icons the game uses. Each doc line gives the pixelarticons name of the icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    /// `play`
    Play,
    /// `pause`
    Pause,
    /// `earth`
    Earth,
    /// `moon`
    Moon,
    /// `plus`
    Plus,
    /// `sliders`
    Settings,
    /// `logout`
    LogOut,
    /// `power`
    Power,
    /// `pencil`
    Pencil,
    /// `copy`
    Copy,
    /// `trash`
    Trash,
    /// `sort-vertical`
    Sort,
    /// `arrow-left`
    Back,
    /// `check`
    Check,
    /// `close`
    Close,
    /// `monitor`
    Monitor,
    /// `keyboard`
    Keyboard,
    /// `volume-2`
    Volume,
    /// `circle-info`
    Info,
    /// `star`
    Star,
    /// `send`
    Send,
    /// `map-pin`
    MapPin,
    /// `shuffle`
    Shuffle,
    /// `eraser`
    Eraser,
    /// `chevron-left`
    ChevronLeft,
    /// `chevron-right`
    ChevronRight,
    /// `home`
    Home,
    /// `map`
    Map,
    /// `users`
    Users,
    /// `user`
    User,
    /// `server`
    Server,
    /// `wifi`
    Wifi,
    /// `reload`
    Reload,
    /// `link`
    Link,
}

/// Every icon with its codepoint in the pixelarticons font, in declaration order.
pub(super) const CODEPOINTS: [(Icon, char); 34] = [
    (Icon::Play, '\u{ecd3}'),
    (Icon::Pause, '\u{ecc0}'),
    (Icon::Earth, '\u{eb85}'),
    (Icon::Moon, '\u{eca1}'),
    (Icon::Plus, '\u{ecd8}'),
    (Icon::Settings, '\u{ed3d}'),
    (Icon::LogOut, '\u{ec70}'),
    (Icon::Power, '\u{ece0}'),
    (Icon::Pencil, '\u{ecc5}'),
    (Icon::Copy, '\u{eb49}'),
    (Icon::Trash, '\u{edbe}'),
    (Icon::Sort, '\u{ed4b}'),
    (Icon::Back, '\u{ea70}'),
    (Icon::Check, '\u{eb0a}'),
    (Icon::Close, '\u{eb30}'),
    (Icon::Monitor, '\u{eca0}'),
    (Icon::Keyboard, '\u{ec15}'),
    (Icon::Volume, '\u{edea}'),
    (Icon::Info, '\u{eb20}'),
    (Icon::Star, '\u{ed6d}'),
    (Icon::Send, '\u{ed1e}'),
    (Icon::MapPin, '\u{ec7b}'),
    (Icon::Shuffle, '\u{ed31}'),
    (Icon::Eraser, '\u{eb87}'),
    (Icon::ChevronLeft, '\u{eb14}'),
    (Icon::ChevronRight, '\u{eb16}'),
    (Icon::Home, '\u{ebea}'),
    (Icon::Map, '\u{ec7d}'),
    (Icon::Users, '\u{ede1}'),
    (Icon::User, '\u{eddf}'),
    (Icon::Server, '\u{ed20}'),
    (Icon::Wifi, '\u{ee01}'),
    (Icon::Reload, '\u{ecfa}'),
    (Icon::Link, '\u{ec5f}'),
];

impl Icon {
    /// Every icon, in declaration order.
    pub const ALL: [Icon; 34] = [
        Icon::Play,
        Icon::Pause,
        Icon::Earth,
        Icon::Moon,
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
    ];

    /// Returns the codepoint of the icon in the pixelarticons font.
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
