# Better 3D models from code: what one research pass found (2026-11-20)

Question: which methods and tools help an agent that writes meshes in code (no artist, no modelling program) make better low-poly models for the robot, the companion and the ship, keeping the game's look?

## What the sources say

- Render, look, criticise, fix, in a loop. LL3M (arXiv, August 2025) writes Blender code with a team of agents; after a first version, a critic agent renders several images of the asset with the camera distance set from its bounding box, lists visual problems per sub-task, and a coding agent edits the existing code instead of rewriting it; a verification agent checks the fix. Giving the coder the existing code is what keeps the shape and changes only the flawed part. https://arxiv.org/html/2508.08228
- The typical faults of code-written models are known. 3DCodeBench (arXiv, June 2026) tested 12 models writing procedural 3D code: failures mostly come from API mismatches, "while successful renders still suffer from disconnected or floating 3D geometric components", and multi-turn refinement raised executability from 0.702 to 0.974. https://arxiv.org/html/2606.01057
- Low poly: shapes and proportions first, every polygon earns its place. "It's most important to get the shapes and proportions of the characters correct. You want enough polygons to be able to have a clean silhouette, but you also want to make sure that you clean up the polygons that are unnecessary" (Danette Beatty, 80.lv, 2016). She builds proportions from simple primitives first, models eyes as inset cavities, and keeps flat colour areas. https://80.lv/articles/the-soul-of-low-poly-characters
- Two primitives go a long way. A 64 kB demo built every object either from deformed cubes or from surfaces of revolution, got bevelled corners by elongating a pointed octagon, and added detail by extruding faces step by step (with Catmull-Clark subdivision to smooth). https://www.ctrl-alt-test.fr/2023/procedural-3d-mesh-generation-in-a-64kb-intro
- Ships: technology first, silhouette second, few tones. "Defining your universe's technology will shape your design"; in a game with several ships "it's crucial that they have distinctive silhouettes"; keep the number of tones to a minimum because the ship is seen under many lights; put a figure beside it for scale. https://www.creativebloq.com/sci-fi/video-game-spaceship-design-21619331
- A hard-surface recipe exists as a Claude skill for Blender: cube primitive plus Bevel plus Subdivision modifier stack for vehicles and props. https://github.com/roble3/cc-blender-skill/blob/main/plugin/skills/blender-modeling/SKILL.md

## What applies here

Inference: Blender is not installed on the build machine and the game's models are Rust code, so the Blender skill and the Blender MCP are not used; their method is. The loop of LL3M maps onto the probe: a turntable stage renders the model from fixed angles, an agent reads the shots, lists faults, edits the existing mesh code, renders again.

Inference: the modelling vocabulary to add to the mesh builders is small: a loft through cross-sections (the ship has one), a lathe (surface of revolution) for round parts, an extrude and inset on a face, and a chamfer on hard edges so flat-shaded parts catch light on their edges instead of looking like boxes.

Inference: the checks to automate are the ones the benchmark names: no floating or disconnected part (the ship has this test: give it to the robot and the companion), no degenerate triangle, parts sunk into their carrier at joints, faces outward.

## Not found

- No source read on procedural low-poly character rigs in code, or on joint design for mechanical characters. Two candidate pages (a blockout guide and a Blender low-poly guide) were found but not opened.
- No source read on Rust mesh libraries; none is needed and no new crate is allowed without asking.
