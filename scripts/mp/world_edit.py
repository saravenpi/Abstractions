"""Edits the record of one player in a server's world.toml, for the real-process checks.

Usage: world_edit.py WORLD_TOML (--key HEX | --name NAME) FIELD MODE
  place nan          the height of the stored place is not a number
  place air:METRES   the stored place is that many metres above the sea, on foot
  ship inside        the stored ship is seated, at half the distance from the centre of its planet
The server must be stopped: it writes the file when it stops.
"""

import json
import re
import sys
import tomllib


def member_key(data, key, name):
    players = data.get("players", {})
    if key and key in players:
        return key
    for hexkey, member in players.items():
        if name and member.get("name") == name:
            return hexkey
    sys.exit(f"no such player in the world file: {key or name}")


def edited(field, mode, old):
    if field == "place" and mode == "nan":
        return re.sub(r"^altitude = .*$", "altitude = nan", old, count=1, flags=re.M)
    if field == "place" and mode.startswith("air:"):
        metres = float(mode[4:])
        return re.sub(r"^altitude = .*$", f"altitude = {metres}", old, count=1, flags=re.M)
    if field == "ship" and mode == "inside":
        return inside(old)
    sys.exit(f"unknown edit: {field} {mode}")


def inside(old):
    if "Surface" not in old:
        sys.exit("the stored ship is not on a surface: run the game phase first")
    found = re.search(r"^pos = \[(.*)\]$", old, flags=re.M)
    if not found:
        sys.exit("the stored ship has no position")
    half = [float(part) / 2.0 for part in found.group(1).split(",")]
    text = re.sub(r"^pos = \[.*\]$", "pos = " + str(half), old, count=1, flags=re.M)
    return re.sub(r"^seated = .*$", "seated = true", text, count=1, flags=re.M)


def renderings(old):
    escaped = old.replace("\\", "\\\\").replace('"', '\\"')
    return [
        "'''\n" + old + "'''",
        "'''" + old + "'''",
        '"""\n' + escaped + '"""',
        '"""' + escaped + '"""',
        json.dumps(old),
        "'" + old + "'",
    ]


def replace(text, hexkey, field, old, new):
    header = text.find(f"[players.{hexkey}]")
    if header < 0:
        header = text.find(f'[players."{hexkey}"]')
    for form in renderings(old):
        at = text.find(f"{field} = {form}", header)
        if at >= 0:
            end = at + len(f"{field} = {form}")
            return text[:at] + f"{field} = " + json.dumps(new) + text[end:]
    sys.exit(f"could not find the {field} text of the player in the file")


def main():
    args = sys.argv[1:]
    path, who, field, mode = args[0], args[2], args[3], args[4]
    text = open(path, encoding="utf-8").read()
    data = tomllib.loads(text)
    hexkey = member_key(data, who if args[1] == "--key" else None, who if args[1] == "--name" else None)
    old = data["players"][hexkey][field]
    new = edited(field, mode, old)
    out = replace(text, hexkey, field, old, new)
    if tomllib.loads(out)["players"][hexkey][field] != new:
        sys.exit("the edited file does not read back as planned")
    open(path, "w", encoding="utf-8").write(out)
    print(f"edited {field} of {hexkey[:8]}: {mode}")


main()
