# Pictures on the home page

There are two sets: the hero slideshow at the top, and the carousel under it. Both are made by `shots.sh` from PNG files.

## Convert

    sh site/shots.sh <folder-of-pngs>

Needs ImageMagick (`magick` or `convert`, with WebP). Name the PNGs after the slots below. A missing PNG is skipped. The script also sets the `?v=` of each picture in `index.html` and `hero.js` from a checksum of the new file, so a replaced picture is never served from cache. Run it again after any new shooting run and commit the result. No other step is needed for the carousel.

For a new hero picture, add one line to `SLIDES` in `hero.js` with its base name and its alt text. The script prints the line to add.

## Hero slideshow

Files: `hero/NN-1280.webp`, `hero/NN-1280.jpg`, `hero/NN-1920.webp`, `hero/NN-1920.jpg`. Source: `hero-NN.png`, 1920x1080 if you can, 1280x720 otherwise. The page picks the variant by screen width. With a 1280 source, the 1920 files are copies of the 1280 ones, so nothing points at a missing file.

Encoding: quality 78, metadata stripped, JPEG progressive, WebP method 6. Current weights: slide 1 is 36 KB (WebP) and 71 KB (JPEG) at 1280, 76 KB and 165 KB at 1920. A visitor downloads one variant per slide, in WebP almost everywhere, and only the slides actually shown. Expect 40 to 100 KB per slide, so a full show of 16 costs about 1 MB in a visit that lets it run to the end. All files of 16 slides on disk: about 2 to 4 MB.

The title and the text sit on the left and in the lower half. Keep the left third calm: sky, ground or dark space, no HUD, no ship nose, no bright cloud edge there. Put the subject in the right two thirds. Avoid a frame that is mostly white. The page darkens every slide, and a pure white slide still keeps the body text above 5:1 and the title above 2.7:1.

| Slot | Source | Should show |
| --- | --- | --- |
| 1 | `hero-01.png` | Now: the old hero picture. Replace with the robot on the home plain at dawn, seen from behind, subject right of centre |
| 2 | `hero-02.png` | The companion close up, floating, with the home base behind it |
| 3 | `hero-03.png` | The ship on its pad, three-quarter view, low sun |
| 4 | `hero-04.png` | The ship lifting off, with dust |
| 5 | `hero-05.png` | The ship low over a coast |
| 6 | `hero-06.png` | The robot in the wingsuit over a valley |
| 7 | `hero-07.png` | A herd on the grass, far away, in warm light |
| 8 | `hero-08.png` | Forest and tall grass, backlit |
| 9 | `hero-09.png` | Sea and shore, a long horizon |
| 10 | `hero-10.png` | Clouds seen from above |
| 11 | `hero-11.png` | The home planet from orbit |
| 12 | `hero-12.png` | A moon and its planet in the same sky |
| 13 | `hero-13.png` | A galaxy from outside |
| 14 to 16 | `hero-14.png` to `hero-16.png` | Free: the best of the postcards run |

No HUD in any hero picture. Alt text says what is really in the picture, in one sentence.

## Carousel

Files: `shots/<slot>.webp` and `shots/<slot>.jpg`, 1280x720, quality 78. Source: `<slot>.png`, 1280x720, HUD allowed. Current weight: 379 KB of WebP and 584 KB of JPEG for six slots.

The page lists the slots in this order. The first six are live. The last three are pending and are not in `index.html` yet. To add one, convert its PNG, then copy a `figure` block in the `shots` section, with the same file base name, a caption and an alt text.

| Slot | Name | Shows | State |
| --- | --- | --- | --- |
| 1 | `01-home` | Third person at the home base | Live. Taken before the robot head and chest redesign. Replace |
| 2 | `02-characters` | The robot and the companion, three-quarter front | Live. Taken before the head, chest and crest redesign. Replace |
| 3 | `03-ship` | The ship outside | Live |
| 4 | `04-cockpit` | The cockpit from the seat | Live |
| 5 | `05-herd` | A herd on the home planet | Pending |
| 6 | `06-flight` | Flight over the planet | Live |
| 7 | `07-orbit` | The planet from orbit | Live. It shows the ship burning in orbit. Replace with a shot where the planet fills more of the frame if you can |
| 8 | `08-map` | The map with pins and a course | Pending |
| 9 | `09-together` | Two players, each with a companion | Pending |

The compass at the top lost its labels after the live shots were taken, so slots 1 to 4 and 6 show the old compass. Replace them after the next shooting run.
