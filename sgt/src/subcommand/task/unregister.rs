use std::error::Error;

use shigotolog::repository::Manipulation;
use shigotolog::sqlite_db::SQLiteDatabase;

use crate::prompt;
use crate::util::map_tasks;

pub fn run(db: &SQLiteDatabase) -> Result<(), Box<dyn Error>> {
    let tasks = db.tasks()?;
    let (mut task_map, keys) = map_tasks(tasks);
    if let Ok(key) = prompt::select(keys, "Select task") {
        let task = task_map.get_mut(&key).unwrap();
        if let Ok(false) = prompt::confirm("Unregister?", false) {
            db.unregister_task(task.id.unwrap())?;
        }
    }
    Ok(())
}
