use schemars::generate::SchemaSettings;
use sqlpage::app_config::AppConfig;
use std::io::{self, Write};

fn main() -> anyhow::Result<()> {
    let settings =
        SchemaSettings::draft2020_12().with(|settings| settings.inline_subschemas = true);
    let schema = settings
        .into_generator()
        .into_root_schema_for::<AppConfig>();
    let mut stdout = io::stdout().lock();
    serde_json::to_writer_pretty(&mut stdout, &schema)?;
    writeln!(stdout)?;
    Ok(())
}
