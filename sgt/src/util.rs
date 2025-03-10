use std::collections::HashMap;

use shigotolog::datetime::TimeDisplay;
use shigotolog::task::{Task, TaskRecord};

/// Creates a map that is used as a lookup table for tasks.
///
/// The key is a string that combines the task levels and description.
pub fn map_tasks(tasks: Vec<Task>) -> (HashMap<String, Task>, Vec<String>) {
    let mut map = HashMap::new();
    let mut keys = vec![];

    for task in tasks {
        let mut key = task.format_name("/");
        if !task.description.is_empty() {
            key += &format!(" - {}", &task.description)
        }

        map.insert(key.clone(), task);
        keys.push(key);
    }
    (map, keys)
}

pub fn push_front<T>(x: T, v: Vec<T>) -> Vec<T> {
    let mut result = vec![x];
    result.extend(v);
    result
}

pub fn map_records(records: Vec<TaskRecord>) -> (HashMap<String, TaskRecord>, Vec<String>) {
    let mut map = HashMap::new();
    let mut keys = vec![];

    for record in records {
        let key = format!(
            "{}  {} - {:5}  {}",
            record.working_date,
            record.begin.to_string_hm(),
            record
                .end
                .clone()
                .map_or_else(|| "".to_string(), |t| t.to_string_hm()),
            record.task.format_name("/")
        );
        map.insert(key.clone(), record);
        keys.push(key);
    }
    (map, keys)
}
