"""Measure local warm CLI launch-to-pwd timing; no universal performance claims."""
from pathlib import Path
import json
import platform
import shutil
import statistics
import subprocess
import time
import tomllib

ROOT=Path(__file__).resolve().parents[1]
binary=ROOT/'target/release/opsec-demon'
assert binary.exists(), 'Build release first'
samples=[]
for name,program in [('OPSEC DEMON',str(binary)),('Bash',shutil.which('bash')),('fish',shutil.which('fish')),('Zsh',shutil.which('zsh')),('Nushell',shutil.which('nu'))]:
    if not program:
        samples.append({'name':name,'available':False})
        continue
    command=[program,'-c','pwd']
    try:
        subprocess.run(command,cwd=ROOT,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True,timeout=5)
        times=[]
        for _ in range(25):
            start=time.perf_counter_ns()
            subprocess.run(command,cwd=ROOT,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,check=True,timeout=5)
            times.append((time.perf_counter_ns()-start)/1e6)
        samples.append({'name':name,'available':True,'samples':25,'median_ms':round(statistics.median(times),3),'min_ms':round(min(times),3),'max_ms':round(max(times),3)})
    except (OSError,subprocess.SubprocessError):
        samples.append({'name':name,'available':False})
ci_paths=['.github/workflows/ci.yml','.gitlab-ci.yml','bitbucket-pipelines.yml','.teamcity/settings.kts','Jenkinsfile','azure-pipelines.yml','.circleci/config.yml','.buildkite/pipeline.yml','.drone.yml','.woodpecker.yml','.travis.yml','.cirrus.yml','.semaphore/semaphore.yml','buildspec.yml','cloudbuild.yaml','.build.yml','codefresh.yml','concourse.yml','.gitea/workflows/ci.yml','.forgejo/workflows/ci.yml']
cargo=tomllib.loads((ROOT/'Cargo.toml').read_text())
report={'environment':{'system':platform.system(),'machine':platform.machine()},'method':'25 warm launch-to-pwd runs per available shell, stdout and stderr discarded; includes launcher, parser, builtin and process exit; local observation only','features':{'external_crate_dependencies':len(cargo.get('dependencies',{})),'dashboard_tabs':8,'synthetic_graphs':6,'named_mascots':5,'selectable_themes':3,'ansi_color_indices':256,'ci_adapter_files':sum((ROOT/p).exists() for p in ci_paths),'novel_words_all_endings':len((ROOT/'assets/story/lost-signals.md').read_text().split()),'original_score_seconds':51.891,'release_binary_bytes':binary.stat().st_size},'launch_to_pwd':samples}
out=ROOT/'docs/metrics/local-report.json';out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
