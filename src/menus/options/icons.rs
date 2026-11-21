//! The icon of each setting: what tells a row apart from its neighbours at a glance.

use super::field::Field;
use crate::ui::{Icon, sign};

/// Returns the icon of a setting, or nothing for one whose concept has no glyph in the icon font
/// yet. Such a row keeps the room of its icon, so labels stay on one line. The glyphs wanted,
/// by Material Symbols name: `ev_shadow` for the shadow rows, `blur_circular` for ambient
/// occlusion, `exposure`, `wb_auto`, `sync` for vsync, `fullscreen`, `panorama_horizontal` for
/// the field of view, `vibration` for head bob, `videocam` for the camera that follows,
/// `music_note`, `graphic_eq` for effects, `bloodtype` and `animation` for reduced motion.
pub fn of(field: Field) -> Option<Icon> {
    let icon = match field {
        Field::RenderDistance | Field::ShadowDistance => Icon::Distance,
        Field::LightingQuality => Icon::Light,
        Field::LocalShadows => sign::LAMP,
        Field::Bloom => sign::BLOOM,
        Field::Clouds => Icon::Cloudy,
        Field::WaterGlint => Icon::Water,
        Field::WaterReflections => sign::ON_FOOT,
        Field::ShowFps => sign::FRAME_RATE,
        Field::Sensitivity => Icon::Mouse,
        Field::InvertY => sign::INVERT,
        Field::ViewFoot => sign::ON_FOOT,
        Field::ViewShip => Icon::ShipParked,
        Field::SpeedFx => Icon::Speed,
        Field::Master => Icon::Volume,
        Field::CheckUpdates => sign::UPDATES,
        Field::Hitstop => sign::HITSTOP,
        Field::WorldMarkers => Icon::MapPin,
        Field::Interface => sign::INTERFACE_SIZE,
        _ => return None,
    };
    Some(icon)
}
