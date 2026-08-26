import QtQuick
import QtTest
import "../.." as App

TestCase {
    name: "ServerProfilesWriteProcess"

    Component { id: storeComponent; App.ServerProfiles {} }

    function capabilities() {
        return ["accounts.invite-registration.v1", "auth.device-sessions.v1",
                "identity.personas.v1"]
    }

    function test_write_two_public_profiles() {
        const store = createTemporaryObject(storeComponent, this)
        verify(store !== null)
        verify(store.clearProfiles())
        verify(store.saveProfile({
            "origin": "https://alpha.example.test",
            "server_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa",
            "server_name": "Alpha Arcade",
            "protocol_version": 1,
            "capabilities": capabilities()
        }))
        verify(store.saveProfile({
            "origin": "https://beta.example.test",
            "server_id": "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb",
            "server_name": "Beta Arcade",
            "protocol_version": 1,
            "capabilities": capabilities()
        }))
        compare(store.profiles.length, 2)
        const encoded = store.serializedProfiles()
        verify(encoded.indexOf("token") === -1)
        verify(encoded.indexOf("password") === -1)
        verify(encoded.indexOf("selected_persona") === -1)
    }
}
