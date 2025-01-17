use std::error::Error;
use std::io::Write;

use shigotolog::datetime::{TaskTime, TimeDisplay, WorkingDate};
use shigotolog::repository::{Manipulation, State};
use shigotolog::sqlite_db::SQLiteDatabase;
use shigotolog::task::TaskRecord;

use crate::prompt;
use crate::table;
use crate::util::map_tasks;

pub fn run(
    db: &SQLiteDatabase,
    date: Option<String>,
    mut writer: impl Write,
) -> Result<(), Box<dyn Error>> {
    let date = if let Some(date) = date {
        WorkingDate::parse(&date)?
    } else {
        WorkingDate::today()
    };

    let current_time = TaskTime::now();
    let state = db.current_state(&date)?;
    let tasks = db.tasks()?;
    let (task_map, task_names) = map_tasks(tasks);

    if let Ok(task_name) = prompt::select(task_names, "Select task:") {
        let task = task_map.get(&task_name).unwrap();
        if let Ok(begin_hm) =
            prompt::text_input_with_default("Begin time:", &current_time.to_string_hm())
        {
            let begin = TaskTime::parse_with_date(&date, &begin_hm)?;
            if let State::Active(mut last_record) = state {
                last_record.end = Some(begin.clone());
                db.add_record(&last_record)?;
            }
            let record = TaskRecord::new(None, task.clone(), date.clone(), begin, None);
            db.add_record(&record)?;
            // show records
            let records = db.get_records_by_date(&date)?;
            writeln!(writer, "{}", table::record_list(&records))?;
        }
    }
    Ok(())
}
