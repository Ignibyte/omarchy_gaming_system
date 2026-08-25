# OmarchyGS Game Cartridge SDK v1

This is a deterministic export of the public data-only cartridge contract. A
game repository pins `sdk-lock.json`, uses the exact tool version named there,
and treats the production CLI as the packaging and conformance authority.

The schemas document supported inputs and signed-release records. They do not
authorize publisher QML, JavaScript, native code, URLs, direct network access,
platform credentials, or database access.

SDK and presentation protocol v1 are current. Deprecated versions may build
with a warning; retired versions cannot create a new release. Existing sessions
stay pinned to an exact cartridge and follow a signed catalog lifecycle policy.
