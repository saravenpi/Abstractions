//! `abstractions-server run`: load or make the world, listen, and save on the way out.

use std::fs;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::Path;

use abstractions_net::io::{ServeConfig, serve};
use abstractions_net::lan::lan_ip;
use abstractions_net::room::{Room, RoomConfig};
use abstractions_net::store::{self, FILE, World};
use abstractions_net::trust::short_key;

use crate::VERSION;
use crate::banner::{Banner, banner};
use crate::cli::Options;
use crate::saving::Saving;
use crate::signal;

/// Name of a new world when no `--name` is given.
const NEW_WORLD: &str = "Abstractions server";

/// Runs the server until a stop signal, then saves the world.
pub fn run(options: Options) -> Result<(), String> {
    let data = &options.data;
    let shown = data.display();
    fs::create_dir_all(data)
        .map_err(|error| format!("Could not make the folder {shown}: {error}"))?;
    let (room, name, seed) = room(&options)?;
    let saving = Saving::new(room, data.clone(), options.max_players)
        .map_err(|error| format!("Could not start the saver: {error}"))?;
    let mut signals =
        signal::listen().map_err(|error| format!("Could not catch signals: {error}"))?;
    let bind = SocketAddr::from((Ipv4Addr::UNSPECIFIED, options.port));
    let config = ServeConfig {
        bind,
        dir: data.clone(),
    };
    let handle = serve(config, saving)
        .map_err(|error| format!("Could not listen on UDP {bind}: {error}"))?;
    let port = handle.local_addr().port();
    announce(&options, &name, seed, port, &handle.fingerprint());
    signal::wait(&mut signals);
    let saving = handle.stop();
    saving
        .finish()
        .map_err(|error| format!("Could not save the world in {shown}: {error}"))?;
    println!("Saved, bye");
    Ok(())
}

/// Returns the room of the world, named and sized as asked, with its name and seed. The place
/// kept of each player is checked first, and every repair is logged.
fn room(options: &Options) -> Result<(Room, String, u64), String> {
    let world = checked(world(options)?, &options.data)?;
    let seed = world.seed;
    let name = options.name.clone().unwrap_or_else(|| world.name.clone());
    let config = RoomConfig {
        name: name.clone(),
        password: options.password.clone(),
        cap: options.max_players,
        version: VERSION.to_string(),
    };
    Ok((Room::new(config, world, 0.0), name, seed))
}

/// Returns `world` with the place kept of each player checked. Every repair is logged, and a
/// repaired world is written to `data` at once, so the same repair is not made at every start.
fn checked(mut world: World, data: &Path) -> Result<World, String> {
    let lines = store::place::mend_all(&mut world);
    for line in &lines {
        println!("{line}");
    }
    if !lines.is_empty() {
        store::save(data, &world)
            .map_err(|error| format!("Could not write {}: {error}", data.join(FILE).display()))?;
    }
    Ok(world)
}

/// Prints the banner of the server `name` listening on `port` with the certificate
/// `fingerprint`.
fn announce(options: &Options, name: &str, seed: u64, port: u16, fingerprint: &[u8; 32]) {
    let file = options.data.join(FILE);
    let key = short_key(fingerprint);
    let info = Banner {
        version: VERSION,
        name,
        seed,
        file: &file,
        port,
        lan: lan_ip(),
        key: &key,
        players: options.max_players,
        password: options.password.is_some(),
    };
    print!("{}", banner(&info));
}

/// Returns the world to serve: imported from `--import`, or loaded, or made new.
fn world(options: &Options) -> Result<World, String> {
    let data = &options.data;
    let Some(source) = &options.import else {
        let hint = options.name.as_deref().unwrap_or(NEW_WORLD);
        return store::load_or_create(data, hint)
            .map_err(|error| format!("Could not load {}: {error}", data.join(FILE).display()));
    };
    imported(source, data)
}

/// Converts the single-player world at `source` into `data/world.toml`, which must not exist.
fn imported(source: &Path, data: &Path) -> Result<World, String> {
    let file = data.join(FILE);
    if file.exists() {
        return Err(format!(
            "{} already exists, so nothing was imported. Move it away or delete it, then import again.",
            file.display()
        ));
    }
    let world = store::import(source)
        .map_err(|error| format!("Could not import {}: {error}", source.display()))?;
    store::save(data, &world)
        .map_err(|error| format!("Could not write {}: {error}", file.display()))?;
    println!("Imported {} into {}", source.display(), file.display());
    Ok(world)
}
