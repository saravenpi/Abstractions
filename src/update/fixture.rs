//! Made-up release data for tests: a checksum and the document GitHub would serve.

use sha2::{Digest, Sha256};

use super::allow::RELEASES;

/// Returns the SHA-256 of `bytes` as lowercase hexadecimal.
pub fn sha256(bytes: &[u8]) -> String {
    let hash = Sha256::digest(bytes);
    hash.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Returns the release document GitHub would serve for `tag`, listing `files` by name and size.
pub fn document(tag: &str, files: &[(String, usize)]) -> Vec<u8> {
    let assets: Vec<serde_json::Value> = files
        .iter()
        .map(|(name, size)| {
            let url = format!("{RELEASES}{tag}/{name}");
            serde_json::json!({ "name": name, "browser_download_url": url, "size": size })
        })
        .collect();
    let body = "## What's Changed\n* **Rivers** run to the sea\n- Fixed `moons`\n\nThanks";
    let release = serde_json::json!({
        "tag_name": tag, "published_at": "2026-11-20T09:30:00Z", "body": body,
        "prerelease": false, "draft": false, "assets": assets, "author": { "login": "someone" },
    });
    release.to_string().into_bytes()
}
