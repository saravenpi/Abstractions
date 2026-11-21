//! Icons, drawn as glyphs of the Material Symbols Sharp icon font embedded in the binary.
//!
//! The look follows the owner's direction of 2026-11-20: square corners, no borders, no round
//! caps. The font is a subset of `MaterialSymbolsSharp[FILL,GRAD,opsz,wght].ttf`, instanced at
//! weight 400, FILL 0, GRAD 0, optical size 24, cut to the glyphs below. `packaging/icons.py`
//! builds it and writes `icon/codes.rs` from the names on the doc lines of [`Icon`]: to add an
//! icon, add its variant with its Material name and run the script. The source and its commit
//! are in `packaging/icons.md`. The licence (Apache-2.0) is in `LICENSES/`. Icons are real
//! pictures with a name on screen somewhere: an icon never stands alone without a text, and
//! no black box sits behind it. Colour comes from the tokens of `ui::theme`: `ACCENT` for the
//! course, `PIN` for pins, `WARN` and `DANGER` for trouble, `TEXT` for home and the ship.

use bevy::asset::uuid_handle;
use bevy::prelude::*;

mod codes;

pub use codes::CODEPOINTS;

use super::style::Tone;
use super::theme;

/// Name of the font instance the embedded file comes from.
pub const ICONS_VERSION: &str = "Material Symbols Sharp, instance wght 400 FILL 0 opsz 24";

/// Bytes of the icon font file.
pub(super) const ICONS_TTF: &[u8] =
    include_bytes!("../../assets/fonts/material-symbols-sharp-subset.ttf");

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
    /// `clear_night`
    ClearNight,
    /// `partly_cloudy_day`
    PartlyCloudy,
    /// `partly_cloudy_night`
    PartlyCloudyNight,
    /// `cloud`
    Cloudy,
    /// `rainy`
    Rain,
    /// `weather_snowy`
    Snow,
    /// `thunderstorm`
    Storm,
    /// `foggy`
    Fog,
    /// `grain`
    Dust,
    /// `light_mode`
    Light,
    /// `weight`
    Mass,
    /// `humidity_percentage`
    Moisture,
    /// `schedule`
    Clock,
    /// `hourglass_empty`
    Age,
    /// `orbit`
    Orbit,
    /// `distance`
    Distance,
    /// `altitude`
    Altitude,
    /// `height`
    Height,
    /// `target`
    Zone,
    /// `rotate_right`
    Tilt,
    /// `trip_origin`
    Rings,
    /// `storm`
    Spiral,
    /// `category`
    Class,
    /// `explore`
    Compass,
    /// `potted_plant`
    Plant,
    /// `park`
    Tree,
    /// `grass`
    Grass,
    /// `local_florist`
    Flower,
    /// `spa`
    Bulb,
    /// `beach_access`
    Cap,
    /// `waves`
    Kelp,
    /// `diamond`
    Crystal,
    /// `psychiatry`
    Vine,
    /// `warning`
    Danger,
    /// `skull`
    Skull,
    /// `flashlight_on`
    HeadlightOn,
    /// `flashlight_off`
    HeadlightOff,
    /// `flight_land`
    GearDown,
    /// `flight_takeoff`
    GearUp,
    /// `speed`
    Speed,
    /// `bolt`
    Throttle,
    /// `balance`
    Assist,
    /// `vertical_align_top`
    Apoapsis,
    /// `vertical_align_bottom`
    Periapsis,
    /// `route`
    Route,
    /// `search`
    Search,
    /// `filter_list`
    Filter,
    /// `mouse`
    Mouse,
    /// `unfold_more`
    Scroll,
    /// `zoom_in`
    Zoom,
    /// `visibility`
    Eye,
    /// `visibility_off`
    EyeOff,
    /// `question_mark`
    Unknown,
    /// `menu_book`
    Guide,
    /// `expand_more`
    ExpandMore,
    /// `expand_less`
    ExpandLess,
    /// `backpack`
    Pack,
    /// `construction`
    Workshop,
    /// `content_cut`
    Cut,
    /// `push_pin`
    Pierce,
    /// `hardware`
    Crush,
    /// `sports_handball`
    Throw,
    /// `handyman`
    Repair,
    /// `favorite`
    Health,
    /// `restaurant`
    Diet,
    /// `nutrition`
    Eats,
    /// `landscape`
    Terrain,
    /// `directions_run`
    Flees,
    /// `shield`
    Shield,
    /// `inventory_2`
    Yield,
    /// `egg`
    Egg,
    /// `workspaces`
    Herd,
    /// `footprint`
    Footprint,
    /// `skeleton`
    Skeleton,
    /// `sprint`
    Moves,
    /// `microbiology`
    Blob,
    /// `flare`
    Radial,
    /// `linear_scale`
    Segments,
    /// `hive`
    Swarm,
}

impl Icon {
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
