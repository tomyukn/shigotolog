use std::error::Error;
use std::io::Write;

use shigotolog::repository::Manipulation;
use shigotolog::sqlite_db::SQLiteDatabase;

use crate::table;

pub fn run(
    db: &SQLiteDatabase,
    show_all: bool,
    mut writer: impl Write,
) -> Result<(), Box<dyn Error>> {
    let mut tasks = db.tasks()?;

    if !show_all {
        tasks = tasks
            .iter()
            .filter(|task| task.is_active)
            .cloned()
            .collect();
    }

    writeln!(writer, "{}", table::task_list(&tasks))?;
    Ok(())
}
