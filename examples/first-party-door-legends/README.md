# Door Legends — first-party cartridge repository fixture

This directory is source-only and intentionally has no path dependency on the
OmarchyGS repository. The Ticket 017 clean-room harness copies it into a fresh
Git repository, supplies an exported SDK and installed production CLI through
explicit environment variables, then builds a signed release twice.

The cartridge is a small BBS-style lobby screen. Gameplay rules remain compiled
into the authoritative OmarchyGS server; this fixture proves independent
presentation packaging and release provenance only.
