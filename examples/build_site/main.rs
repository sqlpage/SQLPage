mod configuration;
mod content;

use anyhow::{Context, Result, bail};
use rusqlite::{
    Connection, ToSql,
    types::{ToSqlOutput, Value, ValueRef},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

const SEED_FILE: &str = "sqlpage/migrations/01_documentation.sql";

fn main() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = root.join("examples/official-site");
    let seed = documentation_seed(&source.join("content"))?;

    let parent = root.join("target");
    fs::create_dir_all(&parent)?;
    let destination = parent.join("official-site");
    let staging = tempfile::tempdir_in(parent)?;
    let next = staging.path().join("next");
    let previous = staging.path().join("previous");
    copy_site(&source, &next, Path::new(""))?;
    fs::copy(root.join("configuration.md"), next.join("configuration.md"))?;
    fs::write(next.join(SEED_FILE), seed)?;

    // Leave the current preview in place until the replacement is ready.
    let had_previous = match fs::rename(&destination, &previous) {
        Ok(()) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => false,
        Err(error) => return Err(error.into()),
    };
    if let Err(error) = fs::rename(next, &destination) {
        if had_previous && let Err(restore) = fs::rename(&previous, &destination) {
            let backup = staging.keep().join("previous");
            bail!(
                "Could not publish site ({error}) or restore it ({restore}). Backup: {}",
                backup.display()
            );
        }
        return Err(error).context("Publishing the prepared site");
    }
    println!("Prepared {}", destination.display());
    Ok(())
}

fn copy_site(source: &Path, target: &Path, relative: &Path) -> Result<()> {
    const EXCLUDED: &[&str] = &[
        "content",
        "README.md",
        "docker-compose.yml",
        "test.hurl",
        "sqlpage/sqlpage.db",
        "sqlpage/https",
    ];
    fs::create_dir(target)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let child = relative.join(entry.file_name());
        if EXCLUDED.iter().any(|path| child == Path::new(path)) {
            continue;
        }
        let dest = target.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_site(&entry.path(), &dest, &child)?;
        } else {
            fs::copy(entry.path(), dest)?;
        }
    }
    Ok(())
}

fn documentation_seed(content: &Path) -> Result<String> {
    let mut seed = Seed {
        db: Connection::open_in_memory()?,
        sql: String::new(),
    };
    seed.db.pragma_update(None, "foreign_keys", true)?;
    seed.script(include_str!("schema.sql"))?;
    content::insert(&mut seed, content)?;
    configuration::insert(&mut seed)?;
    seed.script(include_str!("search.sql"))?;
    Ok(seed.sql)
}

struct Seed {
    db: Connection,
    sql: String,
}

impl Seed {
    fn script(&mut self, sql: &str) -> Result<()> {
        self.db.execute_batch(sql)?;
        self.sql.push_str(sql);
        Ok(())
    }

    fn row(&mut self, sql: &str, values: &[&dyn ToSql]) -> Result<()> {
        // SQLite's expanded SQL cannot round-trip NUL bytes in text.
        for value in values {
            let output = value.to_sql()?;
            let text = match &output {
                ToSqlOutput::Borrowed(ValueRef::Text(bytes)) => *bytes,
                ToSqlOutput::Owned(Value::Text(text)) => text.as_bytes(),
                _ => continue,
            };
            anyhow::ensure!(!text.contains(&0), "NUL byte in SQL text value");
        }
        let mut statement = self.db.prepare(sql)?;
        statement.execute(values)?;
        // SQLite renders the bound values for the startup seed.
        let expanded = statement
            .expanded_sql()
            .context("Expanding SQLite bindings")?;
        self.sql.push_str(&expanded);
        self.sql.push_str(";\n");
        Ok(())
    }
}

fn read(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)?)
}

fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("Reading {}", dir.display()))? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "md") {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_seed_round_trips_examples_and_indexes_configuration() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let content = temp.path();
        for dir in ["components/demo/examples", "functions", "blog"] {
            fs::create_dir_all(content.join(dir))?;
        }
        for file in ["categories.yaml", "guides.yaml", "projects.yaml"] {
            fs::write(content.join(file), "[]\n")?;
        }
        fs::write(
            content.join("shared-parameters.yaml"),
            "id: {name: id, type: TEXT, description: HTML id}\n",
        )?;
        let topic = content.join("components/demo.md");
        fs::write(&topic, "---\ntop: [id]\n---\n\nA contributor's demo.\n")?;
        let examples = content.join("components/demo/examples");
        fs::write(
            examples.join("02-result.md"),
            "---\nrows:\n  - component: text\n    contents: \"Reader's choice\"\n    \"2\": 9007199254740993\n    \"1\": 42\n---\nResult\n",
        )?;
        fs::write(examples.join("01-notes.md"), "Just a note.\n")?;

        let seed = documentation_seed(content)?;
        assert_eq!(seed, documentation_seed(content)?);
        let db = Connection::open_in_memory()?;
        db.execute_batch(&seed)?;
        let rows = db
            .prepare("SELECT description, source FROM example_with_source ORDER BY example_number")?
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        assert_eq!(
            rows,
            [
                ("Just a note.".into(), None),
                ("Result".into(), Some("select\n    'text' as \"component\",\n    'Reader''s choice' as \"contents\",\n    9007199254740993 as \"2\",\n    42 as \"1\";".into())),
            ]
        );
        let config = db
            .prepare("SELECT name, type, default_md FROM config_option
                      WHERE name IN ('allow_exec', 'listen_on', 'max_recursion_depth', 'smtp_from') ORDER BY name")?
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?)))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        assert_eq!(
            config[0],
            (
                "allow_exec".into(),
                "boolean".into(),
                Some("`false`".into())
            )
        );
        assert_eq!(config[1].1, "string or null");
        assert_eq!(
            config[1].2.as_deref(),
            Some("`0.0.0.0:443` with `https_domain`, otherwise `0.0.0.0:8080`.")
        );
        assert_eq!(config[2].1, "integer (0 to 255)");
        assert_eq!(
            config[3],
            ("smtp_from".into(), "string or null".into(), None)
        );
        let link: String = db.query_row(
            "SELECT page || '#' || fragment FROM documentation_fts
             WHERE documentation_fts MATCH 'listen_on' AND fragment = 'listen_on'",
            [],
            |row| row.get(0),
        )?;
        assert_eq!(link, "/configuration#listen_on");

        fs::write(&topic, "---\ntop: [missing]\n---\nExample.\n")?;
        let error = format!("{:#}", documentation_seed(content).unwrap_err());
        assert!(
            error.contains("demo.md: Unknown shared parameter \"missing\""),
            "{error}"
        );
        fs::rename(topic, content.join("components/moved.md"))?;
        let error = documentation_seed(content).unwrap_err().to_string();
        assert!(
            error.contains("no corresponding reference demo.md"),
            "{error}"
        );
        Ok(())
    }
}
