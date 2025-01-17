use std::error::Error;
use std::io::Write;

use shigotolog::datetime::WorkingDate;
use shigotolog::repository::Manipulation;
use shigotolog::sqlite_db::SQLiteDatabase;

use crate::table;

pub fn run(
    db: &SQLiteDatabase,
    date: Option<String>,
    month: Option<String>,
    show_all: bool,
    mut writer: impl Write,
) -> Result<(), Box<dyn Error>> {
    let records = if show_all {
        db.records()?
    } else if let Some(arg_date) = &date {
        db.get_records_by_date(&WorkingDate::parse(arg_date)?)?
    } else if let Some(arg_yearmonth) = &month {
        let (st, en) = WorkingDate::parse_ym(arg_yearmonth)?;
        db.get_records_in_period(&st, &en)?
    } else {
        db.get_records_by_date(&WorkingDate::today())?
    };

    write!(writer, "{}", table::record_list(&records))?;
    if !show_all && month.is_none() {
        let task_summary_table = table::task_summary(&records);
        if !task_summary_table.is_empty() {
            write!(writer, "\n\n Summary\n{}", task_summary_table)?;
        }

        let task_durations_table = table::task_durations(&records);
        if !task_durations_table.is_empty() {
            write!(writer, "\n{}", task_durations_table)?;
        }

        let break_times_table = table::break_times(&records);
        if !break_times_table.is_empty() {
            write!(writer, "\n\n Break\n{}", break_times_table)?;
        }
    } else if month.is_some() {
        write!(writer, "\n\n Summary\n{}", table::task_durations(&records))?;
    }
    Ok(())
}
