//! Looking for a newer release.

use super::host::Host;
use super::release;
use super::state::Status;
use super::version;

/// Where GitHub describes the latest full release of the game.
pub const LATEST: &str = "https://api.github.com/repos/saravenpi/Abstractions/releases/latest";

/// Largest release document accepted, in bytes.
const LIMIT: u64 = 1024 * 1024;

/// Asks `host` for the latest release and compares it with the version `current`. Nothing is
/// sent but the request itself. A draft, a pre-release or an older release counts as up to date.
pub fn check(host: &dyn Host, current: &str) -> Status {
    let found = host
        .get(LATEST, LIMIT)
        .and_then(|json| release::parse(&json));
    match found {
        Err(failure) => Status::Failed(failure.reason()),
        Ok(release) if release.stable && version::newer(current, &release.tag) => {
            Status::Available(release)
        }
        Ok(_) => Status::UpToDate,
    }
}
