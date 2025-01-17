use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime};
use rusqlite::config::DbConfig::SQLITE_DBCONFIG_ENABLE_FKEY;
use rusqlite::{params, Connection};

use crate::datetime::WorkingDate;
use crate::repository::{Manipulation, Result, State};
use crate::task::{Task, TaskRecord};

pub use rusqlite::OpenFlags;

/// Database connection.
pub struct SQLiteDatabase {
    conn: Connection,
}

impl SQLiteDatabase {
    /// Opens a new connection with flags and apply configulations.
    pub fn open<P: AsRef<Path>>(path: P, flags: OpenFlags) -> Result<Self> {
        let conn = Connection::open_with_flags(path, flags)?;
        let db = Self { conn };
        db.setup()?;
        Ok(db)
    }

    /// Opens a new connection in read-only mode.
    pub fn open_r<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
    }

    /// Open a new connection in read/write mode.
    pub fn open_rw<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open(path, OpenFlags::SQLITE_OPEN_READ_WRITE)
    }

    /// Open a new connection in read/write mode. Creates the database if it does not exist.
    pub fn open_rwc<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::open(
            path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE,
        )
    }

    /// Creates tables if they do not exist.
    pub fn initialize(&self) -> Result<()> {
        self.setup()?;
        self.conn.execute_batch(
            "BEGIN;\
            DROP TABLE IF EXISTS tasks;\
            DROP TABLE IF EXISTS records;\
            CREATE TABLE tasks (\
                id INTEGER PRIMARY KEY AUTOINCREMENT,\
                level1 TEXT,\
                level2 TEXT,\
                level3 TEXT,\
                description TEXT,\
                is_break INTEGER,\
                is_active INTEGER\
            );\
            CREATE TABLE records (\
                id INTEGER PRIMARY KEY AUTOINCREMENT,\
                task_id INTEGER,\
                working_date TEXT,\
                begin TEXT,\
                end TEXT,\
                is_break INTEGER,\
                FOREIGN KEY(task_id) REFERENCES tasks(id)\
            );\
            COMMIT;",
        )?;
        Ok(())
    }

    /// Applies configulations to the database.
    fn setup(&self) -> Result<()> {
        let _ = self.conn.set_db_config(SQLITE_DBCONFIG_ENABLE_FKEY, true)?;
        Ok(())
    }
}

impl Manipulation for SQLiteDatabase {
    fn is_ready(&self) -> Result<bool> {
        let table_count = self.conn.query_row(
            "SELECT count(name) \
            FROM sqlite_master \
            WHERE type = 'table' and name in ('tasks', 'records')",
            [],
            |row| row.get::<_, u32>(0),
        )?;

        Ok(table_count == 2)
    }

    fn register_task(&self, task: &Task) -> Result<()> {
        if let Some(id) = task.id {
            self.conn.execute(
                "UPDATE tasks \
                SET level1 = ?1, level2 = ?2, level3 = ?3, description = ?4, is_break = ?5, is_active = ?6 \
                WHERE id = ?7",
                params![
                    task.task[0],
                    task.task[1],
                    task.task[2],
                    task.description,
                    task.is_break as u8,
                    task.is_active as u8,
                    id,
                ],
            )?
        } else {
            self.conn.execute(
                "INSERT INTO tasks (level1, level2, level3, description, is_break, is_active) \
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    task.task[0],
                    task.task[1],
                    task.task[2],
                    task.description,
                    task.is_break as u8,
                    task.is_active as u8,
                ],
            )?
        };
        Ok(())
    }

    fn unregister_task(&self, id: u32) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks \
             SET is_active = 0 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    fn tasks(&self) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, level1, level2, level3, description, is_break, is_active FROM tasks \
            ORDER BY level1, level2, level3",
        )?;

        let rows = stmt.query_map([], |row| {
            let task = Task::new(
                row.get::<_, u32>(0).ok(),
                row.get::<_, String>(1).ok().as_deref(),
                row.get::<_, String>(2).ok().as_deref(),
                row.get::<_, String>(3).ok().as_deref(),
                &row.get::<_, String>(4).unwrap_or_default(),
                row.get::<_, u8>(5).unwrap() != 0,
                row.get::<_, u8>(6).unwrap() != 0,
            );
            Ok(task)
        })?;

        let tasks = rows.flatten().collect();
        Ok(tasks)
    }

    fn get_task(&self, id: u32) -> Result<Task> {
        let task = self.conn.query_row(
            "SELECT level1, level2, level3, description, is_break, is_active FROM tasks \
            WHERE id = ?1",
            params![id],
            |row| {
                let task = Task::new(
                    Some(id),
                    row.get::<_, String>(0).ok().as_deref(),
                    row.get::<_, String>(1).ok().as_deref(),
                    row.get::<_, String>(2).ok().as_deref(),
                    &row.get::<_, String>(3).unwrap_or_default(),
                    row.get::<_, u8>(4).unwrap() != 0,
                    row.get::<_, u8>(5).unwrap() != 0,
                );
                Ok(task)
            },
        )?;

        Ok(task)
    }

    fn current_state(&self, date: &WorkingDate) -> Result<State> {
        let mut stmt = self.conn.prepare(
            "SELECT \
                r.id, r.working_date, r.begin, r.end,\
                t.id, t.level1, t.level2, t.level3, t.description, t.is_break, t.is_active \
            FROM (SELECT * FROM records WHERE working_date = ?1 ORDER BY working_date DESC, begin DESC LIMIT 1) AS r \
            LEFT JOIN tasks AS t \
            ON r.task_id = t.id",
        )?;

        let task_record = stmt.query_map(params![NaiveDate::from(date)], |row| {
            let task = Task::new(
                row.get::<_, u32>(4).ok(),
                row.get::<_, String>(5).ok().as_deref(),
                row.get::<_, String>(6).ok().as_deref(),
                row.get::<_, String>(7).ok().as_deref(),
                &row.get::<_, String>(8).unwrap_or_default(),
                row.get::<_, u8>(9).unwrap() != 0,
                row.get::<_, u8>(10).unwrap() != 0,
            );
            let end_raw = row.get::<_, Option<NaiveDateTime>>(3).unwrap();
            let record = TaskRecord::new(
                row.get::<_, u32>(0).ok(),
                task,
                row.get::<_, NaiveDate>(1).unwrap().into(),
                row.get::<_, NaiveDateTime>(2).unwrap().into(),
                end_raw.map(|t| t.into()),
            );
            Ok(record)
        })?;

        let task_records = task_record.flatten().collect::<Vec<_>>();

        if task_records.is_empty() {
            return Ok(State::Completed);
        }

        let task_record = task_records[0].clone();

        match task_record.end {
            Some(_) => Ok(State::Completed),
            None => Ok(State::Active(task_record)),
        }
    }

    fn add_record(&self, record: &TaskRecord) -> Result<()> {
        if let Some(id) = record.id {
            self.conn.execute(
                "UPDATE records \
                SET task_id = ?1, working_date = ?2, begin = ?3, end = ?4 \
                WHERE id = ?5",
                params![
                    record.task.id,
                    NaiveDate::from(&record.working_date),
                    NaiveDateTime::from(record.begin.clone()),
                    record.end.clone().map(NaiveDateTime::from),
                    id,
                ],
            )?;
        } else {
            self.conn.execute(
                "INSERT INTO records (task_id, working_date, begin, end) \
                VALUES (?1, ?2, ?3, ?4)",
                params![
                    record.task.id,
                    NaiveDate::from(&record.working_date),
                    NaiveDateTime::from(record.begin.clone()),
                    record.end.clone().map(NaiveDateTime::from),
                ],
            )?;
        }
        Ok(())
    }

    fn delete_record(&self, id: u32) -> Result<()> {
        self.conn
            .execute("DELETE FROM records WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn records(&self) -> Result<Vec<TaskRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT \
                r.id, r.working_date, r.begin, r.end,\
                t.id, t.level1, t.level2, t.level3, t.description, t.is_break, t.is_active \
            FROM records AS r \
            LEFT JOIN tasks AS t \
            ON r.task_id = t.id \
            ORDER BY working_date, begin",
        )?;

        let rows = stmt.query_map([], |row| {
            let task = Task::new(
                row.get::<_, u32>(4).ok(),
                row.get::<_, String>(5).ok().as_deref(),
                row.get::<_, String>(6).ok().as_deref(),
                row.get::<_, String>(7).ok().as_deref(),
                &row.get::<_, String>(8).unwrap_or_default(),
                row.get::<_, u8>(9).unwrap() != 0,
                row.get::<_, u8>(10).unwrap() != 0,
            );
            let end_raw = row.get::<_, Option<NaiveDateTime>>(3).unwrap();
            let record = TaskRecord::new(
                row.get::<_, u32>(0).ok(),
                task,
                row.get::<_, NaiveDate>(1).unwrap().into(),
                row.get::<_, NaiveDateTime>(2).unwrap().into(),
                end_raw.map(|t| t.into()),
            );
            Ok(record)
        })?;

        let records = rows.flatten().collect();
        Ok(records)
    }

    fn get_records_by_date(&self, date: &WorkingDate) -> Result<Vec<TaskRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT \
                r.id, r.working_date, r.begin, r.end,\
                t.id, t.level1, t.level2, t.level3, t.description, t.is_break, t.is_active \
            FROM (SELECT * FROM records WHERE working_date = ?1) AS r \
            LEFT JOIN tasks AS t \
            ON r.task_id = t.id \
            ORDER BY working_date, begin",
        )?;

        let rows = stmt.query_map(params![NaiveDate::from(date)], |row| {
            let task = Task::new(
                row.get::<_, u32>(4).ok(),
                row.get::<_, String>(5).ok().as_deref(),
                row.get::<_, String>(6).ok().as_deref(),
                row.get::<_, String>(7).ok().as_deref(),
                &row.get::<_, String>(8).unwrap_or_default(),
                row.get::<_, u8>(9).unwrap() != 0,
                row.get::<_, u8>(10).unwrap() != 0,
            );
            let end_raw = row.get::<_, Option<NaiveDateTime>>(3).unwrap();
            let record = TaskRecord::new(
                row.get::<_, u32>(0).ok(),
                task,
                row.get::<_, NaiveDate>(1).unwrap().into(),
                row.get::<_, NaiveDateTime>(2).unwrap().into(),
                end_raw.map(|t| t.into()),
            );
            Ok(record)
        })?;

        let records = rows.flatten().collect();
        Ok(records)
    }

    fn get_records_in_period(
        &self,
        from: &WorkingDate,
        to: &WorkingDate,
    ) -> Result<Vec<TaskRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT \
                r.id, r.working_date, r.begin, r.end,\
                t.id, t.level1, t.level2, t.level3, t.description, t.is_break, t.is_active \
            FROM (SELECT * FROM records WHERE working_date BETWEEN ?1 AND ?2) AS r \
            LEFT JOIN tasks AS t \
            ON r.task_id = t.id \
            ORDER BY working_date, begin",
        )?;

        let rows = stmt.query_map(params![NaiveDate::from(from), NaiveDate::from(to)], |row| {
            let task = Task::new(
                row.get::<_, u32>(4).ok(),
                row.get::<_, String>(5).ok().as_deref(),
                row.get::<_, String>(6).ok().as_deref(),
                row.get::<_, String>(7).ok().as_deref(),
                &row.get::<_, String>(8).unwrap_or_default(),
                row.get::<_, u8>(9).unwrap() != 0,
                row.get::<_, u8>(10).unwrap() != 0,
            );
            let end_raw = row.get::<_, Option<NaiveDateTime>>(3).unwrap();
            let record = TaskRecord::new(
                row.get::<_, u32>(0).ok(),
                task,
                row.get::<_, NaiveDate>(1).unwrap().into(),
                row.get::<_, NaiveDateTime>(2).unwrap().into(),
                end_raw.map(|t| t.into()),
            );
            Ok(record)
        })?;

        let records = rows.flatten().collect();
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datetime::{TaskTime, WorkingDate};
    use std::error::Error;
    use std::result::Result;

    fn prep_db() -> Result<SQLiteDatabase, Box<dyn Error>> {
        let conn = Connection::open_in_memory()?;
        let db = SQLiteDatabase { conn };
        db.initialize()?;
        Ok(db)
    }

    #[test]
    #[rustfmt::skip]
    fn test_task_register() -> Result<(), Box<dyn Error>> {
        let db = prep_db()?;
        let task1 = Task::new(None, Some("aaa"), Some("xxx"), None, "", false, true);
        let task2 = Task::new(None, Some("bbb"), Some("yyy"), Some("123"), "", false, true);
        db.register_task(&task1)?;
        db.register_task(&task2)?;

        let tasks = db.tasks()?;
        let expected = vec![
            Task::new(Some(1), Some("aaa"), Some("xxx"), None, "", false, true),
            Task::new(Some(2), Some("bbb"), Some("yyy"), Some("123"), "", false, true),
        ];

        assert_eq!(tasks, expected);
        Ok(())
    }

    #[test]
    #[rustfmt::skip]
    fn test_task_unregister() -> Result<(), Box<dyn Error>> {
        let db = prep_db()?;
        let task1 = Task::new(None, Some("aaa"), Some("xxx"), None, "", false, true);
        let task2 = Task::new(None, Some("bbb"), Some("yyy"), Some("123"), "", false, true);
        db.register_task(&task1)?;
        db.register_task(&task2)?;
        db.unregister_task(1)?;

        let tasks = db.tasks()?;
        let expected = vec![
            Task::new(Some(1), Some("aaa"), Some("xxx"), None, "", false, false),
            Task::new(Some(2), Some("bbb"), Some("yyy"), Some("123"), "", false, true),
        ];

        assert_eq!(tasks, expected);
        Ok(())
    }

    #[test]
    #[rustfmt::skip]
    fn test_get_task() -> Result<(), Box<dyn Error>> {
        let db = prep_db()?;
        let task1 = Task::new(None, Some("aaa"), Some("xxx"), None, "", false, true);
        let task2 = Task::new(None, Some("bbb"), Some("yyy"), Some("123"), "", false, true);
        db.register_task(&task1)?;
        db.register_task(&task2)?;

        let task = db.get_task(1)?;
        let expected = Task::new(Some(1), Some("aaa"), Some("xxx"), None, "", false, true);
        assert_eq!(task, expected);

        let task = db.get_task(2)?;
        let expected= Task::new(Some(2), Some("bbb"), Some("yyy"), Some("123"), "", false, true);
        assert_eq!(task, expected);
        Ok(())
    }

    #[test]
    fn test_add_record() -> Result<(), Box<dyn Error>> {
        let task = Task::new(None, Some("aaa"), Some("xxx"), None, "", false, true);
        let db = prep_db()?;
        db.register_task(&task)?;

        let begin = TaskTime::parse("2021-01-01T09:00:00")?;
        let date = begin.clone().into();
        let record = TaskRecord::new(None, task, date, begin, None);
        db.add_record(&record)?;
        Ok(())
    }

    #[test]
    fn test_delete_record() -> Result<(), Box<dyn Error>> {
        let task = Task::new(Some(1), Some("aaa"), Some("xxx"), None, "", false, true);
        let db = prep_db()?;
        db.register_task(&task)?;

        let begin = TaskTime::parse("2021-01-01T09:00:00")?;
        let end = TaskTime::parse("2021-01-01T17:00:00")?;
        let date = begin.clone().into();
        let record = TaskRecord::new(Some(10), task, date, begin, Some(end));
        db.add_record(&record)?;
        db.delete_record(10)?;
        Ok(())
    }

    #[test]
    fn test_records() -> Result<(), Box<dyn Error>> {
        let db = prep_db()?;
        db.conn.execute(
            "INSERT INTO tasks (level1, level2, level3, description, is_break, is_active) \
            VALUES ('a', 'b', 'c', 'd', 0, 1), ('e', 'f', 'g', 'h', 0, 1)",
            [],
        )?;
        db.conn.execute(
            "INSERT INTO records (task_id, working_date, begin, end, is_break) \
            VALUES \
                (1, '2021-01-01', '2021-01-01 09:00:00', '2021-01-01 12:00:00', 0),\
                (2, '2021-01-01', '2021-01-01 13:00:00', '2021-01-01 17:30:00', 0),\
                (1, '2021-01-02', '2021-01-02 09:00:00', '2021-01-02 15:00:00', 0),\
                (2, '2021-01-02', '2021-01-02 15:00:00', NULL, 0),\
                (1, '2021-01-03', '2021-01-03 09:00:00', '2021-01-03 17:30:00', 0)",
            [],
        )?;
        let task1 = Task::new(Some(1), Some("a"), Some("b"), Some("c"), "d", false, true);
        let task2 = Task::new(Some(2), Some("e"), Some("f"), Some("g"), "h", false, true);

        let date1 = WorkingDate::parse("2021-01-01")?;
        let date2 = WorkingDate::parse("2021-01-02")?;
        let date3 = WorkingDate::parse("2021-01-03")?;

        let record1 = TaskRecord::new(
            Some(1),
            task1.clone(),
            date1.clone(),
            TaskTime::parse("2021-01-01T09:00:00").unwrap(),
            Some(TaskTime::parse("2021-01-01T12:00:00").unwrap()),
        );
        let record2 = TaskRecord::new(
            Some(2),
            task2.clone(),
            date1.clone(),
            TaskTime::parse("2021-01-01T13:00:00").unwrap(),
            Some(TaskTime::parse("2021-01-01T17:30:00").unwrap()),
        );
        let record3 = TaskRecord::new(
            Some(3),
            task1.clone(),
            date2.clone(),
            TaskTime::parse("2021-01-02T09:00:00").unwrap(),
            Some(TaskTime::parse("2021-01-02T15:00:00").unwrap()),
        );
        let record4 = TaskRecord::new(
            Some(4),
            task2.clone(),
            date2.clone(),
            TaskTime::parse("2021-01-02T15:00:00").unwrap(),
            None,
        );
        let record5 = TaskRecord::new(
            Some(5),
            task1.clone(),
            date3.clone(),
            TaskTime::parse("2021-01-03T09:00:00").unwrap(),
            Some(TaskTime::parse("2021-01-03T17:30:00").unwrap()),
        );

        let result = db.records()?;
        let expected = vec![record1, record2, record3, record4, record5];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_get_records_by_date() -> Result<(), Box<dyn Error>> {
        let db = prep_db()?;
        db.conn.execute(
            "INSERT INTO tasks (level1, level2, level3, description, is_break, is_active) \
            VALUES ('a', 'b', 'c', 'd', 0, 1), ('e', 'f', 'g', 'h', 0, 1)",
            [],
        )?;
        db.conn.execute(
            "INSERT INTO records (task_id, working_date, begin, end, is_break) \
            VALUES \
                (1, '2021-01-01', '2021-01-01 09:00:00', '2021-01-01 12:00:00', 0),\
                (2, '2021-01-01', '2021-01-01 13:00:00', '2021-01-01 17:30:00', 0),\
                (1, '2021-01-02', '2021-01-02 09:00:00', '2021-01-02 15:00:00', 0),\
                (2, '2021-01-02', '2021-01-02 15:00:00', NULL, 0),\
                (1, '2021-01-03', '2021-01-03 09:00:00', '2021-01-03 17:30:00', 0)",
            [],
        )?;
        let task1 = Task::new(Some(1), Some("a"), Some("b"), Some("c"), "d", false, true);
        let task2 = Task::new(Some(2), Some("e"), Some("f"), Some("g"), "h", false, true);

        let date2 = WorkingDate::parse("2021-01-02")?;

        let record3 = TaskRecord::new(
            Some(3),
            task1.clone(),
            date2.clone(),
            TaskTime::parse("2021-01-02T09:00:00").unwrap(),
            Some(TaskTime::parse("2021-01-02T15:00:00").unwrap()),
        );
        let record4 = TaskRecord::new(
            Some(4),
            task2.clone(),
            date2.clone(),
            TaskTime::parse("2021-01-02T15:00:00").unwrap(),
            None,
        );

        let result = db.get_records_by_date(&WorkingDate::parse("2021-01-02").unwrap())?;
        let expected = vec![record3, record4];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_get_records_in_period() -> Result<(), Box<dyn Error>> {
        let db = prep_db()?;
        db.conn.execute(
            "INSERT INTO tasks (level1, level2, level3, description, is_break, is_active) \
            VALUES ('a', 'b', 'c', 'd', 0, 1), ('e', 'f', 'g', 'h', 0, 1)",
            [],
        )?;
        db.conn.execute(
            "INSERT INTO records (task_id, working_date, begin, end, is_break) \
            VALUES \
                (1, '2021-01-01', '2021-01-01 09:00:00', '2021-01-01 12:00:00', 0),\
                (2, '2021-01-02', '2021-01-02 13:00:00', '2021-01-02 17:30:00', 0),\
                (1, '2021-01-03', '2021-01-03 09:00:00', '2021-01-03 15:00:00', 0),\
                (2, '2021-01-04', '2021-01-04 15:00:00', NULL, 0),\
                (1, '2021-01-05', '2021-01-05 09:00:00', '2021-01-05 17:30:00', 0)",
            [],
        )?;
        let task1 = Task::new(Some(1), Some("a"), Some("b"), Some("c"), "d", false, true);
        let task2 = Task::new(Some(2), Some("e"), Some("f"), Some("g"), "h", false, true);

        let date2 = WorkingDate::parse("2021-01-02")?;
        let date3 = WorkingDate::parse("2021-01-03")?;

        let record2 = TaskRecord::new(
            Some(2),
            task2.clone(),
            date2.clone(),
            TaskTime::parse("2021-01-02T13:00:00").unwrap(),
            Some(TaskTime::parse("2021-01-02T17:30:00").unwrap()),
        );
        let record3 = TaskRecord::new(
            Some(3),
            task1.clone(),
            date3.clone(),
            TaskTime::parse("2021-01-03T09:00:00").unwrap(),
            Some(TaskTime::parse("2021-01-03T15:00:00").unwrap()),
        );

        let result = db.get_records_in_period(&date2, &date3)?;
        let expected = vec![record2, record3];
        assert_eq!(result, expected);
        Ok(())
    }

    #[test]
    fn test_current_state_active() -> Result<(), Box<dyn Error>> {
        let db = prep_db()?;
        db.conn.execute(
            "INSERT INTO tasks (level1, level2, level3, description, is_break, is_active) \
            VALUES ('a', 'b', 'c', 'd', 0, 1), ('e', 'f', 'g', 'h', 0, 1)",
            [],
        )?;
        db.conn.execute(
            "INSERT INTO records (task_id, working_date, begin, end, is_break) \
            VALUES \
                (1, '2021-01-01', '2021-01-01 09:00:00', '2021-01-01 12:00:00', 0),\
                (2, '2021-01-01', '2021-01-01 13:00:00', NULL, 0)",
            [],
        )?;
        let task = Task::new(Some(2), Some("e"), Some("f"), Some("g"), "h", false, true);
        let date = WorkingDate::parse("2021-01-01")?;
        let record = TaskRecord::new(
            Some(2),
            task,
            date,
            TaskTime::parse("2021-01-01T13:00:00").unwrap(),
            None,
        );
        assert_eq!(
            db.current_state(&WorkingDate::parse("2021-01-01").unwrap())?,
            State::Active(record)
        );
        Ok(())
    }

    #[test]
    fn test_current_state_completed() -> Result<(), Box<dyn Error>> {
        let db = prep_db()?;
        db.conn.execute(
            "INSERT INTO tasks (level1, level2, level3, description, is_break, is_active) \
            VALUES ('a', 'b', 'c', 'd', 0, 1), ('e', 'f', 'g', 'h', 0, 1)",
            [],
        )?;
        db.conn.execute(
            "INSERT INTO records (task_id, working_date, begin, end, is_break) \
            VALUES \
                (1, '2021-01-01', '2021-01-01 09:00:00', '2021-01-01 12:00:00', 0), \
                (2, '2021-01-01', '2021-01-01 13:00:00', '2021-01-01 17:30:00', 0)",
            [],
        )?;
        assert_eq!(
            db.current_state(&WorkingDate::parse("2021-01-01").unwrap())?,
            State::Completed
        );
        Ok(())
    }
}
