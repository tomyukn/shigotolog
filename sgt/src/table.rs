use tabled::settings::location::ByColumnName;
use tabled::settings::object::Rows;
use tabled::settings::style::Style;
use tabled::settings::themes::Colorization;
use tabled::settings::{Alignment, Color, Modify};
use tabled::{Table, Tabled};

use shigotolog::datetime::{TaskTime, TimeDisplay};
use shigotolog::task::{Task, TaskRecord, TaskSummary};

/// Basic function that creates a list table
fn build_table<I, T>(rows: I) -> Table
where
    I: IntoIterator<Item = T>,
    T: Tabled,
{
    Table::new(rows)
        .with(Style::sharp())
        .with(Colorization::exact([Color::BOLD], Rows::first()))
        .to_owned()
}

/// Task list table row.
#[derive(Tabled)]
struct TaskRow {
    #[tabled(rename = "Level 1")]
    level1: String,
    #[tabled(rename = "Level 2")]
    level2: String,
    #[tabled(rename = "Level 3")]
    level3: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Break time")]
    #[tabled(display_with = "display_bool")]
    is_break: bool,
    #[tabled(rename = "Active")]
    #[tabled(display_with = "display_bool")]
    is_active: bool,
}

impl From<&Task> for TaskRow {
    fn from(value: &Task) -> Self {
        let level1 = value.task[0].clone().unwrap_or("".into());
        let level2 = value.task[1].clone().unwrap_or("".into());
        let level3 = value.task[2].clone().unwrap_or("".into());

        TaskRow {
            level1,
            level2,
            level3,
            description: value.description.clone(),
            is_break: value.is_break,
            is_active: value.is_active,
        }
    }
}

/// Table output for `bool` value
fn display_bool(x: &bool) -> String {
    match x {
        true => "Yes".into(),
        false => "No".into(),
    }
}

/// Creates a task list table.
pub fn task_list(tasks: &[Task]) -> String {
    let rows = tasks.iter().map(TaskRow::from);
    build_table(rows).to_string()
}

/// Task records table row.
#[derive(Tabled)]
struct TaskRecordRow {
    #[tabled(rename = "Date")]
    date: String,
    #[tabled(rename = "Begin")]
    begin: String,
    #[tabled(rename = "End  ")]
    end: String,
    #[tabled(rename = "Duration")]
    duration: String,
    #[tabled(rename = "Task")]
    task: String,
}

impl From<&TaskRecord> for TaskRecordRow {
    fn from(value: &TaskRecord) -> Self {
        let date = &value.working_date;
        let begin = &value.begin;
        let end = &value.end.as_ref();
        let duration = &end.map_or_else(|| &TaskTime::now() - begin, |end| end - begin);

        Self {
            date: date.to_string(),
            begin: begin.to_string_hm(),
            end: end.map(|end| end.to_string_hm()).unwrap_or("".into()),
            duration: duration.to_string_hm(),
            task: value.task.format_name("/"),
        }
    }
}

/// Creates task records table.
pub fn record_list(records: &[TaskRecord]) -> String {
    if records.is_empty() {
        return "No Records".into();
    }

    let rows = records.iter().map(TaskRecordRow::from);
    build_table(rows)
        .with(Modify::new(ByColumnName::new("Duration")).with(Alignment::right()))
        .to_string()
}

/// Task summary table.
#[derive(Tabled)]
struct TotalDuration {
    #[tabled(rename = "Begin")]
    begin: String,
    #[tabled(rename = "End  ")]
    end: String,
    #[tabled(rename = "Duration")]
    duration: String,
}

impl From<&TaskSummary> for TotalDuration {
    fn from(value: &TaskSummary) -> Self {
        Self {
            begin: value.begin.to_string_hm(),
            end: value.end.clone().map_or("".into(), |t| t.to_string_hm()),
            duration: value.total_duration.to_string_hm(),
        }
    }
}

/// Create task summary table.
pub fn task_summary(records: &[TaskRecord]) -> String {
    if records.is_empty() {
        return "".into();
    }

    let summary = [TaskSummary::from(records)];

    if summary[0].task_durations.is_empty() {
        return "".into();
    }

    let total_duration = summary.iter().map(TotalDuration::from);
    build_table(total_duration)
        .with(Modify::new(ByColumnName::new("Duration")).with(Alignment::right()))
        .to_string()
}

/// Duration by task table
#[derive(Tabled)]
pub struct TaskDuration {
    #[tabled(rename = "Task")]
    task: String,
    #[tabled(rename = "Duration")]
    duration: String,
    #[tabled(rename = "%")]
    percent: String,
}

/// Creates duration by task table.
pub fn task_durations(records: &[TaskRecord]) -> String {
    if records.is_empty() {
        return "".into();
    }

    let summary = TaskSummary::from(records);

    if summary.task_durations.is_empty() {
        return "".into();
    }

    let durations = summary.task_durations.iter().collect::<Vec<_>>();

    let total_time = durations
        .iter()
        .map(|tup| *tup.1)
        .reduce(|acc, dur| acc + dur)
        .unwrap();

    let mut task_durations = durations
        .iter()
        .map(|(task, duration)| TaskDuration {
            task: task.to_string(),
            duration: duration.to_string_hm(),
            percent: format!(
                "{:.1}",
                duration.num_minutes() as f64 / total_time.num_minutes() as f64 * 100.
            ),
        })
        .collect::<Vec<_>>();
    // sort in descending order of duration
    task_durations.sort_by(|a, b| b.duration.cmp(&a.duration));

    build_table(task_durations)
        .with(Modify::new(ByColumnName::new("Duration")).with(Alignment::right()))
        .with(Modify::new(ByColumnName::new("%")).with(Alignment::right()))
        .to_string()
}

/// Brwak time list table
#[derive(Tabled)]
pub struct BreakTimes {
    #[tabled(rename = "Break")]
    task: String,
    #[tabled(rename = "Time")]
    time: String,
}

/// Creates break time list table.
pub fn break_times(records: &[TaskRecord]) -> String {
    if records.is_empty() {
        return "".into();
    }

    let summary = TaskSummary::from(records);

    if summary.break_times.is_empty() {
        return "".into();
    }

    let break_times = summary.break_times.iter().map(|record| BreakTimes {
        task: record.task.format_name("/"),
        time: format!(
            "{} - {}",
            record.begin.to_string_hm(),
            &record
                .end
                .clone()
                .map_or("".to_string(), |t| t.to_string_hm())
        ),
    });

    build_table(break_times).to_string()
}
