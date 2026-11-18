//! The short help `--help` prints.

/// What `abstractions-server --help` prints.
pub const USAGE: &str = "\
Usage: abstractions-server [run] [options]
       abstractions-server ping [address]   (this machine when none)
       abstractions-server version

Options (a flag wins over its environment variable):
  --name <name>          name in the server list      ABSTRACTIONS_NAME
  --password <password>  password players must give   ABSTRACTIONS_PASSWORD
  --port <port>          UDP port (24650)             ABSTRACTIONS_PORT
  --max-players <n>      players at once (16)         ABSTRACTIONS_MAX_PLAYERS
  --data <folder>        world and key (./data)       ABSTRACTIONS_DATA
  --import <file>        start from a single-player world file
";
