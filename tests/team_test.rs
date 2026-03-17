use agentboard::agent::team::{Team, TeamStatus};

#[test]
fn test_team_creation() {
    let team = Team::new("Alpha".into(), "Build landing page".into(), "claude".into(), Some("~/projects/site".into()));
    assert_eq!(team.name, "Alpha");
    assert_eq!(team.status, TeamStatus::Queued);
    assert!(team.output.is_empty());
    assert!(team.summary.is_none());
}

#[test]
fn test_team_status_transitions() {
    let mut team = Team::new("Beta".into(), "Fix bug".into(), "claude".into(), None);
    assert!(team.transition_to(TeamStatus::Running).is_ok());
    assert!(team.transition_to(TeamStatus::Done).is_ok());
    assert!(team.transition_to(TeamStatus::Running).is_err());
}

#[test]
fn test_team_append_output() {
    let mut team = Team::new("Gamma".into(), "Test".into(), "claude".into(), None);
    team.append_output("line 1");
    team.append_output("line 2");
    assert_eq!(team.output.len(), 2);
}

#[test]
fn test_name_sanitization() {
    let team = Team::new("Alpha & Beta!".into(), "task".into(), "claude".into(), None);
    assert_eq!(team.name, "AlphaBeta");
}
