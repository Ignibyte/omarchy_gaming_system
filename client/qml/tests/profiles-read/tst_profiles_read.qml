import QtQuick
import QtTest
import "../.." as App

TestCase {
    name: "ServerProfilesReadProcess"

    Component { id: storeComponent; App.ServerProfiles {} }

    function test_read_two_isolated_profiles_after_process_restart() {
        const store = createTemporaryObject(storeComponent, this)
        verify(store !== null)
        compare(store.profiles.length, 2)
        compare(store.profiles[0].origin, "https://alpha.example.test")
        compare(store.profiles[0].server_name, "Alpha Arcade")
        compare(store.profiles[1].origin, "https://beta.example.test")
        compare(store.profiles[1].server_name, "Beta Arcade")
        compare(Object.keys(store.profiles[0]).sort(),
                ["capabilities", "origin", "protocol_version", "server_id",
                 "server_name"])
        verify(store.clearProfiles())
    }
}
