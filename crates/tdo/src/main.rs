use std::path::PathBuf;

fn db_path() -> PathBuf {
    let home = std::env::var("HOME").expect("HOME not set");
    let dir = PathBuf::from(home).join(".local/share/tdo");
    std::fs::create_dir_all(&dir).expect("failed to create data directory");
    dir.join("todo.db")
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() == 2 && args[1] == "--next_task" {
        match next_task() {
            Ok(Some(nt)) => {
                let due = nt
                    .task
                    .due_date
                    .map(|dt| dt.format("%b %d").to_string())
                    .unwrap_or_default();
                println!("{} ({}) — due {}", nt.task.name, nt.project_name, due);
            }
            Ok(None) => {}
            Err(e) => eprintln!("error: {e}"),
        }
    } else {
        eprintln!("Usage: tdo --next_task");
        std::process::exit(1);
    }
}

fn next_task() -> Result<Option<engine::NextTask>, Box<dyn std::error::Error>> {
    let engine = engine::Engine::open(db_path())?;
    Ok(engine.next_task()?)
}
