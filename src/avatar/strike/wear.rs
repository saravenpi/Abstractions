//! Wear: every blow that lands counts on the joints of the tool that struck it.

use bevy::prelude::*;

use super::hit::Hit;
use crate::craft::Worn;

/// Tells the pack that the tool of each landed blow struck.
pub fn worn(mut hits: MessageReader<Hit>, mut wear: MessageWriter<Worn>) {
    for hit in hits.read() {
        wear.write(Worn(hit.item));
    }
}
