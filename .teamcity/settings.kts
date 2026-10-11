import jetbrains.buildServer.configs.kotlin.*
import jetbrains.buildServer.configs.kotlin.buildSteps.script
import jetbrains.buildServer.configs.kotlin.triggers.vcs

version = "2025.11"
project { buildType(Nightfall) }
object Nightfall : BuildType({
    name = "Deus Ex Machina"
    vcs { root(DslContext.settingsRoot) }
    steps { script { name = "Rust / PTY verification"; scriptContent = "sh scripts/ci.sh" } }
    triggers { vcs {} }
    requirements { equals("teamcity.agent.jvm.os.name", "Linux") }
    artifactRules = "target/release/opsec-demon"
})
