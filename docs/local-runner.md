# Local self-hosted runner

The repository uses the WSL self-hosted runner as the primary push/pull-request
CI lane. The GitHub-hosted lane is retained as a manual compatibility check
because hosted jobs for this private repository have recently failed before a
runner is allocated.

## Required runner labels

Use GitHub's standard self-hosted Linux/x64 runner labels for scheduling. The workflow selects:

```yaml
runs-on: [self-hosted, linux, x64]
```

Guix is verified as the first runtime contract step inside the job rather than encoded as a scheduler label. This lets a correctly registered my-idea WSL runner take the job even when it was not configured with the custom `guix` label.

## Registration

In GitHub open **Settings → Actions → Runners → New self-hosted runner**, choose
Linux/x64, then run the generated commands inside WSL. Keep GitHub's default Linux/x64 labels when registering the runner. A custom `guix` label is optional and is not required by this repository.

A typical layout is:

```bash
mkdir -p /home/agents/actions-runner/my-idea
cd /home/agents/actions-runner/my-idea
# download/extract the GitHub Actions runner using GitHub's generated commands
./config.sh --url https://github.com/juv4uk/my-idea --token <one-time-token> \
  --name my-idea-wsl
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

Every pull request queues exactly one `Local my-idea CI · WSL/Guix` job on `[self-hosted, linux, x64]`. Pushes do not create a duplicate automatic run.
It verifies the exact SENS gitlink, the single-pin invariant, builds the frontend
and SENS/WASM integration, checks/tests the frontend, builds the pinned REPL
sidecar, checks out the exact CML SHA, builds `cml-compile`, and runs Rust
witnesses inside the declared Guix environment.

A manual **Run workflow** also exposes the GitHub-hosted compatibility lane.
That lane is not required for normal PR progress while hosted allocation is
unavailable.
