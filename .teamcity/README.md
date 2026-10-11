# TeamCity relay

Import versioned settings from this repository and use the Kotlin DSL in `settings.kts`.
It targets the 2025.11 DSL; align the version with your server when importing.
The Linux build agent needs rustup with Rust 1.93.1, Python 3, Git, and a POSIX terminal.
Configure the settings VCS root in TeamCity. No server URL or credentials are committed.

[Official Kotlin DSL reference](https://www.jetbrains.com/help/teamcity/kotlin-dsl.html).
