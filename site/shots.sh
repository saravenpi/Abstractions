#!/bin/sh
# Converts a folder of PNG shots into the files the site uses.
#
#   sh site/shots.sh <folder-of-pngs> [site-dir]
#
# Carousel: <slot>.png for each slot listed below becomes
#   shots/<slot>.webp and shots/<slot>.jpg at 1280x720.
# Hero: hero-NN.png (hero-01.png, hero-02.png, ...) becomes
#   hero/NN-1280.webp, NN-1280.jpg, NN-1920.webp, NN-1920.jpg.
#   A source narrower than 1920 px gets a copy of the 1280 file as the 1920
#   variant, so the srcset never points at a missing file.
# Missing PNGs are skipped. After each conversion the ?v= of that picture in
# index.html (and in hero.js for hero slides) is set from a checksum of the
# new file, so browsers and nginx never serve the old one.

set -eu

src=${1:-}
site=${2:-$(dirname "$0")}
slots="01-home 02-characters 03-ship 04-cockpit 05-herd 06-flight 07-orbit 08-map 09-together"
quality=78

[ -d "$src" ] || { echo "usage: sh shots.sh <folder-of-pngs> [site-dir]" >&2; exit 1; }
[ -f "$site/index.html" ] || { echo "no index.html in $site" >&2; exit 1; }

if command -v magick >/dev/null 2>&1; then
	im="magick"
elif command -v convert >/dev/null 2>&1; then
	im="convert"
else
	echo "needs ImageMagick (magick or convert) with WebP support." >&2
	exit 1
fi

sum() {
	cksum < "$1" | cut -d' ' -f1
}

edit() {
	sed -E "$1" "$2" > "$2.tmp" && mv "$2.tmp" "$2"
}

encode() {
	$im "$1" -resize "${2}x" -strip -interlace Plane -quality "$quality" "$3.jpg"
	$im "$1" -resize "${2}x" -strip -quality "$quality" -define webp:method=6 "$3.webp"
}

mkdir -p "$site/shots" "$site/hero"
count=0

for slot in $slots; do
	png="$src/$slot.png"
	if [ ! -f "$png" ]; then
		echo "skip   $slot"
		continue
	fi
	size=$($im "$png" -format '%wx%h' info:)
	[ "$size" = "1280x720" ] || echo "warn   $slot is $size, scaled and cropped to 1280x720"
	out="$site/shots/$slot"
	$im "$png" -resize 1280x720^ -gravity center -extent 1280x720 -strip -interlace Plane -quality "$quality" "$out.jpg"
	$im "$png" -resize 1280x720^ -gravity center -extent 1280x720 -strip -quality "$quality" -define webp:method=6 "$out.webp"
	v=$(sum "$out.webp")
	edit "s#(shots/$slot\.(webp|jpg))\?v=[^\" ]*#\1?v=$v#g" "$site/index.html"
	echo "wrote  $slot  v=$v"
	count=$((count + 1))
done

for png in "$src"/hero-[0-9][0-9].png; do
	[ -f "$png" ] || continue
	n=$(basename "$png" .png)
	n=${n#hero-}
	width=$($im "$png" -format '%w' info:)
	out="$site/hero/$n"
	encode "$png" 1280 "$out-1280"
	if [ "$width" -ge 1920 ]; then
		encode "$png" 1920 "$out-1920"
	else
		cp "$out-1280.jpg" "$out-1920.jpg"
		cp "$out-1280.webp" "$out-1920.webp"
		echo "note   hero-$n is $width px wide: the 1920 variant is a copy of the 1280 one"
	fi
	v=$(sum "$out-1920.webp")
	edit "s#(hero/$n-[0-9]+\.(webp|jpg))\?v=[^\", ]*#\1?v=$v#g" "$site/index.html"
	edit "s#(base: \"hero/$n\", v: \")[^\"]*#\1$v#" "$site/hero.js"
	if ! grep -q "base: \"hero/$n\"" "$site/hero.js"; then
		echo "TODO   add { base: \"hero/$n\", v: \"$v\", alt: \"...\" } to SLIDES in hero.js"
	fi
	echo "wrote  hero/$n  v=$v"
	count=$((count + 1))
done

echo "$count picture(s) written."
