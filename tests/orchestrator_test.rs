use agentboard::orchestrator::prompt;
use agentboard::orchestrator::client::TriagePlan;

#[test]
fn test_triage_prompt_includes_todos() {
    let prompt = prompt::build_triage_prompt("Fix auth\nAdd tests");
    assert!(prompt.contains("Fix auth"));
    assert!(prompt.contains("Add tests"));
}

#[test]
fn test_plan_deserializes() {
    let json = r#"{"teams": [{"name": "Alpha", "task": "Fix auth", "working_dir": "~/proj"}]}"#;
    let plan: TriagePlan = serde_json::from_str(json).unwrap();
    assert_eq!(plan.teams.len(), 1);
    assert_eq!(plan.teams[0].name, "Alpha");
    assert_eq!(plan.teams[0].working_dir, Some("~/proj".into()));
}

#[test]
fn test_plan_with_null_working_dir() {
    let json = r#"{"teams": [{"name": "Beta", "task": "Research", "working_dir": null}]}"#;
    let plan: TriagePlan = serde_json::from_str(json).unwrap();
    assert!(plan.teams[0].working_dir.is_none());
}

#[test]
fn test_summary_prompt_includes_output() {
    let prompt = prompt::build_summary_prompt("Created index.html");
    assert!(prompt.contains("Created index.html"));
}
