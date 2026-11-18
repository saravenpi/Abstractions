//! A release as GitHub describes it: its version, date, notes and files.

use serde::Deserialize;

use super::failure::Failure;
use super::version::{self, Version};

/// Most lines of release notes kept.
const NOTE_LINES: usize = 60;
/// Most characters of one line of release notes kept.
const NOTE_CHARS: usize = 160;

/// One file of a release.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Asset {
    /// File name.
    pub name: String,
    /// Where the file is downloaded from.
    #[serde(rename = "browser_download_url")]
    pub url: String,
    /// Size in bytes, as the release declares it.
    pub size: u64,
}

/// The release document of the GitHub API, reduced to what the updater reads.
#[derive(Debug, Deserialize)]
struct Document {
    /// Tag of the release, such as `v0.9.1`.
    tag_name: String,
    /// Publication time, ISO 8601.
    published_at: Option<String>,
    /// Release notes, in markdown.
    body: Option<String>,
    /// Whether the release is marked as a pre-release.
    #[serde(default)]
    prerelease: bool,
    /// Whether the release is an unpublished draft.
    #[serde(default)]
    draft: bool,
    /// Files of the release.
    #[serde(default)]
    assets: Vec<Asset>,
}

/// A published release of the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// Version, read from the tag.
    pub version: Version,
    /// Tag as published, which is part of every file name.
    pub tag: String,
    /// Day of publication, `YYYY-MM-DD`, or nothing when unknown.
    pub date: String,
    /// Release notes as plain lines.
    pub notes: Vec<String>,
    /// Whether it is a full release: not a draft, not a pre-release, no suffix in its tag.
    pub stable: bool,
    /// Files of the release.
    pub assets: Vec<Asset>,
}

/// Returns one line of markdown as plain text: heading marks, bullet marks, bold marks and code
/// ticks are dropped, a bullet starts with a dash, and a long line is cut.
fn plain(line: &str) -> String {
    let line = line.trim();
    let bullet = line.starts_with("* ") || line.starts_with("- ");
    let text = line.trim_start_matches(['#', '*', '-', ' ']);
    let text = text.replace("**", "").replace('`', "");
    let lead = if bullet { "- " } else { "" };
    format!("{lead}{text}").chars().take(NOTE_CHARS).collect()
}

/// Returns release notes as plain lines: markdown marks removed, empty lines dropped, and only
/// the first lines kept.
pub fn notes(body: &str) -> Vec<String> {
    body.lines()
        .map(plain)
        .filter(|line| !line.is_empty())
        .take(NOTE_LINES)
        .collect()
}

/// Reads the release document of the GitHub API.
pub fn parse(json: &[u8]) -> Result<Release, Failure> {
    let document: Document = serde_json::from_slice(json).map_err(|_| Failure::Unreadable)?;
    let version = version::parse(&document.tag_name).ok_or(Failure::Unreadable)?;
    let day = document.published_at.unwrap_or_default();
    Ok(Release {
        version,
        date: day.chars().take(10).collect(),
        notes: notes(&document.body.unwrap_or_default()),
        stable: !document.prerelease && !document.draft && !document.tag_name.contains('-'),
        tag: document.tag_name,
        assets: document.assets,
    })
}
