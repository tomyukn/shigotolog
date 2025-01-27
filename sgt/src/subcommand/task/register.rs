use std::error::Error;

use shigotolog::repository::Manipulation;
use shigotolog::sqlite_db::SQLiteDatabase;
use shigotolog::task::Task;

use crate::prompt;
use crate::util::{map_tasks, push_front};

pub fn run(db: &SQLiteDatabase) -> Result<(), Box<dyn Error>> {
    let tasks = db.tasks()?;
    let (mut task_map, keys) = map_tasks(tasks);

    let candidates = push_front("new".to_string(), keys);
    task_map.insert(candidates[0].clone(), Task::default());

    if let Ok(task_name) = prompt::select(candidates, "Select new or updating task:") {
        let ans_default = task_name == "new";
        let task = task_map.get_mut(&task_name).unwrap();

        if let Ok(true) = prompt::confirm_taskname_input(1, &task.task[0], ans_default) {
            let value = prompt::text_input(">")?;
            task.task[0] = Some(value);
        }

        if let Ok(true) = prompt::confirm_taskname_input(2, &task.task[1], ans_default) {
            let value = prompt::text_input(">")?;
            task.task[1] = Some(value);
        }

        if let Ok(true) = prompt::confirm_taskname_input(3, &task.task[2], ans_default) {
            let value = prompt::text_input(">")?;
            task.task[2] = Some(value);
        }

        if let Ok(true) = prompt::confirm("Set description?", ans_default) {
            let value = prompt::text_input(">")?;
            task.description = value;
        }

        match prompt::confirm("Break time?", task.is_break) {
            Ok(state) => task.is_break = state,
            _ => panic!("Error"),
        }

        match prompt::confirm("Active task?", task.is_active) {
            Ok(state) => task.is_active = state,
            _ => panic!("Error"),
        }

        db.register_task(task)
    } else {
        Ok(())
    }
}
