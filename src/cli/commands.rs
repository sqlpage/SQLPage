use clap::Parser;
use std::path::Path;
use chrono::Utc;

use crate::app_config::AppConfig;

/// Sub-commands for the sqlpage CLI.
/// Each subcommand can be executed using the `sqlpage <subcommand name>` from the command line.
#[derive(Parser)]
pub enum SubCommand {
    /// Create a new migration file.
    CreateMigration {
        /// Name of the migration.
        migration_name: String,
    },
}

impl SubCommand {
    /// Execute the subcommand.
    pub async fn execute(&self, app_config: AppConfig) -> anyhow::Result<()> {
        match self {
            SubCommand::CreateMigration { migration_name } => {
                // Pass configuration_directory from app_config
                create_migration_file(
                    migration_name,
                    &app_config.configuration_directory,
                ).await?;
                Ok(())
            }
        }
    }
}

async fn create_migration_file(
    migration_name: &str,
    configuration_directory: &Path,
) -> anyhow::Result<()> {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S").to_string();
    let snake_case_name = migration_name
        .replace(|c: char| !c.is_alphanumeric(), "_")
        .to_lowercase();
    let file_name = format!("{timestamp}_{snake_case_name}.sql");
    let migrations_dir = Path::new(configuration_directory).join("migrations");

    if !migrations_dir.exists() {
        tokio::fs::create_dir_all(&migrations_dir).await?;
    }

    let mut unique_file_name = file_name.clone();
    let mut counter = 1;

    while migrations_dir.join(&unique_file_name).exists() {
        unique_file_name = format!("{timestamp}_{snake_case_name}_{counter}.sql");
        counter += 1;
    }

    let file_path = migrations_dir.join(unique_file_name);
    tokio::fs::write(&file_path, "-- Write your migration here\n").await?;

    // the following code cleans up the display path to show where the migration was created
    // relative to the current working directory, and then outputs the path to the migration
    let file_path_canon = file_path.canonicalize().unwrap_or(file_path.clone());
    let cwd_canon = std::env::current_dir()?
        .canonicalize()
        .unwrap_or(std::env::current_dir()?);
    let rel_path = match file_path_canon.strip_prefix(&cwd_canon) {
        Ok(p) => p,
        Err(_) => file_path_canon.as_path(),
    };
    let mut display_path_str = rel_path.display().to_string();
    if display_path_str.starts_with("\\\\?\\") {
        display_path_str = display_path_str.trim_start_matches("\\\\?\\").to_string();
    }
    display_path_str = display_path_str.replace('\\', "/");
    println!("Migration file created: {display_path_str}");
    Ok(())
}
