# Abstractions

A first-person exploration game with a pixelated, low-poly look. You wake up in an endless ocean of procedurally generated islands. Walk, swim and look around under a warm sun.

## Controls

| Input | Action |
| --- | --- |
| W A S D | Move |
| Mouse | Look |
| Space | Jump, or swim up |
| Shift | Sprint |
| Esc | Pause menu |

Rebind every key in Settings > Controls. New game lets you type a seed, or leave it empty for a random world.

## Run

You need Rust and [mise](https://mise.jdx.dev).

```sh
mise run dev
mise run run
```

`dev` builds fast with dynamic linking. `run` builds an optimized release. The release binary has no assets folder: the shader is embedded.

Other tasks: `mise run test`, `mise run check`, `mise run build-release`.

## macOS

Download the `aarch64-apple-darwin` archive (Apple Silicon) or `x86_64-apple-darwin` (Intel). The binary is unsigned, so remove the quarantine flag once, then run it:

```sh
tar -xzf abstractions-*-apple-darwin.tar.gz
cd abstractions-*-apple-darwin
xattr -d com.apple.quarantine ./abstractions
./abstractions
```

## Settings

Settings live in `settings.toml` inside the `abstractions` folder of your config directory:

- Linux: `~/.config/abstractions/settings.toml`
- macOS: `~/Library/Application Support/abstractions/settings.toml`
- Windows: `%APPDATA%\abstractions\settings.toml`

A missing or broken file is replaced by the defaults.

## Source layout

- `src/lib.rs`: root plugin that wires every module together.
- `src/screens`: the `Screen` state (title, loading, gameplay).
- `src/menus`: title, pause and settings menus.
- `src/ui`: shared UI widgets.
- `src/world`: terrain, chunk streaming, props and sea.
- `src/player`: first-person movement, jump and swimming.
- `src/render`: pixelate post-process, sun, sky and fog.
- `src/settings`: persisted settings and the code that applies them.
