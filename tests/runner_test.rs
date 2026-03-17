use agentboard::agent::registry::BackendRegistry;
use agentboard::config::Config;

#[test]
fn test_registry_loads_from_config() {
    let config = Config::default();
    let registry = BackendRegistry::from_config(&config);
    assert_eq!(config.backend.default, "claude");
    // registry is built successfully (no panic)
    drop(registry);
}

#[test]
fn test_build_command_replaces_task() {
    let mut config = Config::default();
    config.backends.insert("claude".into(), agentboard::config::BackendEntry {
        command: "claude".into(),
        args: vec!["--print".into(), "{task}".into()],
        mode: "one-shot".into(),
        description: "test".into(),
    });
    let registry = BackendRegistry::from_config(&config);
    let backend = registry.get("claude").unwrap();
    let (cmd, args) = backend.build_command("do something");
    assert_eq!(cmd, "claude");
    assert_eq!(args, vec!["--print", "do something"]);
}
