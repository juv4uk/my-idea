# Local self-hosted runner

The repository keeps normal GitHub-hosted CI as the always-available compatibility lane.
The local WSL runner is an opt-in lane so an offline workstation never blocks pull requests.

## Required runner labels

Register the repository runner with the custom label:

```text
my-idea
```

GitHub adds the standard `self-hosted`, `linux`, and `x64` labels automatically on a normal x86-64 WSL/Linux runner.
The workflow therefore selects exactly:

```yaml
runs-on: [self-hosted, linux, x64, my-idea]
```

## Registration

In GitHub open **Settings → Actions → Runners → New self-hosted runner**, choose Linux/x64,
then run the generated commands inside WSL. Add `--labels my-idea` to the generated
`config.sh` command.

A typical layout is:

```bash
mkdir -p /home/agents/actions-runner/my-idea
cd /home/agents/actions-runner/my-idea
# download/extract the GitHub Actions runner using GitHub's current generated commands
./config.sh --url https://github.com/juv4uk/my-idea --token <one-time-token> --name my-idea-wsl --labels my-idea
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

## Running the repository lane

Open **Actions → CI · Перевірка · Prüfung → Run workflow**.
A manual dispatch runs the normal hosted job and, when the local runner is online,
the `Local my-idea runner smoke` job. The local job proves runner identity, exact
SENS gitlink checkout, the single-pin invariant, local toolchain presence, and the
lightweight Bun test gate.

The local lane intentionally does not replace the hosted gate. Once its environment
is stable, heavier build/witness jobs can be moved or mirrored deliberately.
