//! One run of consecutive leaf keys, as it travels and is stored.

use serde::{Deserialize, Serialize};

/// Consecutive leaf keys, placed by the distance from the end of the run before it so the
/// numbers stay small and a varint holds them in a byte or two.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Run {
    /// Keys between the end of the previous run and the start of this one. The first run counts
    /// from 0.
    pub gap: u64,
    /// Number of keys in the run.
    pub len: u64,
}
