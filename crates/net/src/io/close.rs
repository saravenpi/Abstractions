//! The application codes a connection closes with, and what they mean.

use quinn::VarInt;

use crate::wire::Reason;

/// Code of a normal close: the player or the server is done.
pub(crate) const BYE: VarInt = VarInt::from_u32(0);

/// Code of a connection closed because it could not take what was sent fast enough.
pub(crate) const LAGGING: VarInt = VarInt::from_u32(100);

/// Code of a connection closed because its stream carried something that is not a message.
pub(crate) const BROKEN: VarInt = VarInt::from_u32(101);

/// Every reason, in the order of their codes: a reason's code is its place plus one.
const REASONS: [Reason; 8] = [
    Reason::WrongPassword,
    Reason::Full,
    Reason::Outdated,
    Reason::Newer,
    Reason::Flooding,
    Reason::Closing,
    Reason::Replaced,
    Reason::Silent,
];

/// Returns the code a connection closes with for `reason`.
pub(crate) fn code(reason: Reason) -> VarInt {
    let place = REASONS.iter().position(|known| *known == reason);
    place.map_or(BYE, |place| VarInt::from_u32(place as u32 + 1))
}

/// Returns the reason a close code stands for, if it stands for one.
pub(crate) fn reason(code: VarInt) -> Option<Reason> {
    let place = code.into_inner().checked_sub(1)?;
    REASONS.get(usize::try_from(place).ok()?).copied()
}

/// Returns the text sent with a close for `reason`, for logs on the other side: the words the
/// player reads, without a version.
pub(crate) fn text(reason: Reason) -> Vec<u8> {
    reason.words(None).into_bytes()
}
