use std::error::Error;
use std::io::Write;

use shigotolog::datetime::{TaskTime, TimeDisplay, WorkingDate};
use shigotolog::repository::Manipulation;
use shigotolog::sqlite_db::SQLiteDatabase;

use crate::prompt;
use crate::table;
use crate::util::map_records;

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

    let records = db.get_records_by_date(&date)?;
    let (mut record_map, record_s) = map_records(records);

    if let Ok(record) = prompt::select(record_s, "Select record:") {
        let record = record_map.get_mut(&record).unwrap();

        if let Ok(begin_time) =
            prompt::text_input_with_default("Begin time", &record.begin.to_string_hm())
        {
            record.begin = TaskTime::parse_with_date(&date, &begin_time)?;
        };

        let end = match record.end.clone() {
            Some(time) => time.to_string_hm(),
            None => "".to_string(),
        };
        if let Ok(end_time) = prompt::text_input_with_default("End time", &end) {
            record.end = Some(TaskTime::parse_with_date(&date, &end_time)?);
        };
        db.add_record(record)?;
        // show records
        let records = db.get_records_by_date(&date)?;
        writeln!(writer, "{}", table::record_list(&records))?;
    };
    Ok(())
}
