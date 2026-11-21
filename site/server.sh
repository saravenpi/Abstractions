#!/bin/sh
# Installs the Abstractions server on Linux.
#
#   curl -fsSL https://abstractions.facile.studio/server.sh | sh
#   curl -fsSL https://abstractions.facile.studio/server.sh | sudo sh -s -- --service
#   curl -fsSL https://abstractions.facile.studio/server.sh | sh -s -- --uninstall
#
# Environment:
#   ABSTRACTIONS_VERSION=vX.Y.Z  install this version instead of the latest one
#   ABSTRACTIONS_ARCH=aarch64    use this machine type instead of `uname -m`
#   ABSTRACTIONS_DRY_RUN=1       print what would happen, change nothing

set -eu

REPO="saravenpi/Abstractions"
API="https://api.github.com/repos/$REPO/releases/latest"
DOWNLOADS="https://github.com/$REPO/releases/download"
NAME="abstractions-server"
UNIT="/etc/systemd/system/$NAME.service"
SERVICE_USER="abstractions"
SERVICE_DATA="/var/lib/abstractions"
SCRIPT="https://abstractions.facile.studio/server.sh"

say() {
	printf '%s\n' "$*"
}

fail() {
	printf 'Error: %s\n' "$*" >&2
	exit 1
}

dry() {
	[ "${ABSTRACTIONS_DRY_RUN:-0}" = "1" ]
}

is_root() {
	[ "$(id -u)" = "0" ]
}

has() {
	command -v "$1" >/dev/null 2>&1
}

has_systemd() {
	has systemctl && [ -d /run/systemd/system ]
}

check_os() {
	os=$(uname -s)
	[ "$os" = "Linux" ] && return 0
	say "The server is built for Linux only. This machine runs $os."
	say "Use Docker instead: docker run -d -p 24650:24650/udp -v abstractions:/data ghcr.io/saravenpi/abstractions-server"
	say "Guide: https://abstractions.facile.studio/server/"
	exit 1
}

detect_arch() {
	machine=${ABSTRACTIONS_ARCH:-$(uname -m)}
	case "$machine" in
	x86_64 | amd64) echo x86_64 ;;
	aarch64 | arm64) echo aarch64 ;;
	*) fail "no server build for the machine type '$machine'. Builds exist for x86_64 and aarch64 (arm64)." ;;
	esac
}

fetch() {
	if has curl; then
		curl -fsSL -o "$2" "$1" || fail "could not download $1"
	elif has wget; then
		wget -q -O "$2" "$1" || fail "could not download $1"
	else
		fail "this script needs curl or wget."
	fi
}

latest_tag() {
	if dry; then
		echo "<latest>"
		return 0
	fi
	fetch "$API" "$tmp/release.json"
	sed -n 's/.*"tag_name"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$tmp/release.json" | head -n 1
}

check_tag() {
	[ "$1" = "<latest>" ] && return 0
	printf '%s\n' "$1" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$' || fail "'$1' is not a version like v0.11.0."
}

sha256() {
	if has sha256sum; then
		sha256sum "$1" | cut -d ' ' -f 1
	elif has shasum; then
		shasum -a 256 "$1" | cut -d ' ' -f 1
	else
		fail "this script needs sha256sum or shasum to check the download."
	fi
}

expected_sum() {
	awk -v a="$2" -v b="$3" '($2 == a || $2 == "*" a || $2 == b || $2 == "*" b) { print $1; exit }' "$1"
}

install_dir() {
	if is_root; then
		echo /usr/local/bin
	else
		echo "$HOME/.local/bin"
	fi
}

download() {
	file="$NAME-$tag-linux-$arch"
	stable="$NAME-linux-$arch"
	sums="SHA256SUMS-server-$tag.txt"
	say "Downloading $NAME $tag for Linux $arch."
	fetch "$DOWNLOADS/$tag/$file" "$tmp/$file"
	fetch "$DOWNLOADS/$tag/$sums" "$tmp/$sums"
	want=$(expected_sum "$tmp/$sums" "$file" "$stable")
	[ -n "$want" ] || fail "$sums has no line for $file."
	got=$(sha256 "$tmp/$file")
	[ "$got" = "$want" ] || fail "the download does not match its checksum. Nothing was installed."
	say "Checksum OK."
}

place() {
	mkdir -p "$dir"
	stage="$dir/.$NAME.new.$$"
	cp "$tmp/$file" "$stage"
	chmod 755 "$stage"
	mv -f "$stage" "$dir/$NAME"
	say "Installed $dir/$NAME."
}

warn_path() {
	case ":$PATH:" in
	*":$dir:"*) ;;
	*)
		say "Note: $dir is not on your PATH."
		say "Add it: echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.profile, then open a new terminal."
		;;
	esac
}

need_service_rights() {
	is_root || fail "--service needs root. Run: curl -fsSL $SCRIPT | sudo sh -s -- --service"
	has_systemd || fail "--service needs systemd, and this machine does not run it."
}

make_user() {
	id "$SERVICE_USER" >/dev/null 2>&1 && return 0
	shell=/usr/sbin/nologin
	[ -x "$shell" ] || shell=/bin/false
	useradd --system --home-dir "$SERVICE_DATA" --no-create-home --shell "$shell" "$SERVICE_USER"
	say "Created the system user $SERVICE_USER."
}

write_unit() {
	cat >"$UNIT" <<EOF
[Unit]
Description=Abstractions server
Wants=network-online.target
After=network-online.target

[Service]
User=$SERVICE_USER
Group=$SERVICE_USER
WorkingDirectory=$SERVICE_DATA
ExecStart=/usr/local/bin/$NAME --data $SERVICE_DATA
Restart=on-failure
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$SERVICE_DATA
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF
}

service() {
	make_user
	mkdir -p "$SERVICE_DATA"
	chown "$SERVICE_USER:$SERVICE_USER" "$SERVICE_DATA"
	chmod 750 "$SERVICE_DATA"
	write_unit
	systemctl daemon-reload
	systemctl enable "$NAME" >/dev/null 2>&1
	systemctl restart "$NAME"
	say "The server runs as a service and starts with the machine."
	say "Data folder: $SERVICE_DATA"
	say "Status: systemctl status $NAME"
	say "Logs: journalctl -u $NAME -f"
}

uninstall() {
	dir=$(install_dir)
	if dry; then
		say "Would remove $dir/$NAME"
		is_root && say "Would remove $UNIT if it exists"
		return 0
	fi
	if [ -f "$UNIT" ]; then
		is_root || fail "the service is installed. Run: curl -fsSL $SCRIPT | sudo sh -s -- --uninstall"
		systemctl disable --now "$NAME" >/dev/null 2>&1 || true
		rm -f "$UNIT"
		systemctl daemon-reload
		say "Removed the service."
		say "The world is still in $SERVICE_DATA. Delete that folder yourself if you want it gone."
	fi
	if [ -e "$dir/$NAME" ]; then
		rm -f "$dir/$NAME"
		say "Removed $dir/$NAME."
	else
		say "No server found in $dir."
	fi
	say "Worlds were not touched."
	say "A server started by hand keeps its world in ./data, in the folder you started it from."
}

dry_run() {
	file="$NAME-$tag-linux-$arch"
	say "Dry run. Nothing is downloaded or changed."
	say "Machine:  Linux $arch"
	[ "$tag" = "<latest>" ] && say "Version:  latest, read from $API"
	[ "$tag" = "<latest>" ] || say "Version:  $tag"
	say "Binary:   $DOWNLOADS/$tag/$file"
	say "Checksum: $DOWNLOADS/$tag/SHA256SUMS-server-$tag.txt"
	say "Install:  $dir/$NAME"
	if [ "$mode" = "service" ]; then
		say "Service:  $UNIT, user $SERVICE_USER, data $SERVICE_DATA"
	fi
}

main() {
	mode=install
	for arg in "$@"; do
		case "$arg" in
		--service) mode=service ;;
		--uninstall) mode=uninstall ;;
		-h | --help)
			say "Usage: server.sh [--service | --uninstall]"
			say "Guide: https://abstractions.facile.studio/server/"
			return 0
			;;
		*) fail "unknown option '$arg'. Use --service or --uninstall." ;;
		esac
	done
	check_os
	[ "$mode" = "uninstall" ] && {
		uninstall
		return 0
	}
	arch=$(detect_arch)
	dry || [ "$mode" != "service" ] || need_service_rights
	tmp=$(mktemp -d)
	trap 'rm -rf "$tmp"' EXIT INT TERM
	tag=${ABSTRACTIONS_VERSION:-$(latest_tag)}
	[ -n "$tag" ] || fail "could not read the latest version from GitHub."
	check_tag "$tag"
	dir=$(install_dir)
	[ "$mode" = "service" ] && dir=/usr/local/bin
	if dry; then
		dry_run
		return 0
	fi
	download
	place
	if [ "$mode" = "service" ]; then
		service
		return 0
	fi
	warn_path
	say "Start it: $NAME"
	say "It keeps the world in ./data, in the folder you start it from."
}

main "$@"
