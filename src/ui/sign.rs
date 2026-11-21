//! The icon vocabulary: one concept, one icon, on every screen. A variant of [`Icon`] is named
//! after the concept it draws, and that name is the vocabulary. This file holds the concepts
//! that have no glyph of their own yet and borrow the nearest one, each with the Material
//! Symbols name of the glyph it wants, so the swap is one line when the font gains it.

use super::icon::Icon;

/// The glow round bright lights. Wants its own variant on `flare`, which `Radial` draws today.
pub const BLOOM: Icon = Icon::Radial;
/// Turning the vertical axis of the mouse over. `swap_vert` is the glyph it wants: it shares
/// it with sorting.
pub const INVERT: Icon = Icon::Sort;
/// The short stop of the picture on a hit. Wants `front_hand`.
pub const HITSTOP: Icon = Icon::Pause;
/// The size of the interface. Wants `format_size`.
pub const INTERFACE_SIZE: Icon = Icon::Zoom;
/// The lamp of the robot.
pub const LAMP: Icon = Icon::HeadlightOn;
/// The robot on foot, as opposed to the ship.
pub const ON_FOOT: Icon = Icon::User;
/// The frames drawn each second.
pub const FRAME_RATE: Icon = Icon::Speed;
/// Looking for a newer release.
pub const UPDATES: Icon = Icon::Reload;

/// Every borrowed icon: the concept, the icon it borrows and the Material Symbols name of the
/// glyph it wants. An empty name means the borrowed glyph is the wanted one.
pub const BORROWED: [(&str, Icon, &str); 8] = [
    ("bloom", BLOOM, "flare"),
    ("invert the mouse", INVERT, ""),
    ("hitstop", HITSTOP, "front_hand"),
    ("interface size", INTERFACE_SIZE, "format_size"),
    ("robot lamp", LAMP, ""),
    ("on foot", ON_FOOT, ""),
    ("frame rate", FRAME_RATE, ""),
    ("updates", UPDATES, ""),
];
