# OmarchyGS provider developer kit v1

This local preview kit contains exact Cargo package archives for the public
Provider SDK, provider starter, and provider conformance runner, plus the
published bounded fault inventory and receipt/config schemas.

Providers receive only pairwise game-scoped subjects and scoped grants. They
never receive account or persona identity, reusable device credentials,
platform database access, arbitrary egress, client executable privilege, or
direct client connectivity. This kit grants no registration, activation,
discovery, trust, admission, or publication authority.

Verify `developer-kit-release.json` and every `developer-kit-lock.json` entry
before extracting an archive. Build consumers from a fresh source tree using
Cargo command-line patches to the extracted package roots.
