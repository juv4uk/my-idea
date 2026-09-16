//! [#51 IDE-KEYMAP-AUTHORITY-1] A single, inspectable keybinding-resolution
//! mechanism: policy is explicit, data-driven precedence, and collisions
//! are represented, never silently decided by load order or extension
//! array position.
//!
//! Participants (v0): Lisp plugin bindings (`editor/keymap`), my-idea's
//! own built-in commands, and host/WebView-reserved shortcuts. CodeMirror's
//! own default keymaps aren't enumerated here — they already sit at the
//! lowest priority in `editor.cljs`'s extension order (Lisp/built-in
//! bindings are installed first), so nothing needs to compete with them
//! explicitly; a key claimed by any participant this module knows about
//! already wins over CodeMirror's un-enumerable, ever-growing default set,
//! which is exactly the "don't hard-code an ever-growing table" the issue
//! warns against — we never try to list CodeMirror's defaults.
//!
//! A key is an input surface, not semantic identity: `Binding.command` is
//! a plain command name, entirely independent of what key currently
//! reaches it (see `command_identity_survives_rebinding` below).

use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Source {
    /// `editor/keymap` from a loaded `.lisp` plugin — the highest priority
    /// we resolve among, matching Emacs' own "your config wins over
    /// built-ins" precedence.
    UserPlugin,
    /// A my-idea built-in command (e.g. go-to-definition) — not
    /// Lisp-authored, but still ours, so it outranks CodeMirror's generic
    /// library defaults.
    BuiltIn,
}

impl Source {
    fn rank(self) -> u8 {
        match self {
            Source::UserPlugin => 0,
            Source::BuiltIn => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Binding {
    pub key: String,
    pub command: String,
    pub source: Source,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resolution {
    pub key: String,
    pub candidates: Vec<Binding>,
    /// `None` means no participant actually gets this key — either it is
    /// host-reserved (see `HOST_RESERVED`), or (impossible in practice
    /// today, but represented rather than assumed) no candidate existed.
    pub winner: Option<Binding>,
    pub reason: String,
}

/// Keys the host/WebView reserves — currently just F12 (devtools, enabled
/// by the `devtools` Cargo feature). A Lisp plugin or built-in *can*
/// register a binding for one of these (the registration itself isn't
/// refused — Lisp still owns whether the attempt is meaningful), but the
/// resolver reports it as `winner: None` rather than pretending the
/// binding will actually fire: the OS/WebView layer sees the key first,
/// and reports what it reserves instead of silently deciding Lisp-side
/// behavior.
pub const HOST_RESERVED: &[(&str, &str)] = &[("F12", "WebView/OS devtools shortcut")];

/// Resolves every registered key into exactly one `Resolution`, grouping
/// candidates by key. Deterministic: for a given input list (any order),
/// the winner is always the lowest-rank source; ties within the same rank
/// go to the *last* candidate in the input — this is not a guess, it is
/// the same "later registration wins" rule `EditorCommandRegistry` already
/// uses for a single source (`editor/keymap` re-binding a key), now made
/// visible as data instead of a silent overwrite.
pub fn resolve(bindings: Vec<Binding>) -> Vec<Resolution> {
    let mut by_key: BTreeMap<String, Vec<Binding>> = BTreeMap::new();
    for binding in bindings {
        by_key.entry(binding.key.clone()).or_default().push(binding);
    }

    by_key
        .into_iter()
        .map(|(key, candidates)| {
            if let Some((_, host_reason)) = HOST_RESERVED.iter().find(|(reserved, _)| *reserved == key) {
                return Resolution {
                    key,
                    candidates,
                    winner: None,
                    reason: format!("host-reserved: {host_reason}"),
                };
            }

            let min_rank = candidates.iter().map(|b| b.source.rank()).min().expect("at least one candidate per key");
            let winner = candidates.iter().filter(|b| b.source.rank() == min_rank).last().cloned();
            let reason = if candidates.len() == 1 {
                "no collision".to_string()
            } else {
                format!(
                    "{} candidates for this key; highest-priority source wins (source rank, then last-registered)",
                    candidates.len()
                )
            };
            Resolution { key, candidates, winner, reason }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding(key: &str, command: &str, source: Source) -> Binding {
        Binding { key: key.to_string(), command: command.to_string(), source }
    }

    #[test]
    fn a_key_with_one_candidate_has_no_collision() {
        let resolutions = resolve(vec![binding("Ctrl-g", "greet", Source::UserPlugin)]);
        assert_eq!(resolutions.len(), 1);
        assert_eq!(resolutions[0].reason, "no collision");
        assert_eq!(resolutions[0].winner.as_ref().unwrap().command, "greet");
    }

    #[test]
    fn plugin_beats_builtin_for_the_same_key() {
        let resolutions = resolve(vec![
            binding("Ctrl-g", "builtin-command", Source::BuiltIn),
            binding("Ctrl-g", "plugin-command", Source::UserPlugin),
        ]);
        assert_eq!(resolutions.len(), 1);
        let resolution = &resolutions[0];
        assert_eq!(resolution.candidates.len(), 2);
        assert_eq!(resolution.winner.as_ref().unwrap().command, "plugin-command");
        assert_eq!(resolution.winner.as_ref().unwrap().source, Source::UserPlugin);
    }

    #[test]
    fn plugin_vs_plugin_collision_is_deterministic_last_registered_wins() {
        let resolutions = resolve(vec![
            binding("Ctrl-g", "first-plugin-command", Source::UserPlugin),
            binding("Ctrl-g", "second-plugin-command", Source::UserPlugin),
        ]);
        let resolution = &resolutions[0];
        assert_eq!(resolution.candidates.len(), 2);
        assert_eq!(resolution.winner.as_ref().unwrap().command, "second-plugin-command");
        assert!(resolution.reason.contains("collision") || resolution.reason.contains("candidates"));
    }

    #[test]
    fn host_reserved_key_declines_every_candidate_explicitly() {
        let resolutions = resolve(vec![
            binding("F12", "go-to-definition", Source::BuiltIn),
            binding("F12", "some-plugin-command", Source::UserPlugin),
        ]);
        let resolution = &resolutions[0];
        assert_eq!(resolution.key, "F12");
        assert!(resolution.winner.is_none(), "a host-reserved key must never report a winner");
        assert!(resolution.reason.contains("host-reserved"));
        // Still fully transparent: both attempts remain visible as
        // candidates, never silently dropped.
        assert_eq!(resolution.candidates.len(), 2);
    }

    #[test]
    fn resolving_the_same_input_twice_is_bit_for_bit_identical() {
        // "no nondeterministic load-order winner" / "reload preserves
        // deterministic resolution": resolve() is a pure function of its
        // input, so calling it again (e.g. after an explicit plugin
        // reload re-collects the same bindings) can never flip a winner
        // unless the actual input changed.
        let bindings = vec![
            binding("Ctrl-g", "a", Source::UserPlugin),
            binding("Ctrl-g", "b", Source::UserPlugin),
            binding("F12", "go-to-definition", Source::BuiltIn),
        ];
        let first = resolve(bindings.clone());
        let second = resolve(bindings);
        assert_eq!(
            first.iter().map(|r| (r.key.clone(), r.winner.as_ref().map(|w| w.command.clone()))).collect::<Vec<_>>(),
            second.iter().map(|r| (r.key.clone(), r.winner.as_ref().map(|w| w.command.clone()))).collect::<Vec<_>>(),
        );
    }

    #[test]
    fn command_identity_survives_rebinding_a_key() {
        // "A key is an input surface, not semantic identity": the same
        // command name can appear under a completely different key without
        // that changing what the command *is* -- resolve() never mutates
        // or derives a command's identity from its key spelling.
        let bound_to_a = resolve(vec![binding("Ctrl-a", "greet", Source::UserPlugin)]);
        let bound_to_z = resolve(vec![binding("Ctrl-z", "greet", Source::UserPlugin)]);
        assert_eq!(bound_to_a[0].winner.as_ref().unwrap().command, "greet");
        assert_eq!(bound_to_z[0].winner.as_ref().unwrap().command, "greet");
    }
}
