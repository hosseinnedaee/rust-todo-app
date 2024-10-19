use rusqlite::Connection;
use std::env;
use std::rc::Rc;

#[derive(Debug)]
struct Task {
    id: i32,
    task: String,
    done: bool,
}

struct App {
    db: Rc<Connection>,
}

impl App {
    fn new() -> Self {
        let conn = Rc::new(Connection::open("./database.sqlite").expect("Cannot connect to database."));
        {
            let conn = conn.clone();
            let mut stmt =
                conn.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name='tasks'").expect("Cannot find tasks table.");
            let table_exists = stmt.exists([]).unwrap();

            if !table_exists {
                conn.execute(
                    "CREATE TABLE tasks (
                  id INTEGER PRIMARY KEY AUTOINCREMENT,
                  task TEXT NOT NULL,
                  done BOOLEAN NOT NULL CHECK (done IN (0, 1))
              );",
                    (),
                ).expect("Cannot create tasks table.");
            }
        }
        Self { db: conn }
    }

    fn add_task(&self, tasks: &[String]) {
        for task in tasks {
            self.db
                .execute("INSERT INTO tasks (task, done) VALUES (?1, ?2)", (task, 0))
                .unwrap();
        }
    }

    fn edit_task(&self, task: &String, id: &String) {
        self.db
            .execute("UPDATE tasks SET task = ?1 WHERE id = ?2", (task, id))
            .unwrap();
    }

    fn list_tasks(&self) {
        let mut stmt = self.db.prepare("SELECT id, task, done FROM tasks").unwrap();
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

    fn done_tasks(&self, ids: &[String]) {
        for id in ids {
            self.db
                .execute("UPDATE tasks SET done = 1 WHERE id = ?1", (id,))
                .unwrap();
        }
    }

    fn remove_task(&self, id: &String) {
        self.db
            .execute("DELETE FROM tasks WHERE id = ?1", (id,))
            .unwrap();
    }
}

fn main() -> rusqlite::Result<()> {
    let app = App::new();
    let args: Vec<String> = env::args().skip(1).collect();

    let default_command = "help".to_string();
    let command = args.get(0).unwrap_or(&default_command);

    match command.as_str() {
        "add" => {
            let tasks = args.get(1..).unwrap();
            app.add_task(tasks);
        }
        "edit" => {
            let id = args.get(1).unwrap();
            let task = args.get(2).unwrap();
            app.edit_task(task, id);
        }
        "list" => {
            app.list_tasks();
        }
        "done" => {
            let ids = args.get(1..).unwrap();
            app.done_tasks(ids);
        }
        "rm" => {
            let id = args.get(1).unwrap();
            app.remove_task(id);
        }
        "" | "help" | _ => {
            println!("{}", HELP);
        }
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
