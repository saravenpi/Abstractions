//! The bot's command line.

/// What `--help` and a mistake print.
pub const USAGE: &str =
    "Usage: bot <address>[,<address>..] [--count 1..15] [--seconds S] [--password P]";

/// Most bots at once: a server of 16 seats keeps one for a real player.
const MAX_COUNT: usize = 15;

/// What the bot was asked to do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Args {
    /// Addresses of the server, used in turn by the bots.
    pub addresses: Vec<String>,
    /// Number of bots.
    pub count: usize,
    /// Seconds to run, or nothing to run until stopped.
    pub seconds: Option<u64>,
    /// Password of the server, if it has one.
    pub password: Option<String>,
}

/// Reads the arguments after the program name.
pub fn parse(mut args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut parsed = Args {
        addresses: addresses(args.next())?,
        count: 1,
        seconds: None,
        password: None,
    };
    while let Some(flag) = args.next() {
        let value = args.next().ok_or(format!("{flag} needs a value"))?;
        let number = || {
            value
                .parse::<u64>()
                .map_err(|_| format!("{flag} takes a number, not `{value}`"))
        };
        match flag.as_str() {
            "--count" => parsed.count = number()? as usize,
            "--seconds" => parsed.seconds = Some(number()?),
            "--password" => parsed.password = Some(value),
            _ => return Err(format!("Unknown option `{flag}`")),
        }
    }
    if parsed.count == 0 || parsed.count > MAX_COUNT {
        return Err(format!("--count goes from 1 to {MAX_COUNT}"));
    }
    Ok(parsed)
}

/// Returns the addresses of the first argument, split on commas.
fn addresses(first: Option<String>) -> Result<Vec<String>, String> {
    let first = first
        .filter(|arg| !arg.starts_with('-'))
        .unwrap_or_default();
    let addresses: Vec<String> = first
        .split(',')
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect();
    match addresses.is_empty() {
        true => Err("No address given".to_string()),
        false => Ok(addresses),
    }
}
