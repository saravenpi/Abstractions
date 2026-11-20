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

## Second pass: character design, and skills for Claude (2026-11-20 night)

### Character design

- Silhouette first. Iconic characters stay recognisable as a pure black shape; detail must be "organized and contained within a recognizable silhouette". A character is a collection of shapes: the silhouette is the biggest, then secondary shapes, then the smallest, the details. Shapes carry meaning ("spiky thing dangerous", "round thing safe"). The process is iteration from big to small: many silhouettes first, then the interior shapes, details last (Jon Neimeister, Hi-Rez Studios). https://visualartspassage.com/blog/principles-of-character-design
- Machines with character, from the designer of the machine lifeforms of NieR:Automata: the brief was cute enough to appeal, a little unbalanced to show character, rough and retro, and modular, "assembled from similar units to look like a cohesive force"; his own goals were simple symbolic features "that even a kid could draw" and a silhouette easy to understand; he reached the final design by subtraction, "removing details until only the bare necessities were left"; one shared detail (a connector cover where parts attach) ties every unit together; thinking about how they move and are put together is part of the design. https://www.platinumgames.com/official-blog/article/9863
- Believable robots: humanoid robots rest on human anatomy; understand what each detail is for ("if you creating a hatch, you should understand what this hatch should hide"); before detailing a moving part, block it out and animate it; vary the kinds of joints. https://80.lv/articles/how-to-model-a-robot

### Skills that exist for Claude and 3D

- A curated list counts 213 open-source MCP servers and skills for agents working in 3D (Blender, Unity, Godot, Unreal, CAD, Three.js, model generation), each security-graded, and ran one brief through 25 of them on 2026-11-08. https://github.com/zhuyansen/awesome-3d-skills/blob/main/README.md
- Snyk lists eight Claude skills for 3D, game development and shaders, among them a "3D Modeling specialist" skill with three reference files (creation patterns, known failures, validation rules) and strong rules on topology, and a Blender procedural Python skill; it also warns that its research found prompt injection in 36 percent of skills tested, so every `SKILL.md` and script must be read before install. https://snyk.io/articles/top-claude-skills-3d-modeling-game-dev-shader-programming/
- Inference: almost all of these drive Blender, Unity, Unreal or Three.js. None targets meshes written in Rust for Bevy. Installing a third-party skill is a supply-chain decision for the owner (read it first), and none is needed tonight. What would help this repo is its own small skill: the turntable loop, the mesh vocabulary (loft, lathe, extrude, inset, chamfer), the attach test and the design rules of this file, so every later model follows them.
- Not found: a skill or a source specific to character design for code-written models. `https://www.diva-portal.org/smash/get/diva2:1129926/FULLTEXT02.pdf` (a thesis on guidelines for game character design) was fetched but not read.
