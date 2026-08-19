# ecosystem/ scaffold (Swarm Contract v0.1, MYIDEA-SWARM-CONTRACT-01)

Per `repo.my`'s `(imports language-contract isa-contract compatibility
evidence repo-declarations)`: `my-idea` doesn't track cross-repo state
as versioned epistemic "claims" the way `shiva-sutras`/`my-lisp-panini`
do (see their `ecosystem/imports/*.my` files, `(claim ID (revision ...)
(status ...))` shape) — `my-idea`'s imports are contracts and evidence
records read live off disk and rendered, not hypotheses adopted into
its own reasoning. `src-tauri/src/ecosystem/{contracts,evidence,
repo_graph}.rs` already read `language-contract.my`/`isa-contract.my`/
`compatibility.my`/`evidence/*.my`/`repo.my` directly, fresh on every
"Run ecosystem check" — there is no separate cached claims file for
this repo to keep in sync, because the visualization *is* the sync.

No `imports/*.my` files are populated in this scaffold as of this
writing — an empty placeholder would be worse than an explanation of
why it's empty (same principle fpga-lisp's own `ecosystem/README.md`
applies).
