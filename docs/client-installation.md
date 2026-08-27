# Omarchy client installation

Status: native Arch packaging is available for private-alpha testing. Public
package-repository publication, release signing, and automatic updates are not
implemented yet.

## What the package installs

`omarchy-gaming-system-client` is the player-device package. It contains:

- the exact reviewed production QML client under
  `/usr/share/omarchy-gaming-system/qml`;
- `/usr/bin/omarchygs` as the command launcher;
- `/usr/bin/omarchygs-cartridge-companion` as the loopback acquisition,
  verification, cache, mount, and trusted render-plan service;
- an Omarchy application-menu entry; and
- non-secret build revision and source-digest provenance.

The x86_64 package depends on Omarchy's `qt6-declarative`, `glibc`, and
`gcc-libs` packages. It does not contain the Rust community server, PostgreSQL,
migrations, Docker, Cargo, test fixtures, provider code, Game Cartridge
publisher code, credentials, or keys.

## Build and inspect

Build from a reviewed checkout on Omarchy with the standard Arch package
tooling:

```bash
./scripts/build-client-package.sh
cd target/packages
sha256sum --check omarchy-gaming-system-client-*.pkg.tar.zst.sha256
pacman -Qip omarchy-gaming-system-client-*.pkg.tar.zst
```

The builder does not install the package or modify the system package
database. It validates the exact runtime manifest before invoking `makepkg`,
emits the `.pkg.tar.zst` artifact and a matching `.sha256` sidecar, and embeds
the Git revision, dirty state, and aggregate source digest without embedding a
checkout path or credential. Repeated builds from the same state on the same
Omarchy build host are byte-identical.

For a private-alpha artifact received from someone else, verify the checksum
through the same trusted channel used to obtain the reviewed source. A
SHA-256 sidecar detects mismatched bytes but is not a publisher signature.
Do not treat this local build path as a public package repository or a signed
release channel.

## Install and launch

Install or upgrade one reviewed artifact through pacman:

```bash
sudo pacman -U ./omarchy-gaming-system-client-0.1.0-1-x86_64.pkg.tar.zst
```

Then launch **Omarchy Gaming System** from the application menu or run:

```bash
omarchygs
```

The connection screen accepts a server origin. Remote community servers must
use HTTPS; plain HTTP is accepted only for `localhost`, `127.0.0.1`, or
`[::1]` development origins. A known server may also be selected for one
launch:

```bash
omarchygs --server-url=https://games.example.net
```

The launcher places every application argument after the Qt option terminator,
so a server value cannot become a QML import, plugin, or runner option. The
client validates the server's exact health identity before enabling account
access.

## Credentials and closing

Passwords and MFA factors are submitted through the existing bounded API
client. The raw device-session Bearer and any MFA challenge remain in process
memory only. The launcher creates a random loopback-companion credential for
each run, transfers it through a mode-0600 startup document in a private runtime
directory, then removes that document before launching QML. Every companion
request requires both that credential and the exact random loopback authority;
the companion uses no proxy or redirect for remote acquisition.

The EXIT button closes the window and companion without revoking the durable
server-side session, but the raw Bearer is lost with the process; reopening the
client currently requires signing in again. Persistent sign-in waits for a
separately reviewed OS-keyring boundary.

Each owner-operated server is an independent trust and identity domain. The
client can remember multiple public server profiles but connects and authenticates
to only one selected origin at a time. Profiles do not federate accounts,
social data, gameplay, or credentials, and a cartridge never receives a direct
server/provider credential or network path.

## Configure marketplace trust

Marketplace review must be authenticated independently from the community
server being selected. Obtain the reviewed marketplace Ed25519 public-key JSON
through the marketplace or reviewed client-package channel—not from discovery,
the server catalog, or an acquisition response—and install it as a non-symlink
regular file at either:

```text
${XDG_CONFIG_HOME:-$HOME/.config}/omarchy-gaming-system/marketplace-public.json
/etc/omarchygs/marketplace-public.json
```

The per-user file takes precedence over the system file. A private-alpha tester
may select another absolute reviewed file for one launch:

```bash
OGS_CLIENT_MARKETPLACE_PUBLIC_KEY=/absolute/path/marketplace-public.json \
  omarchygs --server-url=https://games.example.net
```

The launcher passes only that path to the Rust companion and tells trusted QML
whether installation authority is ready; it does not pass the key through a
server request. The companion parses the bounded key once, requires the full
key in every acquisition envelope to match exactly, and binds its SHA-256
fingerprint into each mount. Matching `authority_id` or `key_id` labels with
different key bytes are rejected.

Without a configured key, the normal social/game client still starts and may
browse a server's cartridge catalog, but installation and trusted mount
inventory/removal stay unavailable. An invalid, relative, or symlinked explicit
path fails launch. Changing the configured key also fails closed on mounts made
under the old fingerprint; public enrollment and authenticated rotation are not
implemented yet.

## Cartridge installation and local state

When the selected server truthfully advertises
`games.cartridge-acquisition.v1`, its Games screen can explicitly install or
update one exact catalog release. The Rust companion re-fetches catalog state,
downloads the canonical acquisition envelope from that same origin with the
current device Bearer, independently verifies all server, marketplace,
publisher, lifecycle, SDK, and byte identities, then rechecks catalog admission
before publishing a mount. A redirect, changed revision, changed digest,
invalid signature, substituted marketplace key, incompatible package, or
lifecycle denial leaves the prior mount unchanged.

Immutable cartridge content is shared by digest under
`${XDG_DATA_HOME:-$HOME/.local/share}/omarchy-gaming-system/cartridges/content`.
Read-only public mount records are stored separately under `profiles/` by the
selected server UUID, so the same digest can be reused without sharing one
server's admission with another. The cartridge supplies no filesystem
destination, executable code, raw QML, credential, or network endpoint.

Removal from the Games screen removes only that exact server-profile mount. It
does not delete shared immutable bytes, remote account/game state, achievements,
or the server operator's catalog admission. A mount still does not create or
authorize a game session. When a separately created eligible session pins that
exact release and admission revision, the companion compiles its signed entry
screen from the authoritative REST view. The gameplay screen accepts only the
matching bounded plan and uses platform-owned QML components; declared actions
return to the selected OmarchyGS server. Signal Siege keeps its platform-owned
presenter, while Door Legends proves the mounted portable path. Missing or
mismatched mounts, origins, trust keys, revisions, or lifecycle policy fail
closed. Historical auto-acquisition and multi-screen navigation are not yet
implemented.

## Update and remove

Inspect and install a newer reviewed package with the same `pacman -U` flow.
Pacman replaces the immutable QML and companion payload and keeps the package
inventory coherent. Existing public mount records and immutable cached
cartridge bytes remain in the per-user data directory; no credential is stored
there.

Remove the client and its now-unused runtime dependencies with:

```bash
sudo pacman -Rns omarchy-gaming-system-client
```

Removing the player package does not delete accounts, personas, conversations,
matches, or unrevoked device sessions held by an independently operated
OmarchyGS server. It also leaves the per-user cartridge cache for deliberate
reinstallation or manual cleanup. Use that server's session/account controls
for remote state.
