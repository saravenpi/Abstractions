//! Editing and display of the lighting settings of the graphics tab.

use super::field::Field;
use super::format::on_off;
use crate::settings::{Graphics, ShadowDetail, ShadowReach};

/// Returns the next shadow detail: the two steps swap.
fn other_detail(detail: ShadowDetail) -> ShadowDetail {
    match detail {
        ShadowDetail::Low => ShadowDetail::High,
        ShadowDetail::High => ShadowDetail::Low,
    }
}

/// Moves the shadow reach `dir` steps along near, normal and far, staying on the ends.
fn moved_reach(reach: ShadowReach, dir: i32) -> ShadowReach {
    let all = [ShadowReach::Near, ShadowReach::Normal, ShadowReach::Far];
    let at = all.iter().position(|&r| r == reach).unwrap_or(1) as i32;
    all[(at + dir).clamp(0, 2) as usize]
}

/// Steps a lighting setting by `dir`: the preset walks, toggles flip, the water reflections
/// cycle, the others move on their own steps. Any other field is left alone.
pub fn step(g: &mut Graphics, field: Field, dir: i32) {
    let own = &mut g.lighting;
    match field {
        Field::LightingQuality => g.step_quality(dir),
        Field::ShadowDetail => own.shadow_detail = other_detail(own.shadow_detail),
        Field::ShadowDistance => own.shadow_reach = moved_reach(own.shadow_reach, dir),
        Field::CloudShadows => own.cloud_shade = !own.cloud_shade,
        Field::AutoExposure => own.auto_exposure = !own.auto_exposure,
        Field::WhiteBalance => own.white_balance = !own.white_balance,
        Field::LocalShadows => own.local_shadows = !own.local_shadows,
        Field::WaterGlint => own.water_glint = !own.water_glint,
        Field::WaterReflections => own.reflections = own.reflections.next(),
        _ => {}
    }
}

/// Returns the shown value of a lighting setting, or `None` when the field is not one.
pub fn display(g: &Graphics, field: Field) -> Option<String> {
    let own = &g.lighting;
    let text = match field {
        Field::LightingQuality => g.quality().name().to_string(),
        Field::ShadowDetail => own.shadow_detail.name().to_string(),
        Field::ShadowDistance => own.shadow_reach.name().to_string(),
        Field::CloudShadows => on_off(own.cloud_shade),
        Field::AutoExposure => on_off(own.auto_exposure),
        Field::WhiteBalance => on_off(own.white_balance),
        Field::LocalShadows => on_off(own.local_shadows),
        Field::WaterGlint => match own.water_glint {
            true => "Sky and sun".to_string(),
            false => "Sky only".to_string(),
        },
        Field::WaterReflections => own.reflections.name().to_string(),
        _ => return None,
    };
    Some(text)
}
