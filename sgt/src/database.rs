use std::error::Error;
use std::io::Write;

use directories::ProjectDirs;

use shigotolog::sqlite_db::SQLiteDatabase;

/// Creates a database.
pub fn setup_db(
    app_name: &str,
    mut writer: impl Write,
) -> Result<std::path::PathBuf, Box<dyn Error>> {
    let proj_dirs = ProjectDirs::from("", "", app_name).ok_or("Unable to crate data directory")?;
    let data_dir = proj_dirs.data_dir();
    let db_path = &data_dir.join(format!("{}.db", app_name));

    if !data_dir.exists() {
        std::fs::create_dir_all(data_dir)?;
    }

    if !db_path.exists() {
        let db = SQLiteDatabase::open_rwc(db_path)?;

        writeln!(
            &mut writer,
            "Database created: {}",
            db_path.to_string_lossy()
        )?;

        initialize_tables(&db, &mut writer)?;
    }

    Ok(db_path.to_owned())
}

/// Creates tables in the database.
pub fn initialize_tables(
    db: &SQLiteDatabase,
    mut writer: impl Write,
) -> Result<(), Box<dyn Error>> {
    write!(writer, "Initializing database... ")?;
    db.initialize()?;
    writeln!(writer, "Done.")?;

    Ok(())
}
