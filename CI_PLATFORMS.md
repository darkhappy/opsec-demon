# Deus Ex Machina / the CI council

All Rust adapters invoke `sh scripts/ci.sh`. The runner needs rustup with Rust 1.93.1,
Python 3, Git, and Linux or macOS Unix terminal facilities. Container adapters use
`rust:1.93.1-bookworm` and install Python. The driver checks formatting, Clippy, unit
checks, configuration structure, the release build, and the full PTY boot ceremony.
Set `OPSEC_CI_SOCKETS=1` to include the opt-in local TCP checks on a runner that allows them.

| Platform | File | Activation/setup |
| --- | --- | --- |
| GitHub Actions | `.github/workflows/ci.yml` | Push/PR and manual; Linux/macOS matrix |
| GitLab | `.gitlab-ci.yml` | Enable a Docker-capable runner |
| Bitbucket | `bitbucket-pipelines.yml` | Enable Pipelines in the destination repository |
| TeamCity | `.teamcity/settings.kts` | Import versioned Kotlin settings; provision Linux agent; align DSL version |
| Jenkins | `Jenkinsfile` | Configure `linux-rust-python` agent and a Pipeline job |
| Azure Pipelines | `azure-pipelines.yml` | Connect repository; use Linux container job |
| CircleCI | `.circleci/config.yml` | Connect project and enable workflow |
| Buildkite | `.buildkite/pipeline.yml` | Upload pipeline; provision `linux-rust-python` queue |
| Drone | `.drone.yml` | Activate repository with Docker runner |
| Woodpecker | `.woodpecker.yml` | Activate repository with container backend |
| Travis | `.travis.yml` | Connect Linux project |
| Cirrus | `.cirrus.yml` | Enable repository integration |
| Semaphore | `.semaphore/semaphore.yml` | Create project with compatible Linux image |
| AWS CodeBuild | `buildspec.yml` | Configure Linux image providing rustup and Python |
| Google Cloud Build | `cloudbuild.yaml` | Submit build or configure a trigger |
| SourceHut | `.build.yml` | Supply your actual repository URL as a source at build submission, cloned as `opsec-demon` |
| Codefresh | `codefresh.yml` | Configure Git integration and pipeline |
| Concourse | `concourse.yml` | Set `repository_url` using `fly` and activate pipeline |
| Gitea Actions | `.gitea/workflows/ci.yml` | Enable Actions; provision matching runner labels |
| Forgejo Actions | `.forgejo/workflows/ci.yml` | Enable Actions; provision `docker` runner |

The optional Roblox workflow is manual: `.github/workflows/roblox.yml`. It uses Rokit
1.2.0 to install the pinned Roblox annex and runs `sh scripts/roblox-check.sh`.
These files do not create hosted accounts, register runners, publish artifacts remotely,
or configure deployments. Native hosted execution still needs the chosen service.

Format references: [Bitbucket](https://support.atlassian.com/bitbucket-cloud/docs/bitbucket-pipelines-configuration-reference/),
[GitLab](https://docs.gitlab.com/ci/yaml/), [TeamCity](https://www.jetbrains.com/help/teamcity/kotlin-dsl.html),
[CircleCI](https://circleci.com/docs/reference/configuration-reference/),
[Jenkins](https://www.jenkins.io/doc/book/pipeline/syntax/),
[Woodpecker](https://woodpecker-ci.org/docs/usage/workflow-syntax),
[CodeBuild](https://docs.aws.amazon.com/codebuild/latest/userguide/build-spec-ref.html),
[Cloud Build](https://docs.cloud.google.com/build/docs/build-config-file-schema).
