import QtQuick
import QtCore

QtObject {
    id: root

    readonly property int maximumProfiles: 16
    readonly property int maximumSerializedBytes: 16384
    readonly property int maximumCapabilities: 32
    readonly property url settingsLocation: StandardPaths.writableLocation(
                                                StandardPaths.GenericConfigLocation)
                                            + "/omarchygs-server-profiles.ini"
    property var profiles: []
    property bool loaded: false

    property ApiClient _endpointRules: ApiClient {}
    property Settings _settings: Settings {
        location: root.settingsLocation
        category: "omarchygs/server-profiles-v1"
    }

    Component.onCompleted: reload()

    function reload() {
        const encoded = _settings.value("profiles", "[]")
        let accepted = []
        let valid = typeof encoded === "string"
                && _utf8Length(encoded) <= maximumSerializedBytes
        if (valid) {
            try {
                const parsed = JSON.parse(encoded)
                valid = Array.isArray(parsed) && parsed.length <= maximumProfiles
                if (valid) {
                    const origins = ({})
                    const identities = ({})
                    for (let index = 0; index < parsed.length; index++) {
                        if (!_validProfile(parsed[index])
                                || origins[parsed[index].origin] === true
                                || identities[parsed[index].server_id] === true) {
                            valid = false
                            break
                        }
                        origins[parsed[index].origin] = true
                        identities[parsed[index].server_id] = true
                        accepted.push(_copyProfile(parsed[index]))
                    }
                }
            } catch (error) {
                valid = false
            }
        }

        profiles = valid ? accepted : []
        loaded = true
        if (!valid) {
            _settings.setValue("profiles", "[]")
            _settings.sync()
        }
        return valid
    }

    function saveProfile(profile) {
        if (!_validProfile(profile))
            return false
        let originIndex = -1
        let identityIndex = -1
        for (let index = 0; index < profiles.length; index++) {
            if (profiles[index].origin === profile.origin)
                originIndex = index
            if (profiles[index].server_id === profile.server_id)
                identityIndex = index
        }
        if (originIndex !== identityIndex)
            return false
        if (originIndex === -1 && profiles.length >= maximumProfiles)
            return false

        const next = profiles.slice()
        if (originIndex === -1)
            next.push(_copyProfile(profile))
        else
            next[originIndex] = _copyProfile(profile)
        next.sort(function(left, right) {
            return left.origin < right.origin ? -1 : left.origin > right.origin ? 1 : 0
        })
        return _commit(next)
    }

    function removeProfile(index) {
        if (!Number.isInteger(index) || index < 0 || index >= profiles.length)
            return false
        const next = profiles.slice()
        next.splice(index, 1)
        return _commit(next)
    }

    function clearProfiles() {
        return _commit([])
    }

    function profileForOrigin(origin) {
        for (let index = 0; index < profiles.length; index++) {
            if (profiles[index].origin === origin)
                return profiles[index]
        }
        return null
    }

    function serializedProfiles() {
        return String(_settings.value("profiles", "[]"))
    }

    function _commit(next) {
        const encoded = JSON.stringify(next)
        if (_utf8Length(encoded) > maximumSerializedBytes)
            return false
        _settings.setValue("profiles", encoded)
        _settings.sync()
        profiles = next
        return true
    }

    function _validProfile(profile) {
        const keys = ["origin", "server_id", "server_name",
                      "protocol_version", "capabilities"]
        if (profile.operator_custom !== undefined)
            keys.push("operator_custom")
        if (!_endpointRules.exactKeys(profile, keys))
            return false
        const normalized = _endpointRules.normalizeEndpoint(profile.origin)
        if (!normalized.ok || normalized.url !== profile.origin
                || !_validUuid(profile.server_id)
                || !_boundedPublicString(profile.server_name, 64, 1)
                || profile.protocol_version !== 1
                || !Array.isArray(profile.capabilities)
                || profile.capabilities.length > maximumCapabilities
                || (profile.operator_custom !== undefined
                    && !_validOperatorCustom(profile.operator_custom)))
            return false

        let previous = ""
        for (let index = 0; index < profile.capabilities.length; index++) {
            const capability = profile.capabilities[index]
            if (typeof capability !== "string"
                    || !/^[a-z0-9][a-z0-9.-]{0,63}$/.test(capability)
                    || (index > 0 && previous >= capability))
                return false
            previous = capability
        }
        return true
    }

    function _copyProfile(profile) {
        const copied = {
            "origin": profile.origin,
            "server_id": profile.server_id,
            "server_name": profile.server_name,
            "protocol_version": profile.protocol_version,
            "capabilities": profile.capabilities.slice()
        }
        if (profile.operator_custom !== undefined)
            copied.operator_custom = profile.operator_custom
        return copied
    }

    function _validOperatorCustom(value) {
        if (!value || typeof value !== "object"
                || !_endpointRules.exactKeys(value, ["operator_name", "authority_id", "key_id",
                                                    "key_sha256", "public_key"])
                || !_boundedPublicString(value.operator_name, 128, 1)
                || !/^[a-z][a-z0-9._-]{0,95}$/.test(value.authority_id)
                || !/^[a-z][a-z0-9._-]{0,95}$/.test(value.key_id)
                || !/^[0-9a-f]{64}$/.test(value.key_sha256))
            return false
        const key = value.public_key
        return key && typeof key === "object"
                && _endpointRules.exactKeys(key, ["format_version", "algorithm", "key_id",
                                              "authority_id", "verifying_key"])
                && key.format_version === 1 && key.algorithm === "ed25519"
                && key.key_id === value.key_id && key.authority_id === value.authority_id
                && typeof key.verifying_key === "string"
                && /^[A-Za-z0-9_-]{43}$/.test(key.verifying_key)
    }

    function _boundedPublicString(value, maximum, minimum) {
        return typeof value === "string"
                && value.length >= minimum && value.length <= maximum
                && value.trim() === value
                && !/[\u0000-\u001f\u007f-\u009f]/.test(value)
    }

    function _validUuid(value) {
        return typeof value === "string"
                && /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/.test(value)
    }

    function _utf8Length(value) {
        let bytes = 0
        for (let index = 0; index < value.length; index++) {
            const code = value.charCodeAt(index)
            if (code <= 0x7f)
                bytes += 1
            else if (code <= 0x7ff)
                bytes += 2
            else if (code >= 0xd800 && code <= 0xdbff
                     && index + 1 < value.length
                     && value.charCodeAt(index + 1) >= 0xdc00
                     && value.charCodeAt(index + 1) <= 0xdfff) {
                bytes += 4
                index += 1
            } else
                bytes += 3
        }
        return bytes
    }
}
