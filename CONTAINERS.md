# Container hangar / actual exporter deployment

The image's default command is the standalone metrics exporter. The interactive shell
needs a terminal and is available by overriding that command. Metrics describe the
exporter process, not a separate desktop shell session.

```sh
docker build -t opsec-demon:local .
docker compose up -d demon
curl http://127.0.0.1:9090/healthz
curl http://127.0.0.1:9090/metrics
docker compose run --rm shell
docker compose -f compose.yml -f compose.monitoring.yml up -d
# Prometheus UI: http://127.0.0.1:9091
```

Compose binds host ports to loopback. Its shell profile is a terminal session; the
exporter service uses a read-only filesystem and drops capabilities. The image runs as
UID/GID 10001. It contains the shell binary and standard runtime utilities, not host
music players or a passed-through Scarlett device.

For local Kubernetes, first load `opsec-demon:local` into your kind/minikube node image
store, or push an image to a registry you own and set the deployment image accordingly.
The repository does not configure a registry. Then, when deploying to your chosen cluster:

```sh
kubectl apply -k k8s
kubectl -n nightfall port-forward svc/opsec-demon 9090:9090
helm lint charts/opsec-demon
helm template nightfall charts/opsec-demon
helm install nightfall charts/opsec-demon --namespace nightfall --create-namespace
```

The Helm chart includes Deployment, Service, labels/helpers, resource defaults, image
pull secrets, node scheduling, readiness/liveness probes, and a values schema. Optional
`serviceMonitor.enabled=true` needs Prometheus Operator CRDs. Set its additional labels
to match your Prometheus selector. Optional `networkPolicy.enabled=true` allows metrics
ingress from namespaces and denies egress; probe behavior also depends on the cluster CNI.
No chart template mounts a credential or executes terminal commands.

References: [Helm charts](https://helm.sh/docs/topics/charts/),
[Kubernetes probes](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/).
