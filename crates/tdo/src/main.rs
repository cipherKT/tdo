use clap::Parser;
use engine::Engine;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tdo", about = "A terminal todo manager")]
struct Args {
    #[arg(long, help = "Print today's pending tasks as JSON (for waybar)")]
    today: bool,
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

    if args.today {
        match engine.list_today_tasks() {
            Ok(tasks) => {
                let json = serde_json::json!(
                    tasks
                        .iter()
                        .map(|nt| {
                            serde_json::json!({
                                "name": nt.task.name,
                                "due": nt.task.due_date,
                                "project": nt.project_name,
                                "priority": nt.task.priority
                            })
                        })
                        .collect::<Vec<_>>()
                );
                println!("{}", json);
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
