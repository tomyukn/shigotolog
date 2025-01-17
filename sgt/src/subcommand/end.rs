use std::error::Error;
use std::io::Write;

use shigotolog::datetime::{TaskTime, TimeDisplay, WorkingDate};
use shigotolog::repository::{Manipulation, State};
use shigotolog::sqlite_db::SQLiteDatabase;

use crate::prompt;
use crate::table;

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

    if let State::Active(mut last_record) = state {
        if let Ok(end_hm) =
            prompt::text_input_with_default("End time", &current_time.to_string_hm())
        {
            let end = TaskTime::parse_with_date(&date, &end_hm)?;
            if last_record.begin > end {
                panic!("end time is earlier than start time")
            }
            last_record.end = Some(end);
            db.add_record(&last_record)?;
            // show records
            let records = db.get_records_by_date(&date)?;
            writeln!(writer, "{}", table::record_list(&records))?;
        }
    }
    Ok(())
}
