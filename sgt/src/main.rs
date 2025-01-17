use std::io::{stderr, stdout};

use clap::{Args, Parser, Subcommand};

use shigotolog::sqlite_db::SQLiteDatabase;

use sgt::database::setup_db;
use sgt::prompt;
use sgt::subcommand;

/// ShigotoLog CLI
#[derive(Debug, Parser)]
#[command(name = "sgt")]
#[command(version, about, long_about = None)]
#[command(flatten_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Initialize database
    Init,
    /// Manipulate a task
    #[command(flatten_help = true)]
    Task(TaskArgs),
    /// Start task
    #[command(visible_alias = "s")]
    Start(StartArgs),
    /// End task
    #[command(visible_alias = "e")]
    End(EndArgs),
    /// Fix time
    Fix(FixArgs),
    /// Print records
    Log(LogArgs),
}

#[derive(Debug, Args)]
#[command(args_conflicts_with_subcommands = true)]
struct TaskArgs {
    #[command(subcommand)]
    command: TaskCommands,
}

#[derive(Debug, Subcommand)]
enum TaskCommands {
    /// Register or update a task
    Register,
    /// Unregister a task
    Unregister,
    /// List active tasks
    Ls(LsArgs),
}

#[derive(Debug, Args)]
struct StartArgs {
    /// Specify target date
    #[arg(short, long, value_name = "DATE")]
    date: Option<String>,
}

#[derive(Debug, Args)]
struct EndArgs {
    /// Specify target date
    #[arg(short, long, value_name = "DATE")]
    date: Option<String>,
}

#[derive(Debug, Args)]
struct FixArgs {
    /// Specify target date
    #[arg(short, long, value_name = "DATE")]
    date: Option<String>,
}

#[derive(Debug, Args)]
struct LogArgs {
    /// Print all records
    #[arg(short, long, conflicts_with("date"))]
    all: bool,
    /// Print records with the specified date
    #[arg(short, long, value_name = "DATE", conflicts_with("month"))]
    date: Option<String>,
    /// Print records with the specified month
    #[arg(short, long, value_name = "MONTH", conflicts_with("all"))]
    month: Option<String>,
}

#[derive(Debug, Args)]
struct LsArgs {
    /// Print all tasks
    #[arg(short, long)]
    all: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_path = setup_db("shigotolog", stderr())?;

    let args = Cli::parse();
    match args.command {
        Commands::Init => {
            if let Ok(true) = prompt::confirm_init() {
                let db = SQLiteDatabase::open_rwc(&db_path)?;
                subcommand::init::run(&db, std::io::stderr())?;
            }
        }
        Commands::Task(task) => {
            let task_cmd = task.command;
            match task_cmd {
                TaskCommands::Register => {
                    let db = SQLiteDatabase::open_rw(&db_path)?;
                    subcommand::task::register::run(&db)?;
                }
                TaskCommands::Unregister => {
                    let db = SQLiteDatabase::open_rw(&db_path)?;
                    subcommand::task::unregister::run(&db)?;
                }
                TaskCommands::Ls(args) => {
                    let db = SQLiteDatabase::open_r(&db_path)?;
                    subcommand::task::ls::run(&db, args.all, stdout())?;
                }
            }
        }
        Commands::Start(args) => {
            let db = SQLiteDatabase::open_rw(&db_path)?;
            subcommand::start::run(&db, args.date, stdout())?;
        }
        Commands::End(args) => {
            let db = SQLiteDatabase::open_rw(&db_path)?;
            subcommand::end::run(&db, args.date, stdout())?;
        }
        Commands::Fix(args) => {
            let db = SQLiteDatabase::open_rw(&db_path)?;
            subcommand::fix::run(&db, args.date, stdout())?;
        }
        Commands::Log(args) => {
            let db = SQLiteDatabase::open_r(&db_path)?;
            subcommand::log::run(&db, args.date, args.month, args.all, stdout())?;
        }
    }

    Ok(())
}
