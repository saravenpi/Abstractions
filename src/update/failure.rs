//! Why a check or an install stopped, in words a player can read.

/// What stopped a check, a download or an install.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Failure {
    /// The player cancelled.
    Cancelled,
    /// The server could not be reached.
    Offline,
    /// The server did not answer in time.
    Timeout,
    /// The server answered with this HTTP status instead of the file.
    Status(u16),
    /// The answer could not be read as a release.
    Unreadable,
    /// An address is outside the game's release page on GitHub.
    Forbidden,
    /// The file is larger than the game accepts.
    TooLarge,
    /// The download does not have the size the release declares.
    Size,
    /// The download does not have the checksum the release lists.
    Checksum,
    /// The release has no file for this computer.
    Unsupported,
    /// A file could not be written, moved or started: what the system said.
    Disk(String),
}

impl Failure {
    /// Returns the reason as one short plain sentence, without a full stop.
    pub fn reason(&self) -> String {
        match self {
            Failure::Cancelled => "Cancelled".into(),
            Failure::Offline => "No connection".into(),
            Failure::Timeout => "No answer in time".into(),
            Failure::Status(code) => format!("Server error {code}"),
            Failure::Unreadable => "Unreadable answer".into(),
            Failure::Forbidden => "The download address is not on the game's release page".into(),
            Failure::TooLarge => "The download is larger than 500 MB".into(),
            Failure::Size => "The download is incomplete".into(),
            Failure::Checksum => "The download is damaged: its checksum does not match".into(),
            Failure::Unsupported => "This release has no file for this computer".into(),
            Failure::Disk(what) => format!("The system reported: {what}"),
        }
    }
}

impl From<std::io::Error> for Failure {
    fn from(error: std::io::Error) -> Self {
        Failure::Disk(error.to_string())
    }
}
