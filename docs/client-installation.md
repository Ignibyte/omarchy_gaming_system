# Omarchy client installation

Status: native Arch packaging is available for private-alpha testing. Public
package-repository publication, release signing, and automatic updates are not
implemented yet.

## What the package installs

`omarchy-gaming-system-client` is the player-device package. It contains:

- the exact reviewed production QML client under
  `/usr/share/omarchy-gaming-system/qml`;
- `/usr/bin/omarchygs` as the command launcher;
- an Omarchy application-menu entry; and
- non-secret build revision and source-digest provenance.

It depends on Omarchy's `qt6-declarative` package. It does not contain the
Rust community server, PostgreSQL, migrations, Docker, Cargo, test fixtures,
provider code, Game Cartridge publisher code, credentials, or keys.

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
sudo pacman -U ./omarchy-gaming-system-client-0.1.0-1-any.pkg.tar.zst
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
memory only. The EXIT button closes the window without revoking the durable
server-side session, but the raw Bearer is lost with the process; reopening the
client currently requires signing in again. Persistent sign-in waits for a
separately reviewed OS-keyring boundary.

Each owner-operated server is an independent trust and identity domain. This
package currently connects to one manually selected origin at a time and does
not provide saved multi-server profiles, federation, cartridge acquisition,
or a direct cartridge/provider credential path.

## Update and remove

Inspect and install a newer reviewed package with the same `pacman -U` flow.
Pacman replaces the immutable QML payload and keeps the package inventory
coherent; no client database migration or persistent local credential state is
involved in this slice.

Remove the client and its now-unused runtime dependencies with:

```bash
sudo pacman -Rns omarchy-gaming-system-client
```

Removing the player package does not delete accounts, personas, conversations,
matches, or unrevoked device sessions held by an independently operated
OmarchyGS server. Use that server's session/account controls for remote state.
