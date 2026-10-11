# OpSecULAR 97

A fake operating system inside the real shell. Its desktop, recycle bin, Chinese
WSL console, serial bus, game balance, cooking, and explosions are explicitly SIM
effects held in session memory. They do not operate host files or connect to WSL,
Roblox, Tama-soft software, or a serial device.

Enter these commands one per line in either the shell or TUI:

```text
opsecular boot
opsecular desktop
opsecular recycle put lost-signals.sav
opsecular recycle ls
opsecular recycle restore lost-signals.sav
opsecular wsl
opsecular terminal
novel start
opsecular operator krillion
mascot krillion
opsecular spirit 97
opsecular support
opsecular poutine next
opsecular poutine next
opsecular poutine next
opsecular poutine next
opsecular poutine next
opsecular weep 3
```

The terminal prop identifies `OPSECULAR97::LARP-CITY::MAINFRAME`. Lost Signals is
the existing original, non-explicit Tama-soft-inspired tribute. Use `novel` to play
its scenes. The Roblox annex shares the identifier for a local Studio prop.

Krillion has invented lane pressure 97 versus Mo & Krill's 72, and support healing
`120 + 12 * spirit` versus `100 + 3 * spirit`. Spirit accepts 0–10000. This deliberately
absurd counterpick is fictional balance, independent of any game's live rules.

The kitchen advances through five steps from potatoes to gravy, curds, and serving.
Use `opsecular poutine reset` to cook again. `opsecular weep N` prints 1–16 cycles
of tears, explosion, and respawn; it never crashes the shell. The recycle bin holds
at most 32 named props, restored by name or cleared with `opsecular recycle empty`.

## Kubernetes package

`k8s/opsecular97` is a separate Kustomize package for the existing container image.
Its startup script prints the fake OS boot, Chinese console, mainframe/serial
loopback, nine-track catalog, poutine, and three explosion cycles to pod logs,
then runs the actual metrics exporter. An interactive operator session is a separate
process with its own in-memory state:

```sh
docker build -t opsec-demon:local .
# Load this image into your chosen local cluster's image store first.
kubectl kustomize k8s/opsecular97
kubectl apply -k k8s/opsecular97
kubectl -n opsecular97 logs deployment/opsecular97
kubectl -n opsecular97 exec -it deployment/opsecular97 -- opsec-demon --tui
```

In that operator session, run `opsecular boot`. You can render songs under `/tmp`;
the 64 MiB scratch volume is ephemeral and no audio player/device is provided in
the container. The pod uses the existing health endpoints, an unprivileged user,
a read-only root filesystem, and no service-account token. The package has been
prepared locally; applying it requires your chosen cluster and image to be ready.
