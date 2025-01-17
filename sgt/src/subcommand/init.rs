use std::error::Error;
use std::io::Write;

use shigotolog::sqlite_db::SQLiteDatabase;

use crate::database::initialize_tables;

pub fn run(db: &SQLiteDatabase, writer: impl Write) -> Result<(), Box<dyn Error>> {
    initialize_tables(db, writer)
}
