use crate::datetime::WorkingDate;
use crate::task::{Task, TaskRecord};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Represents the state of `TaskRecord`
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum State {
    Active(TaskRecord),
    Completed,
}

/// define CRUD methods
pub trait Manipulation {
    /// Checks whether the repository is ready
    fn is_ready(&self) -> Result<bool>;

    /// Registers or Updates a specified task.
    fn register_task(&self, task: &Task) -> Result<()>;
    /// Unregisters (deactivate) a task specified by id.
    fn unregister_task(&self, id: u32) -> Result<()>;
    /// Gets all tasks.
    fn tasks(&self) -> Result<Vec<Task>>;
    /// Gets a task specified by id.
    fn get_task(&self, id: u32) -> Result<Task>;

    /// Gets the state of the current record.
    fn current_state(&self, date: &WorkingDate) -> Result<State>;
    /// Creates/updates a record.
    fn add_record(&self, record: &TaskRecord) -> Result<()>;
    /// Deletes a record.
    fn delete_record(&self, id: u32) -> Result<()>;
    /// Gets all records.
    fn records(&self) -> Result<Vec<TaskRecord>>;
    /// Gets records in a specified date.
    fn get_records_by_date(&self, date: &WorkingDate) -> Result<Vec<TaskRecord>>;
    /// Gets records in between the dates.
    fn get_records_in_period(
        &self,
        from: &WorkingDate,
        to: &WorkingDate,
    ) -> Result<Vec<TaskRecord>>;
}
