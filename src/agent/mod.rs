pub mod team;
pub mod registry;
pub mod runner;

#[derive(Debug, Clone, PartialEq)]
pub enum BackendMode { OneShot, Interactive }

pub struct BackendConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub mode: BackendMode,
}

impl BackendConfig {
    pub fn build_command(&self, task: &str) -> (String, Vec<String>) {
        let args: Vec<String> = self.args.iter().map(|a| a.replace("{task}", task)).collect();
        (self.command.clone(), args)
    }
}
