pipeline {
    agent { label 'linux-rust-python' }
    options { timestamps(); timeout(time: 15, unit: 'MINUTES') }
    stages {
        stage('Deus Ex Machina') {
            steps { sh 'sh scripts/ci.sh' }
        }
    }
    post { success { archiveArtifacts artifacts: 'target/release/opsec-demon', fingerprint: true } }
}
