#!/bin/sh
# Converts a folder of 1280x720 PNG shots into the files the site uses.
#
#   sh site/shots.sh <folder-of-pngs> [site-dir]
#
# Reads <slot>.png for each slot listed in SHOTS.md and writes
# <site-dir>/shots/<slot>.webp and <slot>.jpg at 1280x720.
# A slot with no PNG is skipped, so the current file stays.

set -eu

src=${1:-}
site=${2:-$(dirname "$0")}
slots="01-home 02-ship 03-cockpit 04-herd 05-flight 06-orbit 07-map 08-together"

[ -d "$src" ] || { echo "usage: sh shots.sh <folder-of-pngs> [site-dir]" >&2; exit 1; }

if command -v magick >/dev/null 2>&1; then
	im="magick"
elif command -v convert >/dev/null 2>&1; then
	im="convert"
else
	echo "needs ImageMagick (magick or convert) with WebP support." >&2
	exit 1
fi

mkdir -p "$site/shots"
done=0
for slot in $slots; do
	png="$src/$slot.png"
	if [ ! -f "$png" ]; then
		echo "skip   $slot (no $png)"
		continue
	fi
	size=$($im "$png" -format '%wx%h' info:)
	[ "$size" = "1280x720" ] || echo "warn   $slot is $size, resized to 1280x720"
	$im "$png" -resize 1280x720^ -gravity center -extent 1280x720 -strip -quality 80 "$site/shots/$slot.jpg"
	$im "$png" -resize 1280x720^ -gravity center -extent 1280x720 -strip -quality 80 -define webp:method=6 "$site/shots/$slot.webp"
	echo "wrote  $slot.webp $slot.jpg"
	done=$((done + 1))
done
echo "$done of 8 slots written. Bump ?v= on the shots in index.html before deploying."
