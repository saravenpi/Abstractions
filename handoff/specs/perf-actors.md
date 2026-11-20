# Perf audit: actors, UI, net (read-only, no timing)
1. Leaves cloned every frame while planet map open: src/map/raster/tiles.rs:100-118 (start: explored.leaves(body).cloned(), asked Vec each frame); want (line 76) mutates Tiles every frame so tiles_view::show always runs (tiles.rs:57 ready(), tiles_view.rs:60 held). Fix: clone inside `running < JOBS && !current` branch / early return; Arc<Leaves> per version; dirty flag.
2. Tile worker O(total leaves) per tile: src/map/raster/tile_paint.rs:14 (lit). Fix: filter leaves by tile face and bounds, or leaf-to-pixel list once per version.
3. Shoreline probe ~97 Terrain::height per frame: src/audio/sense.rs (shore), src/audio/shore.rs:11-24 (nearest). Fix: skip when height >= CEILING (45 m); cache, recompute after listener moves >3 m or 5-10 Hz.
4. Save/snapshot hitches: src/saves/capture.rs:70-76 rebuilds leaves.runs() per body per save (crates/net/src/leaves/convert.rs:8-20 two Vec copies); src/saves/snapshot.rs capture().cloned(); server crates/net/src/room/mod.rs snapshot on tick thread clones World + explored_list. Fix: rebuild runs only when Explored::version changed; per-body dirty flag.
5. HUD static bars: src/menus/hud/snow.rs:81-94 sets node.top on 8 bars at 20 Hz even with whole head. Fix: early return when head whole and cover clear; write only if differs.
6. Per-frame Vec<Well>: src/ship/sky.rs:~120-137 Skies::refresh from sim.rs survey. Fix: clear+extend; rebuild only on sector change; skip while parked and on foot.
7. Bones rewritten each frame incl. hidden bodies: src/avatar/animate.rs:~150. Fix: set_if_neq, skip Hidden; globals() recomputed several times per body (fx::wings per panel, marks.rs, spark::drizzle).
8. Marker labels/distance strings per frame: src/menus/hud/compass/herds.rs, beacons, waypoint::fill, favourite::fill, compass/show.rs.
9. Lamp lights written before change checks: src/avatar/fx.rs:~157 (face), src/companion/shine.rs.
10. Camera boom casts: src/view/rig.rs:137-138, view/occlude.rs (guess).
