# Markers over the world: the rules, their numbers and their sources

Written 2026-11-21 by `markers`. Code: `src/menus/hud/onscreen/` (`rule.rs`, `zones.rs`, `overlap.rs`, `place.rs`, numbers in `tune.rs`), `home::Base::home`.

Owner request: the home marker marks the container house, and a marker stops showing when we are close to what it marks.

Second pass the same day by `markers2`, from the owner playing 0.11: "don't show the map pins on screen when they are too far away, it's useless information" (rule 13), "maybe put back distances in the compass, right under the icon" (rule 14) and "if there is one rendered above the other, show the one under with less opacity" (rules 10 and 15).

## The principle

A marker answers "where is it". Once the thing is large in view it answers that itself, and the marker only covers it. So a marker is whole when its target is far, reduced when the target is near, and gone when the target fills the view. A pin too far to reach soon answers a question the player is not asking, so its marker over the world fades out too. The compass and the minimap keep every mark: the compass is the bearing instrument, the minimap is the plan.

## Where home is

`Base::home()` is 1.5 m above the middle of the roof of the container, the box the operator never leaves (`src/home/container.rs`). The marker over the world, the compass bearing, the minimap mark and the map pin all read it. Before, home was `Base::centre`, a spot on open ground about 17 m from the container. The pad keeps the marker of the ship. `Base::centre` still names the site for what is about the site: the reserved ground, the map reveal, the mending bay.

## The rules

| # | Rule | Numbers | Why | Source |
| --- | --- | --- | --- | --- |
| 1 | Near, a marker is reduced: a small icon at 75 % opacity, no words | Under 10 lengths of its own target: home 60 m (container 6 m), ship 80 m (8 m), a pin or a friend 20 m (2 m) | At ten lengths a thing is about 6 degrees wide and is its own sign. The limit follows the size of the target, not one number for all | Markers grow, shrink or fade with distance in shipped games: [1] [2] [3] [4] |
| 2 | Nearer, it is hidden | Under 4 lengths: home 24 m, ship 32 m, a pin 8 m | The owner's shots show icons at 23 to 48 m drawn beside the thing they mark. Hidden answers the request; reduced is only the step before it | Hide on arrival: [1] [5] [6] |
| 3 | Each limit is a band, and it sticks | A fade runs over the outer quarter (60 to 75 m, 24 to 30 m). Once it has run out, the band moves out by a fifth until the marker is whole again (28.8 to 36 m) | No pop, and no flicker for a player who stands on a limit | [5] fades over a third of the distance; [7] fade out, never switch |
| 4 | The limit follows how the player travels | On foot, swimming, landed: 1. Gliding: 2. Ship in flight: 5 (home is reduced under 300 m, hidden under 120 m) | At flight speed the last hundreds of metres are the approach, flown by eye on the pad and the beacon of the base, which is a real light | Owner brief. No source gives a number |
| 5 | What the player follows is never reduced and hides only on arrival | The course: never. The followed pin (the player's own): hidden under 8 m, back past 9.6 m. The pin itself stays; B removes it within 10 m | The distance counting down is the point of following something. The course ends when the computer clears it | [1] [6] [8] |
| 6 | Words only where they help | Whole markers say their name and distance ("Home 300 m", "Mars 3.2 million km"). Reduced markers say nothing. Units come from `ui::units::distance` | The compass has no names, so the marker and the map are where an icon is named | [3] [9] |
| 7 | The middle of the view dims a marker on its target | 35 % opacity within 5 degrees of the middle, whole past 12 degrees, smooth between. Border markers are not dimmed | The middle of the screen is where the player looks and aims | [10] [7]. No shipped game was found that documents its numbers |
| 8 | Out of view, a marker sits on the border with an arrow | The border is the safe rectangle of rule 9. A target to the side keeps its real height. A target behind keeps its side and sinks to the bottom border, never the top. Straight overhead is the top | The border tells which way to turn. Behind must read the same way every time | [1] [11] [12] |
| 9 | Markers keep out of the rest of the HUD | Icon centres stay 68 px from the sides, 132 px from the top (under the compass and its line of distances, `compass::HEIGHT` 56 px), 176 px from the bottom (above the prompt and the hotbar). They move out of the place strip, the minimap and, in the ship, the instruments. All in interface pixels, so it follows "Interface size" | A marker must never print over an instrument | [10] |
| 10 | Two markers that meet are settled by rank | Course, followed pin, friends, home, ship, other pins (`Marker::rank`). The first keeps its room, the next is reduced, one whose small icon would still overlap is a small icon at 40 % of its opacity, behind, without words. The first slot of the pool is drawn in front (`ZIndex`). At most 6 markers | The same rule as the compass (rule 15); the tracked target must always win | [4] [13] |
| 11 | A friend is never hidden | Far: icon, name and distance in their colour. Under 20 m and in view: the name alone, over their head (1.5 m above the body). On the border: icon, arrow and name | A name tag is how players find each other, and the body is the sign | [14] |
| 12 | Three steps in Options, General | Off, Minimal (course, followed pin, friends), All. Stored as the old `world_markers` switch plus `markers_minimal`, so old files load | Players differ: some want none, some need them | [8] [15] [16] [17] |
| 13 | Far, a pin nobody follows has no marker over the world | On foot and swimming 1 km, gliding 5 km, in the ship (landed or flying) 15 km. Whole under the limit, fades over the next quarter (1 to 1.25 km on foot), gone past it. Once gone it comes back only a fifth nearer: whole again under 833 m, fading from 1042 m. Not picked into the pool past the band, so it takes no slot. Exempt, at any distance: the course, the followed pin, friends, home, the ship | About two minutes of travel: a sprint of 8.5 m/s (`player/walk.rs`), a glide of about 45 m/s, the ship at 140 m/s 400 m over the ground (`ship/cruise.rs`). Past that the pin is noise over the world; the compass, the minimap and the map still show it. What the player follows is the point of following it, and friends, home and the ship are one each | A far limit beside the near one: [1]. Show only what is needed now: [21] [22]. Too many marks: [23]. The numbers are this game's speeds |
| 14 | The compass says how far, not what | Every compass icon has its distance on one line under the bar's rule, short (`ui::units::distance_short`: "340 m", "1.2 km", "3.2 M km"), 16 px, `TEXT_2`, faded with its icon. No name. The bar grows once by that line: 32 + 24 = 56 px (`compass::HEIGHT`), the rule and the middle tick stay under the row of icons | Owner request. Distance is the one reading the compass adds to a bearing; the name is on the marker over the world and on the map. A secondary reading takes the secondary tone and the smallest size | [3] [9] |
| 15 | Two compass icons that meet: the first in front, the other dimmed behind | Icons whose centres are under 32 px apart (`space::BASE`, the 16 px icon with 8 px each side) meet. The one that ranks first (rule 10), then the nearer, is opaque and drawn last, so in front. The other is at 40 % of its opacity (`space::BEHIND`), its distance dimmed with it. A distance that would print over one already said is left out (width: characters x 16 x 0.6 + 8 px). No mark moves off its bearing. Friends past five fold into the nearest one with "+N" | The compass is the bearing instrument: a mark pushed aside points the wrong way. Dimming keeps both marks at their true bearing and says which one matters | Owner request. Same order as [4] [13] |

## What shows where

`context::marker_shown` decides the kinds, then the rules above decide the look.

| Where | Kinds over the world |
| --- | --- |
| On foot, swimming, gliding | Course, the followed pin, other pins within 1 km (5 km gliding), friends, home, ship |
| Ship landed or in the air | Course, the followed pin, other pins within 15 km, friends, home. The ship itself has no marker while seated |
| Orbit, open space, between stars | Course (a body, a star or a galaxy) and friends. No pins. Home and the ship have a place only on the planet the player stands on |
| Warp | The course only |
| Map, menus | Nothing |

Herds have no marker over the world: the animals are their own sign. They stay on the compass.

## Decisions

- Near means reduced, then hidden. Hidden alone would pop the icon away at 60 m while the container is still small. Reduced alone would not answer "do not show them anymore".
- No occlusion test. A marker shows through a hill, as in every game read for this. A ray cast per marker per frame buys little.
- Two icon sizes, 24 and 16 px, no continuous scaling: Bevy builds a font atlas per size.
- No "always show" switch and no scan key. The compass and the minimap already show every mark at any distance. A scan or focus mode [13] [18] [19] is a feature of its own.
- Colour is never the only sign: every kind has its own icon shape [20].
- Far pins fade, they do not pop, and the band sticks the other way round from the near one: the same `rule::fade` run on the ratio of the limit to the distance (`rule::beyond`).
- Home and the ship have no far limit. There is one of each, and they are the way back; the owner's complaint was about pins.
- The compass no longer spreads marks apart. The old rule nudged overlapping icons sideways by up to a mark's room per neighbour, so a crowd of five pointed up to 64 px, about 25 degrees, off its bearing. A small spread plus dimming would still point wrong, only less; dimming alone keeps every bearing true. Two marks at exactly the same bearing show as one: the front one is what matters, and the minimap and the map separate them.
- Known mismatch: the compass keeps its own older limit for the player's other pins (favourites), 2 km (`compass::aim::FAVOURITE_REACH`). In the ship a favourite 2 to 15 km away has a marker over the world and no compass mark. The owner should say whether the compass limit should follow rule 13.

## What the lead should look at

At the base on foot: 300 m "Home 300 m" whole on the container roof; 60 m small icon, no words; 20 m and 5 m nothing. In the ship on the pad: a small house icon on the container, about 40 m away, no words, and no ship marker. In flight at 500 m: whole, "Home 500 m". In flight under 120 m: nothing.

Second pass (rules 13 to 15), with the seed and a few pins: on foot, a pin 1.5 km away has no marker over the world and is on the compass; walking toward it, its marker fades in from about 1 km (833 m if it had faded out). The followed pin shows its name and distance at any distance. In the ship, pins within 15 km show. The compass is 56 px high: icons on the row, the rule under them, the distances on the line under the rule ("340 m", "1.2 km") in the grey of secondary text. Two pins at nearly the same bearing: one opaque in front with its distance, the other faint behind without one, neither pushed aside. Over the world, a pin right on the course marker is a faint small icon behind it.

Known limit: when the way of travel changes (take-off, opening the wingsuit) the near distance changes at once, so a marker inside the new distance steps to its new look without a fade.

## Sources

Read in full:

1. Waypoint marker component, near 30, far 400, arrive 12, docks on the nearer side when behind: https://brunodelorence.com/components/waypoint-marker
2. s&box objective marker, opacity by distance with a floor: https://superdocs.sboxcool.com/examples/recipe/objective_marker
3. Skyrim compass, farther places are smaller: https://www.gamerguides.com/the-elder-scrolls-v-skyrim-anniversary-edition/guide/introduction/guide-information/the-compass
4. No Man's Sky, mission markers have their own look and icons change size to cut clutter (second-hand list of patch notes): http://tvtropes.org/pmwiki/pmwiki.php/AntiFrustrationFeatures/NoMansSky
5. FTB waypoints, beacons fade as you approach and are gone at two thirds of the fade distance: https://github.com/FTBTeam/FTB-Mods-Issues/issues/669
6. Breath of the Wild pins and stamps: https://uxcellence.com/2017/lessons-from-zelda
7. Tobii Clean UI, fade out over time to avoid flicker: https://developer.tobii.com/pc-gaming/design-guidelines/explored-features/clean-ui
8. Xbox Accessibility Guideline 109, options for waypoint markers, The Outer Worlds marker with its distance: https://learn.microsoft.com/en-us/gaming/accessibility/xbox-accessibility-guidelines/109
9. Outer Wilds HUD, one tagged target with name and distance: https://ankitpassi.medium.com/outer-wilds-a-ux-critique-part-2-146abcb9a5a1
10. HUD design principles, the centre of the screen is where the gaze lives: https://gamedesign.gg/articles/hud-design-principles
13. Destiny, markers in the world on demand: https://www.gamedeveloper.com/design/what-to-steal-from-destiny-s-ui
14. Collaborative ping systems, Deep Rock Galactic laser pointer, Apex: https://digitalthrivingplaybook.org/example/overview-of-collaborative-ping-systems/
15. Horizon Zero Dawn, each HUD element always, off or dynamic: https://www.gamespot.com/articles/horizon-zero-dawns-custom-hud-options-revealed-pho/1100-6447620
16. Game Accessibility Guidelines, reminder of objectives, permanent or on request: https://gameaccessibilityguidelines.com/indicate-allow-reminder-of-current-objectives-during-gameplay
17. Game Accessibility Guidelines, resizable interface: https://gameaccessibilityguidelines.com/allow-interfaces-to-be-resized/
20. Game Accessibility Guidelines, no information by colour alone: https://gameaccessibilityguidelines.com/ensure-no-essential-information-is-conveyed-by-a-fixed-colour-alone/
21. Breath of the Wild UI talk, CEDEC 2017, show information only when needed: https://gist.github.com/idbrii/e39fe96279aa1670319bfa521d907399
22. Pacing information in AAA games, lean in and lean out: https://uxdesign.cc/pacing-information-flow-in-aaa-games-ad68f0745627
23. The Witcher 3, a designer on too many question marks: https://www.nintendolife.com/news/2022/07/witcher-3-dev-admits-he-overcrowded-one-map-with-too-many-points-of-interest
24. Map markers and Elden Ring: https://www.thegamer.com/open-world-games-fix-map-markers-elden-ring-assassins-creed/
25. HUD practice, Elden Ring custom markers, diegetic HUDs: https://www.iabdi.com/designblog/2022/6/1/nier-automata-chips

Search result read, page not opened, so unverified:

11. Off-screen indicators on the border: https://code.tutsplus.com/positioning-on-screen-indicators-to-point-to-off-screen-targets--gamedev-6644t
12. Targets behind the camera flip sides when projected: https://discussions.unity.com/t/hud-waypoint-indicator-with-problem/839237 and https://www.youtube.com/watch?v=EKVYfF8oG0s
18. Death Stranding scan marks things on demand: https://gamefaqs.gamespot.com/ps4/184428-death-stranding/faqs/78057/controls
19. No Man's Sky Minimal HUD mode: https://www.reddit.com/r/NoMansSkyTheGame/comments/1xhp51u/forsaken_introducing_update_710_no_mans_sky
26. Horizon Zero Dawn HUD scale 0.7 to 1.2: https://www.rockpapershotgun.com/horizon-zero-dawn-ultrawide
27. Outer Wilds HUD markers for the ship, the scout and marked places: https://outerwildsmods.com/mods/shipenhancements/
28. Elden Ring map review: https://stamen.com/cartographers-play-video-games-a-review-of-the-map-in-elden-ring
29. Destiny interface talk, GDC 2016: https://www.gamedeveloper.com/art/video-building-the-user-interface-for-bungie-s-i-destiny-i-

Dead ends: no developer source was found for Halo or Destiny waypoints fading near the reticle (https://www.halowaypoint.com/news/tech-preview-2-outcomes has nothing on it), nor for a published near-distance threshold in a shipped open-world game. Rules 1, 2, 4 and 7 take their numbers from this game's sizes, not from a source.
