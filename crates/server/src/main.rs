//! The self-hosted server of Abstractions. It only prints its version for now.

/// Prints the name and the version of the server.
fn main() {
    println!("abstractions-server {}", env!("CARGO_PKG_VERSION"));
}
