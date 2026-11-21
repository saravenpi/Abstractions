# Screenshot slots

The home page carousel (`index.html`, section `shots`) shows eight slots, in this order. Each slot is two files in `site/shots/`: a WebP and a JPEG fallback, both 1280x720. They match the pair used for `hero.webp` and `hero.jpg`.

Until a real shot replaces it, every slot holds a crop of `hero.jpg`, so the page is never broken. The captions and alt text below describe the final shots. Check them against the pictures you supply and edit `index.html` if a shot shows something else.

## Input

PNG, 1280x720, from the game probe. Name each file after its slot: `01-home.png` and so on.

## Convert

    sh site/shots.sh <folder-of-pngs>

The script uses ImageMagick (`magick` or `convert`, with WebP support). It writes `shots/<slot>.webp` and `shots/<slot>.jpg` at 1280x720, quality 80, metadata stripped. A slot with no PNG is skipped. A PNG of another size is scaled to fill and cropped from the centre.

After a replacement, bump the `?v=1` on that slot in `index.html`. nginx caches images for one day.

## Slots

| Slot | Files | Caption | Alt text | The shot should show |
| --- | --- | --- | --- | --- |
| 1 | `01-home.webp`, `01-home.jpg` | At home on Kodya | The robot stands at its home base on the planet Kodya, seen from behind in third person, with its small floating companion beside it. | Third person at the home base, robot and companion in frame, base door or pad behind |
| 2 | `02-ship.webp`, `02-ship.jpg` | The ship | The redesigned ship on the ground, seen from outside: one hull, swept wings, twin engines and landing gear. | The ship outside, landed, three-quarter view that shows hull, wings, nacelles and gear |
| 3 | `03-cockpit.webp`, `03-cockpit.jpg` | The cockpit | The cockpit seen from the pilot seat: the stick, the throttle lever and a dashboard of gauges, with the planet through the window. | First person from the seat, stick and throttle visible, dashboard lit |
| 4 | `04-herd.webp`, `04-herd.jpg` | A herd at home | A herd of grazing animals on the grass of the home planet. | A herd of grazers on the grass of the home planet, with grass blades in the foreground |
| 5 | `05-flight.webp`, `05-flight.jpg` | Flight over the planet | The ship flying low over the land and sea of the home planet. | The ship, or the view from it, low over land and sea |
| 6 | `06-orbit.webp`, `06-orbit.jpg` | From orbit | The home planet seen from orbit, with its clouds, seas and the line of dawn. | The planet from orbit with clouds and sea, lit from one side, stars around |
| 7 | `07-map.webp`, `07-map.jpg` | The map | The map of a planet with pins and a course drawn on it. | The map on a planet with the new HUD, at least one pin and a course |
| 8 | `08-together.webp`, `08-together.jpg` | Together | Two players side by side, each a robot with its floating companion. | Two players together, both robots visible, each with a companion |

Keep the HUD readable in slots 3 and 7. Avoid a dark or red-tinted frame in slot 1, it is the first picture visitors see.
