//! The addresses the updater may fetch: the game's own release files and GitHub's file hosts.

/// Where every release file of the game is published.
pub const RELEASES: &str = "https://github.com/saravenpi/Abstractions/releases/download/";

/// Returns whether `url` is a file of one of the game's releases: under [`RELEASES`], made of
/// plain file name characters, and never climbing out with `..`.
pub fn download(url: &str) -> bool {
    let Some(rest) = url.strip_prefix(RELEASES) else {
        return false;
    };
    let plain = |byte: u8| byte.is_ascii_alphanumeric() || b"._-/".contains(&byte);
    !rest.is_empty() && rest.bytes().all(plain) && !rest.contains("..") && !rest.contains("//")
}

/// Returns whether a download may be redirected to `url`: HTTPS only, to GitHub itself or to one
/// of its file hosts under `githubusercontent.com`, with no user name and no port.
pub fn redirect(url: &str) -> bool {
    let Some(rest) = url.strip_prefix("https://") else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    let named = host
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'.' || byte == b'-');
    let ours = host == "github.com" || host.ends_with(".githubusercontent.com");
    named && ours
}
