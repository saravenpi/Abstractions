//! The plain one-line description of each graphics setting, shown under the list.

use super::field::Field;

/// The line shown when no graphics row is focused: what costs the most.
pub const COSTS: &str = "Costs most: render distance, shadows, ambient occlusion, shadow distance.";

/// Returns the description of a graphics setting, with what it costs, or an empty line for a
/// field that is not one.
pub fn describe(field: Field) -> &'static str {
    match field {
        Field::RenderDistance => "How far the ground is drawn. The biggest cost on a weak GPU.",
        Field::LightingQuality => {
            "Sets every lighting option at once. Change one and it reads Custom."
        }
        Field::Shadows => "Sun and moon shadows. Costs a lot. Every light stays on without them.",
        Field::ShadowDetail => "How sharp shadows are. Low uses a quarter of the memory.",
        Field::ShadowDistance => "How far shadows reach. Longer costs more, 220 m adds a pass.",
        Field::AmbientOcclusion => "Darkens creases and corners near the ground. Costs a lot.",
        Field::Bloom => "A soft glow round very bright lights. Costs little.",
        Field::Clouds => "Clouds in the sky.",
        Field::CloudShadows => "Clouds shade the ground as they pass over it. Costs little.",
        Field::AutoExposure => "The view adapts to bright and dark as an eye does. Costs nothing.",
        Field::WhiteBalance => {
            "The view adapts its white to the colour of the light. Costs nothing."
        }
        Field::LocalShadows => "The robot lamp casts shadows. Costs a lot. Ultra only.",
        Field::WaterGlint => "Sun and moon glint on the sea, not only the sky. Costs little.",
        _ => "",
    }
}
