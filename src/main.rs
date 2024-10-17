use rusqlite::Connection;
use std::env;

#[derive(Debug)]
struct Task {
    id: i32,
    task: String,
    done: bool,
}

fn main() -> rusqlite::Result<()> {
    init_db()?;

    let args: Vec<String> = env::args().skip(1).collect();

    let default_command = "help".to_string();
    let command = args.get(0).unwrap_or(&default_command);

    match command.as_str() {
        "add" => {
            let tasks = args.get(1..).unwrap();
            add_task(tasks);
        }
        "edit" => {
            let id = args.get(1).unwrap();
            let task = args.get(2).unwrap();
            edit_task(task, id);
        }
        "list" => {
            list_tasks();
        },
        "done" => {
            let ids = args.get(1..).unwrap();
            done_tasks(ids);
        },
        "rm" => {
            let id = args.get(1).unwrap();
            remove_task(id);
        },
        "" | "help" | _ => {
            println!("{}", HELP);
        }
    }
    Ok(())
}

fn add_task(tasks: &[String]) {
    let conn = Connection::open("./database.sqlite").unwrap();
    for task in tasks {
        conn.execute("INSERT INTO tasks (task, done) VALUES (?1, ?2)", (task, 0))
            .unwrap();
    }
}

fn edit_task(task: &String, id: &String) {
    let conn = Connection::open("./database.sqlite").unwrap();
    conn.execute("UPDATE tasks SET task = ?1 WHERE id = ?2", (task, id))
        .unwrap();
}

fn list_tasks() {
    let conn = Connection::open("./database.sqlite").unwrap();
    let mut stmt = conn.prepare("SELECT id, task, done FROM tasks").unwrap();
    let task_iter = stmt
        .query_map([], |row| {
            Ok(Task {
                id: row.get(0)?,
                task: row.get(1)?,
                done: row.get(2)?,
            })
        })
        .unwrap();
    for task in task_iter {
        let task = task.unwrap();
        let mut text = format!("{}", task.task);
        if task.done {
            text = format!("\x1B[9m{}\x1B[0m", task.task);
        }
        println!("{}. {}", task.id, text);
    }
}

fn done_tasks(ids: &[String]) {
    let conn = Connection::open("./database.sqlite").unwrap();
    for id in ids {
        conn.execute("UPDATE tasks SET done = 1 WHERE id = ?1", (id,))
            .unwrap();
    }
}

fn remove_task(id: &String) {
    let conn = Connection::open("./database.sqlite").unwrap();
    conn.execute("DELETE FROM tasks WHERE id = ?1", (id,))
            .unwrap();
}

fn init_db() -> rusqlite::Result<()> {
    let conn = Connection::open("./database.sqlite")?;
    let mut stmt =
        conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='tasks'")?;
    let table_exists = stmt.exists([])?;

    if !table_exists {
        conn.execute(
            "CREATE TABLE tasks (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              task TEXT NOT NULL,
              done BOOLEAN NOT NULL CHECK (done IN (0, 1))
          );",
            (),
        )?;
    }
    Ok(())
}

const HELP: &str = r#"
Todo is a super fast and simple tasks organizer written in rust
Example: todo list
Available commands:
    - add [TASK/s]
        adds new task/s
        Example: todo add "buy carrots"
    - edit [INDEX] [EDITED TASK/s]
        edits an existing task/s
        Example: todo edit 1 banana
    - list
        lists all tasks
        Example: todo list
    - done [INDEX]
        marks task as done
        Example: todo done 2 3 (marks second and third tasks as completed)
    - rm [INDEX]
        removes a task
        Example: todo rm 4"#;
