//! Shared setup of the settings tests.

use super::field::Field;
use crate::settings::{KeyLabels, Settings};

/// Formats the value of `field` as the menu shows it on a QWERTY keyboard.
pub fn display(settings: &Settings, field: Field) -> String {
    super::format::display(settings, &KeyLabels::default(), field)
}
