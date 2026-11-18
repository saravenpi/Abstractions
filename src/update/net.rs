//! The real host: HTTPS to GitHub with a blocking client, and the system's process launcher.
//!
//! Every request is HTTPS, carries a generic user agent and nothing else about the player, and
//! follows a redirect only to GitHub or one of its file hosts.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

use ureq::http::Response;
use ureq::{Agent, Body};

use super::allow;
use super::failure::Failure;
use super::host::{Host, Sink};

/// Most redirects followed for one file.
const HOPS: usize = 5;
/// What the updater calls itself. It names no player and no computer.
const USER_AGENT: &str = "abstractions-updater";

/// The network and the programs of this computer.
#[derive(Debug, Clone, Copy, Default)]
pub struct Net;

/// Returns a client that connects within 5 seconds, waits 10 for an answer and 20 for each
/// read, and gives up after `total` when one is given. It follows no redirect by itself.
fn agent(total: Option<Duration>) -> Agent {
    Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .http_status_as_error(false)
        .user_agent(USER_AGENT)
        .timeout_connect(Some(Duration::from_secs(5)))
        .timeout_recv_response(Some(Duration::from_secs(10)))
        .timeout_per_read(Some(Duration::from_secs(20)))
        .timeout_global(total)
        .build()
        .into()
}

/// Returns the failure a player is told for a client error.
fn failure(error: ureq::Error) -> Failure {
    match error {
        ureq::Error::Timeout(_) => Failure::Timeout,
        ureq::Error::StatusCode(code) => Failure::Status(code),
        ureq::Error::BodyExceedsLimit(_) => Failure::TooLarge,
        _ => Failure::Offline,
    }
}

/// Requests `url` and follows up to [`HOPS`] redirects, each only to an allowed address.
/// Returns the first answer that is a success.
fn open(agent: &Agent, url: &str) -> Result<Response<Body>, Failure> {
    let mut at = url.to_string();
    for _ in 0..HOPS {
        let response = agent.get(&at).call().map_err(failure)?;
        let status = response.status().as_u16();
        if !(300..400).contains(&status) {
            return match status {
                200 => Ok(response),
                code => Err(Failure::Status(code)),
            };
        }
        let to = response.headers().get("location");
        let to = to.and_then(|value| value.to_str().ok());
        at = to
            .filter(|to| allow::redirect(to))
            .ok_or(Failure::Forbidden)?
            .to_string();
    }
    Err(Failure::Forbidden)
}

impl Host for Net {
    fn get(&self, url: &str, limit: u64) -> Result<Vec<u8>, Failure> {
        let mut response = open(&agent(Some(Duration::from_secs(10))), url)?;
        let body = response.body_mut().with_config().limit(limit);
        body.read_to_vec().map_err(failure)
    }

    fn stream(&self, url: &str, sink: Sink) -> Result<(), Failure> {
        let mut response = open(&agent(None), url)?;
        let mut reader = response.body_mut().as_reader();
        let mut chunk = vec![0u8; 64 * 1024];
        loop {
            let read = reader.read(&mut chunk).map_err(|_| Failure::Offline)?;
            if read == 0 {
                return Ok(());
            }
            sink(&chunk[..read])?;
        }
    }

    fn launch(&self, program: &Path, args: &[String]) -> Result<(), Failure> {
        let mut command = Command::new(program);
        command.args(args).stdin(Stdio::null());
        command.spawn().map(|_| ()).map_err(Failure::from)
    }
}
