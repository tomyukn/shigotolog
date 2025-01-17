use std::collections::HashMap;

use chrono::TimeDelta;

use crate::datetime::{TaskTime, WorkingDate};

/// Task
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Task {
    /// Identifier
    pub id: Option<u32>,
    /// Task name (multi part)
    pub task: Vec<Option<String>>,
    /// Descriptin
    pub description: String,
    /// Whether this task is break time or not
    pub is_break: bool,
    /// Whether this task is in use or not
    pub is_active: bool,
}

impl Default for Task {
    fn default() -> Self {
        Self::new(None, None, None, None, "", false, true)
    }
}

impl Task {
    /// Creates a new task.
    pub fn new(
        id: Option<u32>,
        level1: Option<&str>,
        level2: Option<&str>,
        level3: Option<&str>,
        description: &str,
        is_break: bool,
        is_active: bool,
    ) -> Self {
        let task = [level1, level2, level3]
            .iter()
            .map(|o| o.map(|x| x.into()))
            .collect();

        let description = description.to_string();

        Task {
            id,
            task,
            description,
            is_break,
            is_active,
        }
    }

    /// Format multi part task names to one string.
    pub fn format_name(&self, sep: &str) -> String {
        let task = self
            .task
            .iter()
            .flatten()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join(sep);
        task
    }
}

/// Represents a task log.
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct TaskRecord {
    /// Identifier
    pub id: Option<u32>,
    /// Task
    pub task: Task,
    /// Date
    pub working_date: WorkingDate,
    /// Begin time
    pub begin: TaskTime,
    /// End time
    pub end: Option<TaskTime>,
}

impl TaskRecord {
    /// Creates a new task.
    pub fn new(
        id: Option<u32>,
        task: Task,
        working_date: WorkingDate,
        begin: TaskTime,
        end: Option<TaskTime>,
    ) -> Self {
        TaskRecord {
            id,
            task,
            working_date,
            begin,
            end,
        }
    }

    /// Accessor
    pub fn is_break(&self) -> bool {
        self.task.is_break
    }

    /// Calculates duration.
    pub fn duration(&self) -> TimeDelta {
        let begin = &self.begin;
        self.end
            .as_ref()
            .map_or_else(|| &TaskTime::now() - begin, |end| end - begin)
    }
}

/// Summary of tasks.
#[derive(Clone, Debug)]
pub struct TaskSummary {
    /// First begin time of tasks.
    pub begin: TaskTime,
    /// Last end time of tasks
    pub end: Option<TaskTime>,
    /// Total duration
    pub total_duration: TimeDelta,
    /// Durations by task excluding break times
    pub task_durations: HashMap<String, TimeDelta>,
    /// Collected break times
    pub break_times: Vec<TaskRecord>,
}

impl From<&[TaskRecord]> for TaskSummary {
    fn from(value: &[TaskRecord]) -> Self {
        let work_records = value.iter().filter(|record| !record.is_break());

        let begin = work_records
            .clone()
            .map(|record| record.begin.clone())
            .min()
            .unwrap();

        let end = work_records
            .clone()
            .filter_map(|record| record.end.clone())
            .max();

        let total_duration = work_records
            .clone()
            .fold(TimeDelta::zero(), |acc, record| acc + record.duration());

        let mut task_durations = HashMap::<String, TimeDelta>::new();

        for record in work_records {
            let task_name = record.task.format_name("/");
            let task_duration = record.duration();
            if task_durations.contains_key(&task_name) {
                let acc = *task_durations.get(&task_name).unwrap() + task_duration;
                task_durations.insert(task_name, acc);
            } else {
                task_durations.insert(task_name, task_duration);
            }
        }

        let break_times = value
            .iter()
            .filter(|record| record.is_break())
            .cloned()
            .collect::<Vec<_>>();

        TaskSummary {
            begin,
            end,
            total_duration,
            task_durations,
            break_times,
        }
    }
}
