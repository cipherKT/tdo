use clap::Parser;
use engine::Engine;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tdo", about = "A terminal todo manager")]
struct Args {
    #[arg(long, help = "Print the next pending task as JSON (for waybar)")]
    next_task: bool,
}

fn db_path() -> PathBuf {
    let base = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(base).join(".local/share/tdo/tdo.db")
}

fn main() {
    let args = Args::parse();

    let path = db_path();

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("Failed to create data directory");
    }

    let engine = Engine::open(&path).expect("Failed to open database");

    if args.next_task {
        match engine.next_task() {
            Ok(Some(nt)) => {
                let json = serde_json::json!({
                    "name": nt.task.name,
                    "due": nt.task.due_date,
                    "project": nt.project_name
                });
                println!("{}", json);
            }
            Ok(None) => {
                // no pending task, waybar thinks all clear
            }
            Err(e) => {
                eprintln!("Error: {:?}", e);
                std::process::exit(1);
            }
        }
        return;
    }

    tdo_tui::run(engine).expect("Failed to run TUI");
}
