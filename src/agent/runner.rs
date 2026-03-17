use crate::agent::BackendConfig;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::broadcast;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct AgentProcess {
    pub child: Child,
    pub output_tx: broadcast::Sender<String>,
}

pub async fn spawn_agent(
    backend: &BackendConfig,
    task: &str,
    working_dir: Option<&PathBuf>,
    max_output_lines: usize,
    log_file: Option<PathBuf>,
) -> Result<AgentProcess, Box<dyn std::error::Error + Send + Sync>> {
    let (cmd, args) = backend.build_command(task);
    let mut command = Command::new(&cmd);
    command.args(&args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if let Some(dir) = working_dir { command.current_dir(dir); }

    let mut child = command.spawn()?;
    let (output_tx, _) = broadcast::channel::<String>(1024);
    let line_count = Arc::new(AtomicUsize::new(0));

    let stdout = child.stdout.take().expect("stdout");
    let tx_clone = output_tx.clone();
    let count_clone = line_count.clone();
    let log_clone = log_file.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut log_writer = match log_clone {
            Some(path) => {
                if let Some(parent) = path.parent() { let _ = tokio::fs::create_dir_all(parent).await; }
                tokio::fs::File::create(&path).await.ok()
            }
            None => None,
        };
        while let Ok(Some(line)) = lines.next_line().await {
            if count_clone.fetch_add(1, Ordering::Relaxed) < max_output_lines {
                let _ = tx_clone.send(line.clone());
            }
            if let Some(ref mut f) = log_writer {
                let _ = f.write_all(format!("{}\n", line).as_bytes()).await;
            }
        }
    });

    let stderr = child.stderr.take().expect("stderr");
    let tx2 = output_tx.clone();
    let count2 = line_count.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            if count2.fetch_add(1, Ordering::Relaxed) < max_output_lines {
                let _ = tx2.send(format!("[stderr] {}", line));
            }
        }
    });

    Ok(AgentProcess { child, output_tx })
}

pub async fn run_with_timeout(
    process: &mut AgentProcess,
    max_runtime_seconds: u64,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let timeout = tokio::time::sleep(std::time::Duration::from_secs(max_runtime_seconds));
    tokio::select! {
        status = process.child.wait() => Ok(status?.success()),
        _ = timeout => { process.child.kill().await?; Ok(false) }
    }
}
