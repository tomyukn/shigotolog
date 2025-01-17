use std::error::Error;

use shigotolog::repository::Manipulation;
use shigotolog::sqlite_db::SQLiteDatabase;

use crate::prompt;
use crate::util::map_tasks;

pub fn run(db: &SQLiteDatabase) -> Result<(), Box<dyn Error>> {
    let tasks = db.tasks()?;
    let (mut task_map, task_names) = map_tasks(tasks);
    if let Ok(task_name) = prompt::select(task_names, "Select task") {
        let task = task_map.get_mut(&task_name).unwrap();
        if let Ok(false) = prompt::confirm("Unregister?", false) {
            db.unregister_task(task.id.unwrap())?;
        }
    }
    Ok(())
}
