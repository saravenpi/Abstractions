---
name: model-design
description: Design or improve a 3D model that is written in code in this repo (the robot, the companion, the ship, animals, props, buildings). Use when asked to model, redesign, polish or review the look of a mesh, a character or a vehicle, or when a model has floating parts, gaps at joints or a weak silhouette. Gives the art direction, the render-critique-edit loop with the probe, the mesh vocabulary, the budgets and the tests.
---

# model-design

Models in this game are Rust code that builds meshes. There is no modelling program. Quality comes from a loop: render, look, criticise, edit the existing code, render again. Sources for every rule are in `handoff/specs/models.md`.

Four rules outrank the rest:

1. **Look before and after.** No model change is done until you have read turntable shots of it.
2. **One piece.** No part floats and no gap opens at a joint, in any pose. A test proves it.
3. **Silhouette first.** If the black shape at 8 m is not clear and recognisable, details will not save it.
4. **Edit, do not rewrite.** Change the part that is wrong and keep the rest, the skeleton and the pivots.

## Art direction (owner, 2026-11-20)

- The pixelated render stays (the world is drawn at 360 rows). Spend polygons on silhouette and on forms that read from 2 to 8 m, not on detail smaller than a pixel.
- "Low poly with more polygons": the faceted look is the base style, with more segments and richer, designed shapes. Smooth shading only on small round parts that would shimmer (joint balls, cables, nozzle bells).
- Stylish futuristic designs with a hint of space samurai, never a costume. Original work: mood references are for traits, nothing is copied from a game or an artist.
- The robot keeps a cute face: a large simple face on a screen, readable at 8 m, in a big rounded helmet. Cute face, cool armour: the contrast is the design. No humanoid face, no menacing mask.
- Traits the owner's references share, to use as vocabulary:
  - A slightly oversized rounded head or helmet with a screen visor, on a thin neck.
  - Light armour plates over a dark frame, with one warm accent colour used sparingly (cream or bone plates, dark grey frame, orange or yellow accent is one such scheme; start from the game's existing colours).
  - Big blocks at the ends: chunky forearms and gauntlets, large shin and calf blocks, over thin upper limbs and a narrow waist.
  - Visible mechanics in the gaps: round joint discs at shoulders, elbows, hips, knees and ankles, pistons and cables.
  - Long plates that hang from the hips or the shoulders like a coat or a robe panel, rigid and hinged (the samurai hint), which must clip nothing in any pose.
  - One strong head feature that makes the silhouette: a crest, a fin, a ring or a flat horizontal bar. It is also the antenna.
  - Small graphic markings as geometry or colour blocks: a triangle, a stripe, a row of studs, a numbered panel. Few of them.
  - Flat colour areas with clear breaks. No gradients, no texture noise.
- Shape meaning: round reads friendly, square reads solid, pointed reads fast or dangerous. The robot and the companion are mostly round and square; the ship may be pointed.
- The robot, the companion and the ship are one family: same chamfer sizes, same plate-over-frame logic, same accent colour, same marking style.

## The loop

1. **Turntable.** Add or reuse probe stages that show the model alone on neutral ground in near-white light, from front, three-quarter front, side, three-quarter back, back and top, at 2 m and 8 m, with another model beside it for scale, and in its main poses or states. Running the probe is described in `~/.cache/abstractions-targets/int-logs/LOOK.md` and in `AGENTS.md` ("The loop"). Save the "before" shots first.
2. **Criticise** like an art director and write the list: silhouette, proportions (big, medium and small shapes; a figure is 1.8 m), joints, hands and feet, head and face, the back (mounts for gear), materials and tones, the scale of detail, family resemblance.
3. **Edit** the mesh code with the vocabulary below. One fault at a time.
4. **Render again**, compare with "before", close the item or try again.
5. **Stop** when the list is closed. Report before and after shot paths, triangle counts, what changed and what was kept on purpose.

## Mesh vocabulary

- **Loft**: a closed shape through cross-sections that share their corner points. Use for hulls, limbs, torsos. `src/ship/mesh/loft.rs` has one.
- **Lathe**: a profile turned round an axis. Use for helmets, joints, nozzles, pods.
- **Extrude and inset** a face to add a step, a panel, a vent or a visor recess.
- **Chamfer** the hard edges of built parts so a flat-shaded edge catches light. Pick one or two chamfer sizes for the whole family.
- **Sink** every part into its carrier at a joint or a root (a housing the limb enters), instead of butting two boxes together.
- **Mirror** left and right from one definition; make asymmetry a deliberate single difference.
- Reuse the builders that exist (`Soup`, `Solid`, `FlatMesh`); do not write another copy.

## Budgets

- State the triangle count near and far before you start. Near may be a few times the old count; far stays at the old count or below (`VisibilityRange` or the model's own far level).
- No extra material or draw call per body beyond what exists. Colour comes from the builder's vertex colours or tones.
- No system without a run condition, no allocation per frame.

## Tests (own files)

- One connected piece in the rest pose, and attached at every joint through the extreme poses of each bone, within a stated tolerance (pattern: `src/ship/mesh/tests_attach.rs`, `tests_graph.rs`).
- No degenerate triangle; every part encloses positive volume.
- Drawn solids collide where the game needs it (ship, props).
- What the game reads from the model still holds: the first-person eye on the face, hands on the stick, the throttle and a tool's grip, feet on the pedals and the ground.

## Checklist before you report

```
- [ ] Before and after turntables read, at 2 m and 8 m
- [ ] Silhouette recognisable as a black shape from side, front and three-quarter
- [ ] No floating part, no gap at any joint in any pose (test passes)
- [ ] Face cute and readable at 8 m (robot, companion)
- [ ] Two or three tones plus one accent; markings few
- [ ] Triangle counts stated, far level cheap
- [ ] Skeleton, pivots and bone names unchanged
- [ ] Nothing copied from a game or an artist
```
