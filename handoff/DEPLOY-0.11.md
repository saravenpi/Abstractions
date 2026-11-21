# Deploy the 0.11 server to the public Space server

For the lead. Do the steps in order. Stop at any step marked STOP if the result is not the expected one.

Facts used below:

- Dokploy compose "Space server", composeId `kVuA_xv-8KNXbLO_cmhYz`, raw compose.
- Container `abstractions-space-bjefhv-abstractions-server-1`, volume `abstractions-space-bjefhv_space-data` mounted at `/data`.
- The volume holds `world.toml`, `explored.bin`, `cert.der` and `key.der`, owned by uid 10001.
- The server image is built `FROM scratch` (crates/server/Dockerfile): no shell, so every file operation uses a helper container.
- Public port: UDP 24650. Server key today: `9C4B-499F-B5B5`.
- Compose text: `docs/handoff/space-compose.yml`, image tag pinned to `ghcr.io/saravenpi/abstractions-server:0.11.0`.

Total time: about 15 minutes, with about 1 minute of downtime.

## What the 0.11 server does with the 0.10 data

Read in code, not tried on the live volume.

1. `world.toml` of 0.10 has no `version` line. The server reads a missing version as 0 and `WORLD_VERSION` is 3 (crates/net/src/store/file.rs, `load_or_create` and `replace_old`, commit b8ceca8).
2. It copies `world.toml` to `world.toml.0.10.bak` in the same folder, then writes a new `world.toml` with a new random seed and the name from `ABSTRACTIONS_NAME` ("Space"). It prints: `The world in /data/world.toml is from 0.10 and cannot be opened by 0.11. A new world starts; the old file is kept as /data/world.toml.0.10.bak.`
3. It does not touch `cert.der` or `key.der`. The certificate is loaded by `Identity::load_or_create(&config.dir)` (crates/net/src/trust/identity.rs, called from crates/net/src/io/server/mod.rs:60). With both files present it reads them and makes nothing. It creates a pair only when both are missing, and refuses to start when only one exists. The banner key is the fingerprint of `cert.der`, so it stays `9C4B-499F-B5B5` and no player sees a key warning.
4. 0.11 writes `explored3.bin` and `hunt3.bin`. The old `explored.bin` stays in the volume, unused. Leave it.
5. A test covers the backup and the kept key: `a_world_from_0_10_is_backed_up_and_replaced` in crates/net/src/store/file_tests.rs.
6. The new world is saved at once, so a second start reads version 3 and changes nothing.

Everything on the server from 0.10 is gone for the players: ship, saved place, favourites, explored ground. Their single-player worlds are not touched.

A 0.10 game that tries to join gets `Refused(Outdated)` and its own plain message (the wire format of the hello and of the refusal did not change between 0.10.1 and 0.11, checked with `git diff v0.10.1 HEAD -- crates/net/src/wire/msg.rs`).

## Before you start

On this machine, in the tree that holds the release:

```sh
git log --oneline -1
grep image docs/handoff/space-compose.yml
docker manifest inspect ghcr.io/saravenpi/abstractions-server:0.11.0 >/dev/null && echo image ok
```

Expected: the release commit, `image: ghcr.io/saravenpi/abstractions-server:0.11.0`, `image ok`. STOP if the image is not published yet: the release workflow pushes `:latest` and `:0.11.0` (.github/workflows/release.yml).

Run `scripts/mp-e2e.sh quick` first if you did not: it needs no graphics and takes about 5 minutes.

## 1. Save the current state

```sh
BK=~/backups/space-0.10
mkdir -p "$BK"
dokploy compose one --composeId kVuA_xv-8KNXbLO_cmhYz --json > "$BK/compose-before.json"
docker volume ls | grep space-data
```

Expected: one volume named `abstractions-space-bjefhv_space-data`. STOP if the name differs: use the real name in every command below.

## 2. Back up the volume

The mount is read only, so the helper cannot change anything.

```sh
docker run --rm -v abstractions-space-bjefhv_space-data:/data:ro -v "$BK":/backup alpine \
  sh -c 'cd /data && ls -ln && sha256sum cert.der key.der world.toml && tar czf /backup/space-data.tgz -C /data .' \
  | tee "$BK/before.txt"
tar tzf "$BK/space-data.tgz"
```

Expected: `world.toml`, `explored.bin`, `cert.der`, `key.der` listed, owner `10001`, and the same four names in the archive. STOP if the archive is empty or `cert.der` or `key.der` is missing: the server would then make a new key.

Keep `before.txt`: step 6 compares the two hashes.

## 3. Check the compose text

```sh
cat docs/handoff/space-compose.yml
```

It must read: image `ghcr.io/saravenpi/abstractions-server:0.11.0`, `ABSTRACTIONS_NAME: "Space"`, `ABSTRACTIONS_MAX_PLAYERS: "64"`, port `24650:24650/udp`, volume `space-data:/data`, `read_only: true`, `cap_drop: ALL`. Nothing else changes in this release.

Note for the owner: one IP address may hold 8 connections to the server (crates/net/src/room/conn.rs, `CONNS_PER_IP`). Friends in the same home share one address.

## 4. Update the compose and deploy

```sh
dokploy compose update --composeId kVuA_xv-8KNXbLO_cmhYz --sourceType raw \
  --composeFile "$(cat docs/handoff/space-compose.yml)"
dokploy compose deploy --composeId kVuA_xv-8KNXbLO_cmhYz --title "0.11.0" --description "Server 0.11.0, new world"
```

Dokploy pulls the image and recreates the container. Wait about 40 seconds: the health check has a 30 second start period.

## 5. Read the banner

```sh
docker logs --tail 40 abstractions-space-bjefhv-abstractions-server-1
```

Expected, in this order:

```
The world in /data/world.toml is from 0.10 and cannot be opened by 0.11. A new world starts; the old file is kept as /data/world.toml.0.10.bak.
Abstractions server v0.11.0
World     Space (seed <a new number>)   /data/world.toml
Listening UDP 24650 on all addresses (0.0.0.0)
Players   up to 64 at once
Key       9C4B-499F-B5B5   players see this the first time they join
```

STOP and go to Rollback if the `Key` line differs from `9C4B-499F-B5B5`, if the version is not `v0.11.0`, or if the container restarts in a loop (`docker ps` shows `Restarting`).

## 6. Check the files

```sh
docker run --rm -v abstractions-space-bjefhv_space-data:/data:ro alpine \
  sh -c 'cd /data && ls -ln && sha256sum cert.der key.der' | tee "$BK/after.txt"
grep -E 'cert.der|key.der' "$BK/before.txt" "$BK/after.txt"
```

Expected: `world.toml.0.10.bak` and a new `world.toml`, `explored3.bin`, `hunt3.bin` added, `explored.bin` still there, and the `sha256sum` of `cert.der` and `key.der` identical in `before.txt` and `after.txt`. Files owned by `10001`.

## 7. Check the port from outside and a bot through the public address

From a machine that is not the VPS if you can (phone hotspot, laptop). `<host>` is the public name or address of the server. Use the binaries from your build target (`CARGO_TARGET_DIR/debug`).

```sh
"$CARGO_TARGET_DIR/debug/abstractions-server" ping <host>:24650
```

Expected: one line with `Space`, `0/64 players`, `v0.11.0` and `key 9C4B-499F-B5B5`. A timeout means UDP 24650 does not reach the container: check the Dokploy port mapping and the firewall of the VPS, not the server.

Then a bot through the public address (about 20 seconds):

```sh
"$CARGO_TARGET_DIR/debug/examples/bot" <host>:24650 --count 1 --seconds 20
BOT_PROTOCOL=2 "$CARGO_TARGET_DIR/debug/examples/bot" <host>:24650 --seconds 3
```

Expected for the first: `Bot 1 welcome: no record kept`, then `connected 1/1` lines, then `Done: 1/1 connected at the end`. Expected for the second: `closed: This server needs a newer game`. Then:

```sh
docker logs --tail 8 abstractions-space-bjefhv-abstractions-server-1
```

Expected: `Bot 1 joined (1/64)` and `Bot 1 left (0/64)`. The bot is a test player: it leaves nothing but a record-free entry.

Never use the check script on the public server: its port is 24733, the public one is 24650.

## Rollback

Only if step 5 or 6 failed.

```sh
dokploy compose stop --composeId kVuA_xv-8KNXbLO_cmhYz
docker run --rm -v abstractions-space-bjefhv_space-data:/data -v "$BK":/backup alpine \
  sh -c 'cd /data && rm -f world.toml explored3.bin hunt3.bin && tar xzf /backup/space-data.tgz -C /data && chown -R 10001:10001 /data && ls -ln'
```

Then restore the 0.10.1 compose text from `$BK/compose-before.json` (the `composeFile` field) with `dokploy compose update ... --composeFile`, and `dokploy compose deploy`. The volume is back to the state of step 2, with the same key.

## What to tell the players

Send this after step 7 passes:

> The Space server runs Abstractions 0.11.0. Update your game before you join: the old game cannot enter and says the server needs a newer game.
> The old world is gone. The server started a new world with a new seed, so your ship, your saved places and your favourites on the server start again at the home base. Your own single-player worlds are not touched.
> The server key is the same, `9C4B-499F-B5B5`. If your game says the key changed, do not accept it and tell me.
> One address can hold 8 connections. If several of you play from the same home, tell me.
