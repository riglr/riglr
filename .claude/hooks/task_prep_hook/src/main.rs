use serde::Deserialize;
use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use std::process;

const TRIGGER_COMMANDS: &[&str] = &["/solve-complex-problem"];

#[derive(Deserialize)]
struct Input {
    prompt: String,
    cwd: String,
}

fn get_next_instance_id(base_dir: &Path) -> u32 {
    if !base_dir.exists() {
        return 1;
    }

    let max_id = fs::read_dir(base_dir)
        .ok()
        .and_then(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|e| e.path().is_dir())
                .filter_map(|e| {
                    e.file_name()
                        .to_str()
                        .and_then(|s| s.strip_prefix("claude-instance-"))
                        .and_then(|s| s.parse::<u32>().ok())
                })
                .max()
        })
        .unwrap_or(0);

    max_id + 1
}

fn main() {
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        process::exit(0);
    }

    let input: Input = match serde_json::from_str(&buffer) {
        Ok(data) => data,
        Err(_) => process::exit(0), // Not valid JSON, exit silently
    };

    if !TRIGGER_COMMANDS.iter().any(|cmd| input.prompt.trim().starts_with(cmd)) {
        process::exit(0); // Not a trigger command
    }

    let base_dir = Path::new(&input.cwd).join("claude-code-storage");
    let instance_id = get_next_instance_id(&base_dir);
    let instance_dir = base_dir.join(format!("claude-instance-{}", instance_id));

    if fs::create_dir_all(&instance_dir).is_ok() {
        let context_msg = format!(
            "A dedicated workspace has been created for this task at: `{}`. All reports (INVESTIGATION_REPORT.md, FLOW_REPORT.md, PLAN.md) MUST be saved in this directory.",
            instance_dir.display()
        );
        println!("{}", context_msg);
    } else {
        eprintln!("Warning: Failed to create instance directory: {}", instance_dir.display());
    }
}