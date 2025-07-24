use clap::Parser;
use std::path::PathBuf;
use super::commands::SubCommand;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// The directory where the .sql files are located.
    #[clap(short, long)]
    pub web_root: Option<PathBuf>,
    /// The directory where the sqlpage.json configuration, the templates, and the migrations are located.
    #[clap(short = 'd', long)]
    pub config_dir: Option<PathBuf>,
    /// The path to the configuration file.
    #[clap(short = 'c', long)]
    pub config_file: Option<PathBuf>,

    /// Subcommands for additional functionality.
    #[clap(subcommand)]
    pub command: Option<SubCommand>,
}

pub fn parse_cli() -> anyhow::Result<Cli> {
    let cli = Cli::parse();
    Ok(cli)
}

#[test]
fn test_cli_argument_parsing() {
    let cli = Cli::parse_from([
        "sqlpage",
        "--web-root",
        "/path/to/web",
        "--config-dir",
        "/path/to/config",
        "--config-file",
        "/path/to/config.json",
    ]);

    assert_eq!(cli.web_root, Some(PathBuf::from("/path/to/web")));
    assert_eq!(cli.config_dir, Some(PathBuf::from("/path/to/config")));
    assert_eq!(cli.config_file, Some(PathBuf::from("/path/to/config.json")));
}
