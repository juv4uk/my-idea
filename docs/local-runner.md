# Local self-hosted runner

The repository uses the WSL self-hosted runner as the primary push/pull-request
CI lane. The GitHub-hosted lane is retained as a manual compatibility check
because hosted jobs for this private repository have recently failed before a
runner is allocated.

## Required runner labels

Reuse the ecosystem Guix runner class. The workflow selects:

```yaml
runs-on: [self-hosted, guix]
```

This matches the live SENS/CML control-plane runner convention and avoids
creating a repository-specific scheduling class when the same WSL/Guix host
already serves those repositories.

## Registration

In GitHub open **Settings → Actions → Runners → New self-hosted runner**, choose
Linux/x64, then run the generated commands inside WSL. Use the ecosystem `guix` label when registering this repository's runner instance.

A typical layout is:

```bash
mkdir -p /home/agents/actions-runner/my-idea
cd /home/agents/actions-runner/my-idea
# download/extract the GitHub Actions runner using GitHub's generated commands
./config.sh --url https://github.com/juv4uk/my-idea --token <one-time-token> \
  --name wsm-i5-6400 --labels guix
./run.sh
```

Do not commit the one-time registration token.

For a service installation:

```bash
cd /home/agents/actions-runner/my-idea
sudo ./svc.sh install
sudo ./svc.sh start
sudo ./svc.sh status
```

## Private dependency token

This repository consumes private SENS/CML sources. Workflows use the same token
convention as the rest of the ecosystem:

```text
SUBMODULE_READ_TOKEN || SENS_READ_TOKEN
```

At least one of those repository secrets must be present and have read access to
`juv4uk/sens` and `juv4uk/cml`. Never put the token itself in a workflow,
service unit, repository file, or runner command line.

## What runs locally

Every push and pull request queues `Local my-idea CI` on the `[self-hosted, guix]` runner class.
It verifies the exact SENS gitlink, the single-pin invariant, builds the frontend
and SENS/WASM integration, checks/tests the frontend, builds the pinned REPL
sidecar, checks out the exact CML SHA, builds `cml-compile`, and runs Rust
witnesses inside the declared Guix environment.

A manual **Run workflow** also exposes the GitHub-hosted compatibility lane.
That lane is not required for normal PR progress while hosted allocation is
unavailable.
