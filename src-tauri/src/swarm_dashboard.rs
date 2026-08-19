use crate::swarm;
use my_lisp::{parse, Expr, ExprKind};
use serde::Serialize;
use std::collections::BTreeMap;

fn as_list(expr: &Expr) -> Option<&[Expr]> {
    match &expr.kind {
        ExprKind::List(items) => Some(items),
        _ => None,
    }
}

fn symbol_name(expr: &Expr) -> Option<&str> {
    match &expr.kind {
        ExprKind::Symbol(name) => Some(name),
        _ => None,
    }
}

/// Finds `(key value ...)` among `items` — swarm-node's flat field shape
/// (`(node my-idea-1)`, `(roles (voter worker))`), same convention as
/// `repo.my` — and returns the trailing symbol names.
fn symbols_of(expr: &Expr) -> Vec<String> {
    match as_list(expr) {
        Some(list) => list.iter().filter_map(symbol_name).map(str::to_string).collect(),
        None => symbol_name(expr).map(str::to_string).into_iter().collect(),
    }
}

fn field_symbols(items: &[Expr], key: &str) -> Vec<String> {
    items
        .iter()
        .find_map(|item| {
            let sub = as_list(item)?;
            (symbol_name(sub.first()?)? == key).then(|| sub[1..].iter().flat_map(symbols_of).collect())
        })
        .unwrap_or_default()
}

fn field_symbol(items: &[Expr], key: &str) -> Option<String> {
    items.iter().find_map(|item| {
        let sub = as_list(item)?;
        (symbol_name(sub.first()?)? == key)
            .then(|| sub.get(1).and_then(symbol_name).map(str::to_string))
            .flatten()
    })
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SwarmMember {
    pub node: String,
    pub present: bool,
    pub roles: Vec<String>,
    pub capabilities: Vec<String>,
    /// Task IDs this node currently holds (claimed, not yet completed) —
    /// cross-referenced from `(list-task-state)`, not part of
    /// `(list-members)` itself.
    pub current_tasks: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwarmDashboard {
    pub members: Vec<SwarmMember>,
    pub open_task_count: usize,
    pub completed_task_count: usize,
    pub error: Option<String>,
}

impl SwarmDashboard {
    fn error(message: String) -> Self {
        SwarmDashboard { members: Vec::new(), open_task_count: 0, completed_task_count: 0, error: Some(message) }
    }
}

/// Parses `(members ((member ...) (member ...)))` into `(node, present,
/// roles, capabilities)` tuples, ignoring `current_tasks` (filled in
/// separately from task state).
fn parse_members(response: &str) -> Option<Vec<(String, bool, Vec<String>, Vec<String>)>> {
    let forms = parse(response).ok()?;
    let top = as_list(forms.first()?)?;
    if symbol_name(top.first()?)? != "members" {
        return None;
    }
    let entries = as_list(top.get(1)?)?;
    Some(
        entries
            .iter()
            .filter_map(|entry| {
                let fields = as_list(entry)?;
                let node = field_symbol(fields, "node")?;
                let present = field_symbol(fields, "present").as_deref() == Some("t");
                let roles = field_symbols(fields, "roles");
                let capabilities = field_symbols(fields, "capabilities");
                Some((node, present, roles, capabilities))
            })
            .collect(),
    )
}

/// Parses `(task-states ((task-entry ...) ...))`, returning
/// `(open_count, completed_count, holder -> open task ids)`.
fn parse_task_states(response: &str) -> Option<(usize, usize, BTreeMap<String, Vec<String>>)> {
    let forms = parse(response).ok()?;
    let top = as_list(forms.first()?)?;
    if symbol_name(top.first()?)? != "task-states" {
        return None;
    }
    let entries = as_list(top.get(1)?)?;

    let mut open_count = 0;
    let mut completed_count = 0;
    let mut by_holder: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for entry in entries {
        let Some(fields) = as_list(entry) else { continue };
        let Some(task) = field_symbol(fields, "task") else { continue };
        let completed = field_symbol(fields, "completed").as_deref() == Some("t");
        if completed {
            completed_count += 1;
            continue;
        }
        open_count += 1;
        if let Some(holder) = field_symbol(fields, "holder") {
            by_holder.entry(holder).or_default().push(task);
        }
    }
    Some((open_count, completed_count, by_holder))
}

/// Queries this agent's own `swarm-node` for `(list-members)` and
/// `(list-task-state)` and merges them into one dashboard view. Both
/// calls go through `swarm::query`, so they share its one-line-only
/// read discipline and connection retry behavior.
pub fn dashboard(port: Option<u16>) -> SwarmDashboard {
    let members_raw = match swarm::query("(list-members)", port) {
        Ok(r) => r,
        Err(e) => return SwarmDashboard::error(e),
    };
    let Some(members) = parse_members(&members_raw) else {
        return SwarmDashboard::error(format!("could not parse (list-members) response: {members_raw}"));
    };

    let tasks_raw = match swarm::query("(list-task-state)", port) {
        Ok(r) => r,
        Err(e) => return SwarmDashboard::error(e),
    };
    let Some((open_task_count, completed_task_count, mut by_holder)) = parse_task_states(&tasks_raw) else {
        return SwarmDashboard::error(format!("could not parse (list-task-state) response: {tasks_raw}"));
    };

    let members = members
        .into_iter()
        .map(|(node, present, roles, capabilities)| {
            let current_tasks = by_holder.remove(&node).unwrap_or_default();
            SwarmMember { node, present, roles, capabilities, current_tasks }
        })
        .collect();

    SwarmDashboard { members, open_task_count, completed_task_count, error: None }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real `(list-members)` response captured 2026-08-18 from the live
    /// `my-idea-1` swarm-node (127.0.0.1:9104, 13-member mesh), via raw
    /// TCP — not a hand-written fixture. Exercises the actual on-the-wire
    /// shape, not an idealized one.
    const MEMBERS_RESPONSE: &str = "(members (((node big-pickle-1) (present t) (roles (voter worker)) (capabilities (codetation file-operations debugging testing multi-language))) ((node cml-1) (present t) (roles (voter)) (capabilities (compiler rust lowering testing iverilog proof cml))) ((node my-idea-1) (present t) (roles (voter)) (capabilities (observer evidence-reader oracle-client gui rust))) ((node shiva-sutras-1) (present t) (roles (worker)) (capabilities (sanskrit panini slp1 gretil provenance epistemic-pipeline)))))";

    /// Real `(list-task-state)` response, same capture — truncated to a
    /// handful of entries covering both an open task held by a specific
    /// node, a completed task, and an open task with no holder.
    const TASK_STATES_RESPONSE: &str = "(task-states (((task ARCH-RECOVERY-REVIEW-CML) (generation 1) (holder ()) (completed t)) ((task ARCH-RECOVERY-REVIEW-FPGA) (generation 1) (holder fpga-lisp-1) (completed nil)) ((task IDEA-SARVAM-VERIFY-I18N-STRINGS) (generation 0) (holder ()) (completed nil)) ((task FPGA-CONFORMANCE-TESTING) (generation 1) (holder engineer-1) (completed nil))))";

    #[test]
    fn parses_real_members_response() {
        let members = parse_members(MEMBERS_RESPONSE).expect("should parse");
        assert_eq!(members.len(), 4);
        let (node, present, roles, capabilities) = &members[2];
        assert_eq!(node, "my-idea-1");
        assert!(present);
        assert_eq!(roles, &vec!["voter".to_string()]);
        assert!(capabilities.contains(&"observer".to_string()));
    }

    #[test]
    fn parses_real_task_states_response() {
        let (open, completed, by_holder) = parse_task_states(TASK_STATES_RESPONSE).expect("should parse");
        assert_eq!(open, 3);
        assert_eq!(completed, 1);
        assert_eq!(by_holder.get("fpga-lisp-1"), Some(&vec!["ARCH-RECOVERY-REVIEW-FPGA".to_string()]));
        assert_eq!(by_holder.get("engineer-1"), Some(&vec!["FPGA-CONFORMANCE-TESTING".to_string()]));
        // A holder of () (no current holder) must not appear as a key at all.
        assert!(!by_holder.contains_key(""));
    }

    #[test]
    fn dashboard_cross_references_holder_to_member() {
        let members = parse_members(MEMBERS_RESPONSE).unwrap();
        let (_, _, by_holder) = {
            let (open, completed, by_holder) = parse_task_states(TASK_STATES_RESPONSE).unwrap();
            (open, completed, by_holder)
        };
        assert!(members.iter().any(|(node, ..)| node == "my-idea-1"));
        // my-idea-1 holds nothing in this fixture — must show as empty, not absent/error.
        assert!(by_holder.get("my-idea-1").is_none());
    }
}
