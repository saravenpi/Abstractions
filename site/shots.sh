#!/bin/sh
# Converts a folder of PNG shots into the files the site uses.
#
#   sh site/shots.sh <folder-of-pngs> [site-dir]
#
# Carousel: <slot>.png for each slot listed below becomes
#   shots/<slot>.webp and shots/<slot>.jpg at 1280x720.
# Hero: hero-NN.png (hero-01.png, hero-02.png, ...) becomes
#   hero/NN-1280.webp, NN-1280.jpg, NN-1920.webp, NN-1920.jpg.
#   An optional hero-NN.txt beside the PNG holds the alt text (one sentence).
#   The script then rewrites the SLIDES array in hero.js and the first slide in
#   index.html between their hero marker comments, from every hero/NN-1920.webp
#   present, so nothing is edited by hand.
#   A source narrower than 1920 px gets a copy of the 1280 file as the 1920
#   variant, so the srcset never points at a missing file.
# Missing PNGs are skipped. The ?v= of every picture is set from a checksum of
# its file, so browsers and nginx never serve a replaced picture from cache.

set -eu

src=${1:-}
site=${2:-$(dirname "$0")}
slots="01-home 02-ship 03-cockpit 04-flight 05-orbit 06-herd 07-map 08-together"
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
	[ -f "${png%.png}.txt" ] && cp "${png%.png}.txt" "$out.txt"
	echo "wrote  hero/$n"
	count=$((count + 1))
done

inject() {
	awk -v s="$2" -v e="$3" -v blk="$4" '
		index($0, s) { print; while ((getline l < blk) > 0) print l; skip = 1; next }
		index($0, e) { skip = 0 }
		!skip { print }
	' "$1" > "$1.tmp" && mv "$1.tmp" "$1"
}

alt_of() {
	if [ -f "$site/hero/$1.txt" ]; then
		tr '\n' ' ' < "$site/hero/$1.txt" | sed -E 's/ +$//'
	else
		echo "warn   hero/$1 has no alt text: add hero-$1.txt beside the PNG" >&2
	fi
}

ids=$(ls "$site/hero" | sed -n -E 's/^([0-9]{2})-1920\.webp$/\1/p' | sort)
total=$(echo "$ids" | grep -c . || true)
first=$(echo "$ids" | head -n 1)
tmp=$(mktemp -d)
: > "$tmp/head"
: > "$tmp/body"
: > "$tmp/js"
for n in $ids; do
	v=$(sum "$site/hero/$n-1920.webp")
	alt=$(alt_of "$n")
	printf '    { base: "hero/%s", v: "%s", alt: "%s" },\n' "$n" "$v" "$(printf '%s' "$alt" | sed 's/\\/\\\\/g; s/"/\\"/g')" >> "$tmp/js"
	if [ "$n" = "$first" ]; then
		size=$($im "$site/hero/$n-1280.jpg" -format '%w %h' info:)
		w=${size% *}
		h=${size#* }
		hal=$(printf '%s' "$alt" | sed 's/&/\&amp;/g; s/</\&lt;/g; s/"/\&quot;/g')
		sw="hero/$n-1280.webp?v=$v 1280w, hero/$n-1920.webp?v=$v 1920w"
		sj="hero/$n-1280.jpg?v=$v 1280w, hero/$n-1920.jpg?v=$v 1920w"
		go=""
		[ "$total" -ge 2 ] && go=" go"
		printf '<link rel="preload" as="image" type="image/webp" imagesrcset="%s" imagesizes="100vw" fetchpriority="high">\n' "$sw" > "$tmp/head"
		printf '      <figure class="layer on%s">\n        <picture><source type="image/webp" srcset="%s" sizes="100vw"><img src="hero/%s-1280.jpg?v=%s" srcset="%s" sizes="100vw" width="%s" height="%s" alt="%s" fetchpriority="high" decoding="async"></picture>\n      </figure>\n' "$go" "$sw" "$n" "$v" "$sj" "$w" "$h" "$hal" > "$tmp/body"
	fi
done
inject "$site/index.html" "<!-- hero-head:start -->" "<!-- hero-head:end -->" "$tmp/head"
inject "$site/index.html" "<!-- hero:start -->" "<!-- hero:end -->" "$tmp/body"
inject "$site/hero.js" "// hero:start" "// hero:end" "$tmp/js"
rm -rf "$tmp"
echo "hero: $total slide(s) in index.html and hero.js."

echo "$count picture(s) written."
