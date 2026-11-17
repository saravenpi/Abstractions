//! The interface font: Undefined Medium, embedded in the binary and used for all text.
//!
//! The file is `undefined-medium.otf` from https://github.com/andirueckel/undefined-medium,
//! unmodified, under the SIL Open Font License 1.1 (`LICENSES/Undefined-Medium-OFL.txt`).
//! It is never drawn bold: no text of the interface sets a weight.

use bevy::prelude::*;

/// Bytes of the font file.
pub(super) const UNDEFINED_OTF: &[u8] = include_bytes!("../../assets/fonts/undefined-medium.otf");

/// Makes Undefined Medium the default font, so that every text uses it without naming it.
/// Does nothing in an app without text.
pub(super) fn load(fonts: Option<ResMut<Assets<Font>>>) {
    if let Some(mut fonts) = fonts
        && let Err(error) =
            fonts.insert(AssetId::default(), Font::from_bytes(UNDEFINED_OTF.to_vec()))
    {
        warn!("could not register the interface font: {error}");
    }
}
