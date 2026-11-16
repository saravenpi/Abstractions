# Changelog

All notable changes to this project are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [Unreleased]

## [0.3.0] - 2026-11-16

### Added

- Ray tracing as an experimental graphics setting, off by default. It needs a GPU with hardware ray tracing and a restart to apply.
- A startup guard: if the game fails to start with ray tracing on, the next launch turns it off.

### Changed

- World colors come from a palette texture instead of vertex colors, which ray tracing requires.

### Known limits

- Ray tracing has never run on real hardware. With it on, distance fog and underwater fog do not apply to the land, and nights may be darker.

## [0.2.0] - 2026-11-16

### Added

- Six biomes: meadow, forest, cherry grove, desert, taiga and tropical, each with its own ground colors.
- Mountains up to about 100 m, plateaus, irregular coastlines and islands of very different sizes.
- Biome plants: oaks, pines, cherry trees, palms, cacti, ferns, mushrooms, reeds and flowers in several colors.
- Eleven kinds of structures to find, such as stone circles, ruined towers, cabins, lighthouses, pyramids, docks and shrines.
- Day and night cycle of 20 minutes with a sun, a moon, stars and drifting clouds.
- Underwater fog and tint.
- Saved worlds: named worlds, a worlds list, Continue, and autosave of position, view and time of day.
- Graphics settings for ambient occlusion, bloom, clouds, color depth, field of view and an FPS counter.
- Keyboard navigation in every menu. Escape goes back.
- macOS release as a universal app in a `.dmg`.

### Changed

- Menus redesigned: left-aligned text, one accent color, no boxes.
- Swimming: you float with your eyes above the water, dive by looking down, and climb out at the shore.
- Each chunk is now at most 4 entities instead of up to 48, with colliders only near the player.
- Chunks appear at most two per frame while you move, to avoid stutter.

### Fixed

- Menus stayed painted over the world after closing. This also hid the world until the window was resized.
- The player slid down slopes while standing still.
- Walking uphill lost speed and bounced; walking downhill launched the player off the ground.
- The camera sat under the water surface while floating.

## [0.1.1] - 2026-11-16

### Fixed

- macOS release builds run on the `macos-latest` runner. The `macos-14` image is no longer available.

## [0.1.0] - 2026-11-16

### Added

- Procedural islands in an endless ocean, streamed in chunks around the player.
- Translucent sea between the islands.
- First-person movement with sprint.
- Jumping and swimming.
- Title, new game (seed), pause and settings menus, with key rebinding.
- Persistent settings file for graphics, controls and audio.
- Pixelate post-process shader with color banding, embedded in the binary.
- Warm sun with cascaded shadows, soft sky and distance fog.
- CI and tagged release builds for Linux, Windows and macOS.
