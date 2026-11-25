//! Frame integrity lines and health tone helpers for the robot condition readout.

#[cfg(test)]
use bevy::prelude::*;

#[cfg(test)]
use crate::condition::Condition;
#[cfg(test)]
use crate::ui::theme;

/// Returns the lines describing `condition`: the integrity of the frame, then each hurt part
/// with its health, worst first. Nothing for a body that is whole.
#[cfg(test)]
pub fn lines(condition: &Condition) -> Vec<String> {
    let hurt = condition.damaged();
    if hurt.is_empty() {
        return Vec::new();
    }
    let share = |health: f32| (health * 100.0).round();
    let mut lines = vec![format!("Frame {}%", share(condition.integrity()))];
    lines.extend(
        hurt.into_iter()
            .map(|part| format!("{} {}%", part.name(), share(condition.of(part)))),
    );
    lines
}

/// Returns the colour of a frame row that reads `health`, the integrity of the frame or the
/// health of one part: the warning colour under half and the danger colour under a quarter.
#[cfg(test)]
pub fn tone(health: f32) -> Color {
    match health {
        low if low < 0.25 => theme::DANGER,
        mid if mid < 0.5 => theme::WARN,
        _ => theme::TEXT,
    }
}

/// Returns the colour of each line of [`lines`] for `condition`: the first by the integrity of
/// the whole frame, each part by its own health, so a broken part reads as danger while the
/// rest of the frame is sound.
#[cfg(test)]
pub fn tones(condition: &Condition) -> Vec<Color> {
    let hurt = condition.damaged();
    if hurt.is_empty() {
        return Vec::new();
    }
    let mut tones = vec![tone(condition.integrity())];
    tones.extend(hurt.into_iter().map(|part| tone(condition.of(part))));
    tones
}
