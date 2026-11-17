# Local patches to bevy_solari

This directory is a patched copy of `bevy_solari` 0.20.1, wired in by `[patch.crates-io]` in the root `Cargo.toml`.

- Upstream crate: https://crates.io/crates/bevy_solari/0.20.1
- Upstream commit: `a0577b5c6a7b208719dd761c5c6a7a539e472a97`, path `crates/bevy_solari` in https://github.com/bevyengine/bevy
- Crate checksum (sha256 of the `.crate` file): `17e09260dd2001f2aeccc671a27ca71fc78f4da211a128596f1014e44424960b`
- Removed from the published crate: `Cargo.lock`, `Cargo.toml.orig`, `.cargo_vcs_info.json`, `.cargo-ok`. `Cargo.toml` is the published, normalized manifest, unchanged.
- The first commit that adds this directory holds the unmodified crate. `git diff <that commit> -- vendor/bevy_solari` shows every patch.

## Patch 1: size the texture and sampler arrays from the device

Why: the scene bind group layout declares 5000 samplers. Apple7 and Apple8 GPUs (M1, M2) allow 1024 in wgpu 30 (`wgpu-hal-30.0.2/src/metal/adapter.rs`, `max_sampler_binding_array_elements`) and 996 in Apple's Metal Feature Set Tables (https://developer.apple.com/metal/Metal-Feature-Set-Tables.pdf, page 8, edition of October 23, 2026). The layout fails validation and Bevy quits. Upstream report: https://github.com/bevyengine/bevy/issues/26131 (open on 2026-11-17, no fix on `main`).

The shaders need no change: `bindings.wesl` declares `binding_array<texture_2d<f32>>` and `binding_array<sampler>` without a length.

| File | Change |
|---|---|
| `src/scene/binder/assets.rs` | New `CONSTRAINED_TEXTURE_COUNT` (256) and `texture_slot_capacity(sampler_array_limit)`: 5000 when the device allows 5000, else `min(limit, 256)`. |
| `src/scene/binder/assets.rs` | `AssetState` gets a `texture_capacity` field, set by `AssetState::new(texture_capacity)`. The four uses of `MAX_TEXTURE_COUNT.get()` in `update_material` (two `vacancies` checks, the warning text, the `acquire` call) read it instead, so no texture id reaches the array length. |
| `src/scene/binder/assets.rs` | New test `texture_slot_capacity_never_exceeds_the_device_limit`. |
| `src/scene/binder/mod.rs` | `RaytracingSceneBindings::from_world` reads `max_binding_array_sampler_elements_per_shader_stage` from the device and uses `texture_slot_capacity` for both `.count(..)` calls and for `AssetState::new`. Re-exports `texture_slot_capacity`. |
| `src/scene/mod.rs` | Re-exports `texture_slot_capacity`, so the game checks the device with the same function (`src/render/ray_tracing/gpu.rs`). |
| `src/scene/binder/allocator.rs` | New test `retained_binding_array_never_hands_out_a_slot_past_capacity`. No code change. |

Behaviour on devices that allow 5000 samplers is unchanged. On the others a scene can bind 256 distinct textures; materials past that are skipped with the upstream warning.

## Patch 2: indirect dispatch arguments as a struct (backport)

Why: `world_cache_compact.wesl` bound a 12-byte buffer as a storage `vec3<u32>`. Metal gives that vector 16 bytes, and Metal API validation aborts. This is upstream pull request https://github.com/bevyengine/bevy/pull/26100, merged on `main` on 2026-11-04 (commit `7e987740ac19fb2e42503a94213f9914fddeb0d1`) and not in 0.20.1.

| File | Change |
|---|---|
| `src/realtime/world_cache_compact.wesl` | `world_cache_active_cells_dispatch` is a `DispatchIndirectArgs { x, y, z: u32 }` struct instead of `vec3<u32>`; the one assignment builds the struct. Same diff as the upstream pull request. |

## When to drop this directory

- Patch 2: as soon as a `bevy_solari` release contains pull request 26100.
- Patch 1: when upstream sizes the sampler array from the device limit (watch issue 26131).

Then delete `vendor/bevy_solari`, remove the `[patch.crates-io]` entry, and replace `bevy::solari::scene::texture_slot_capacity` in `src/render/ray_tracing/gpu.rs` with the upstream equivalent.
