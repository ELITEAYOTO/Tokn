use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use tokn_domain::AgentEvidence;

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceInventory {
    pub schema_version: u64,
    pub watch_root: String,
    #[serde(default)]
    pub candidates: Vec<WorkspaceInventoryCandidate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceInventoryCandidate {
    pub root: String,
    #[serde(default)]
    pub relative: String,
    #[serde(default)]
    pub markers: Vec<WorkspaceMarker>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceMarker {
    pub name: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkspaceResolutionStatus {
    Selected,
    Ambiguous,
    #[default]
    NoCandidate,
}

impl WorkspaceResolutionStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Selected => "SELECTED",
            Self::Ambiguous => "AMBIGUOUS",
            Self::NoCandidate => "NO_CANDIDATE",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct WorkspaceCandidateScore {
    pub root: String,
    pub relative: String,
    pub marker_count: u64,
    pub existed_before: Option<bool>,
    pub declared_expected: bool,
    pub is_source_root: bool,
    pub direct_tool_workdir_hits: u64,
    pub parent_tool_workdir_hits: u64,
    pub agent_cwd_hits: u64,
    pub score: i64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct WorkspaceResolution {
    pub status: WorkspaceResolutionStatus,
    pub source_root: String,
    pub watch_root: String,
    pub selected_root: Option<String>,
    pub candidates: Vec<WorkspaceCandidateScore>,
}

pub fn resolve_workspace(
    before: Option<&WorkspaceInventory>,
    after: &WorkspaceInventory,
    source_root: &str,
    expected_outputs: &[String],
    evidence: &[AgentEvidence],
) -> WorkspaceResolution {
    let before_roots = before
        .map(|inventory| {
            inventory
                .candidates
                .iter()
                .map(|candidate| normalize_path(&candidate.root))
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    let expected = expected_outputs
        .iter()
        .map(|path| normalize_path(path))
        .collect::<HashSet<_>>();
    let source_norm = normalize_path(source_root);
    let watch_root_norm = normalize_path(&after.watch_root);

    let tool_workdirs = count_tool_workdirs(evidence);
    let agent_cwds = count_agent_cwds(evidence);

    let mut candidates = after
        .candidates
        .iter()
        .map(|candidate| {
            score_candidate(
                candidate,
                before.map(|_| &before_roots),
                &source_norm,
                &watch_root_norm,
                &expected,
                &tool_workdirs,
                &agent_cwds,
            )
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.root.cmp(&right.root))
    });

    let Some(best) = candidates.first() else {
        return WorkspaceResolution {
            status: WorkspaceResolutionStatus::NoCandidate,
            source_root: source_root.to_string(),
            watch_root: after.watch_root.clone(),
            selected_root: None,
            candidates,
        };
    };

    if best.score <= 0 {
        return WorkspaceResolution {
            status: WorkspaceResolutionStatus::NoCandidate,
            source_root: source_root.to_string(),
            watch_root: after.watch_root.clone(),
            selected_root: None,
            candidates,
        };
    }

    let best_score = best.score;
    let ties = candidates
        .iter()
        .filter(|candidate| candidate.score == best_score)
        .count();

    let (status, selected_root) = if ties == 1 {
        (WorkspaceResolutionStatus::Selected, Some(best.root.clone()))
    } else {
        (WorkspaceResolutionStatus::Ambiguous, None)
    };

    WorkspaceResolution {
        status,
        source_root: source_root.to_string(),
        watch_root: after.watch_root.clone(),
        selected_root,
        candidates,
    }
}

fn score_candidate(
    candidate: &WorkspaceInventoryCandidate,
    before_roots: Option<&HashSet<String>>,
    source_root: &str,
    watch_root: &str,
    expected_outputs: &HashSet<String>,
    tool_workdirs: &HashMap<String, u64>,
    agent_cwds: &HashMap<String, u64>,
) -> WorkspaceCandidateScore {
    let root = normalize_path(&candidate.root);
    let existed_before = before_roots.map(|roots| roots.contains(&root));
    let declared_expected = expected_outputs.contains(&root);
    let is_source_root = root == source_root;

    let mut direct_tool_workdir_hits = 0_u64;
    let mut parent_tool_workdir_hits = 0_u64;
    for (workdir, count) in tool_workdirs {
        if workdir == watch_root {
            continue;
        }
        if path_is_same_or_child(workdir, &root) {
            direct_tool_workdir_hits = direct_tool_workdir_hits.saturating_add(*count);
        } else if path_is_same_or_child(&root, workdir) {
            parent_tool_workdir_hits = parent_tool_workdir_hits.saturating_add(*count);
        }
    }

    let mut agent_cwd_hits = 0_u64;
    for (cwd, count) in agent_cwds {
        if cwd == watch_root {
            continue;
        }
        if path_is_same_or_child(cwd, &root) || path_is_same_or_child(&root, cwd) {
            agent_cwd_hits = agent_cwd_hits.saturating_add(*count);
        }
    }

    let mut score = 0_i64;
    let mut reasons = Vec::new();

    if declared_expected {
        score += 10_000;
        reasons.push("declared expected output".into());
    }

    if existed_before == Some(false) {
        score += 30;
        reasons.push("new workspace since pre-run inventory".into());
    }

    if is_source_root {
        score += 20;
        reasons.push("original source workspace".into());
    }

    if direct_tool_workdir_hits > 0 {
        score += i64::try_from(direct_tool_workdir_hits.saturating_mul(100)).unwrap_or(i64::MAX);
        reasons.push(format!(
            "{direct_tool_workdir_hits} direct tool workdir hit(s)"
        ));
    }

    if parent_tool_workdir_hits > 0 {
        score += i64::try_from(parent_tool_workdir_hits.saturating_mul(50)).unwrap_or(i64::MAX);
        reasons.push(format!(
            "{parent_tool_workdir_hits} parent tool workdir hit(s)"
        ));
    }

    if agent_cwd_hits > 0 {
        score += i64::try_from(agent_cwd_hits.saturating_mul(5)).unwrap_or(i64::MAX);
        reasons.push(format!("{agent_cwd_hits} agent cwd hit(s)"));
    }

    WorkspaceCandidateScore {
        root: candidate.root.clone(),
        relative: candidate.relative.clone(),
        marker_count: candidate.markers.len() as u64,
        existed_before,
        declared_expected,
        is_source_root,
        direct_tool_workdir_hits,
        parent_tool_workdir_hits,
        agent_cwd_hits,
        score,
        reasons,
    }
}

fn count_tool_workdirs(evidence: &[AgentEvidence]) -> HashMap<String, u64> {
    let mut out = HashMap::new();
    for agent in evidence {
        for tool in &agent.tools {
            if let Some(workdir) = tool.workdir.as_deref() {
                *out.entry(normalize_path(workdir)).or_insert(0) += 1;
            }
        }
    }
    out
}

fn count_agent_cwds(evidence: &[AgentEvidence]) -> HashMap<String, u64> {
    let mut out = HashMap::new();
    for agent in evidence {
        if let Some(cwd) = agent.cwd.as_deref() {
            *out.entry(normalize_path(cwd)).or_insert(0) += 1;
        }
    }
    out
}

fn normalize_path(path: &str) -> String {
    path.replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

fn path_is_same_or_child(path: &str, root: &str) -> bool {
    if path == root {
        return true;
    }

    let mut prefix = root.to_string();
    prefix.push('\\');
    path.starts_with(&prefix)
}

#[cfg(test)]
mod tests {
    use tokn_domain::ToolObservation;

    use super::*;

    fn inventory(roots: &[&str]) -> WorkspaceInventory {
        WorkspaceInventory {
            schema_version: 1,
            watch_root: r"E:\fixture".into(),
            candidates: roots
                .iter()
                .map(|root| WorkspaceInventoryCandidate {
                    root: (*root).into(),
                    relative: root.to_string(),
                    markers: vec![WorkspaceMarker {
                        name: "package.json".into(),
                    }],
                })
                .collect(),
        }
    }

    fn evidence(workdirs: &[&str]) -> Vec<AgentEvidence> {
        vec![AgentEvidence {
            thread_id: "root".into(),
            cwd: Some(r"E:\fixture".into()),
            tools: workdirs
                .iter()
                .enumerate()
                .map(|(index, workdir)| ToolObservation {
                    tool_call_id: format!("tool-{index}"),
                    workdir: Some((*workdir).into()),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }]
    }

    #[test]
    fn tool_activity_selects_new_candidate_over_new_freeze() {
        let before = inventory(&[r"E:\fixture\source\PROJECT"]);
        let after = inventory(&[
            r"E:\fixture\source\PROJECT",
            r"E:\fixture\freeze\PROJECT",
            r"E:\fixture\candidate\PROJECT",
        ]);
        let evidence = evidence(&[
            r"E:\fixture\candidate\PROJECT",
            r"E:\fixture\candidate\PROJECT",
            r"E:\fixture\candidate",
        ]);

        let result = resolve_workspace(
            Some(&before),
            &after,
            r"E:\fixture\source\PROJECT",
            &[],
            &evidence,
        );

        assert_eq!(result.status, WorkspaceResolutionStatus::Selected);
        assert_eq!(
            result.selected_root.as_deref(),
            Some(r"E:\fixture\candidate\PROJECT")
        );
    }

    #[test]
    fn equal_new_candidates_without_activity_are_ambiguous() {
        let before = inventory(&[r"E:\fixture\source\PROJECT"]);
        let after = inventory(&[
            r"E:\fixture\source\PROJECT",
            r"E:\fixture\a\PROJECT",
            r"E:\fixture\b\PROJECT",
        ]);

        let result = resolve_workspace(
            Some(&before),
            &after,
            r"E:\fixture\source\PROJECT",
            &[],
            &[],
        );

        assert_eq!(result.status, WorkspaceResolutionStatus::Ambiguous);
        assert!(result.selected_root.is_none());
    }

    #[test]
    fn generic_watch_root_is_not_candidate_evidence() {
        let after = inventory(&[
            r"E:\fixture\source\PROJECT",
            r"E:\fixture\candidate\PROJECT",
        ]);
        let generic = evidence(&[r"E:\fixture"]);

        let result = resolve_workspace(None, &after, r"E:\fixture\source\PROJECT", &[], &generic);

        let candidate = result
            .candidates
            .iter()
            .find(|item| item.root.ends_with(r"candidate\PROJECT"))
            .expect("candidate");
        assert_eq!(candidate.direct_tool_workdir_hits, 0);
        assert_eq!(candidate.parent_tool_workdir_hits, 0);
        assert_eq!(candidate.agent_cwd_hits, 0);
    }

    #[test]
    fn declared_expected_output_has_priority() {
        let after = inventory(&[
            r"E:\fixture\source\PROJECT",
            r"E:\fixture\candidate\PROJECT",
        ]);
        let result = resolve_workspace(
            None,
            &after,
            r"E:\fixture\source\PROJECT",
            &[r"E:\fixture\candidate\PROJECT".into()],
            &[],
        );

        assert_eq!(result.status, WorkspaceResolutionStatus::Selected);
        assert_eq!(
            result.selected_root.as_deref(),
            Some(r"E:\fixture\candidate\PROJECT")
        );
    }
}
