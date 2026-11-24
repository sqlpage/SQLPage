use super::{Seed, markdown_files, read};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Category {
    id: String,
    title: String,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Guide {
    title: String,
    link: String,
    description: String,
    sort_order: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Project {
    title: String,
    folder: String,
    db_engine: Option<String>,
    description: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Parameter {
    name: String,
    r#type: String,
    description: String,
    #[serde(default)]
    required: bool,
}

impl Parameter {
    fn validate(&self) -> Result<()> {
        for (name, value) in [
            ("name", &self.name),
            ("type", &self.r#type),
            ("description", &self.description),
        ] {
            ensure!(!value.trim().is_empty(), "Expected text in {name}");
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(untagged)]
enum ComponentParameter {
    Shared(String),
    Inline(Parameter),
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Component {
    icon: Option<String>,
    since: Option<String>,
    category: Option<String>,
    summary: Option<String>,
    #[serde(default)]
    top: Vec<ComponentParameter>,
    #[serde(default)]
    row: Vec<ComponentParameter>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Function {
    icon: Option<String>,
    since: Option<String>,
    category: Option<String>,
    summary: Option<String>,
    return_type: Option<String>,
    #[serde(default)]
    parameters: Vec<Parameter>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Example {
    // serde_json preserves mapping order and large integers. The same rows
    // drive both the live preview and the SQL shown to the reader.
    rows: Option<Vec<serde_json::Map<String, Value>>>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Blog {
    title: String,
    date: String,
    icon: String,
    summary: String,
    external_url: Option<String>,
}

pub(super) fn insert(seed: &mut Seed, dir: &Path) -> Result<()> {
    collection(dir, "categories.yaml", |c: Category| {
        seed.row(
            "INSERT INTO reference_category (id, title, sort_order) VALUES (?, ?, ?)",
            &[&c.id, &c.title, &c.sort_order],
        )
    })?;
    collection(dir, "guides.yaml", |g: Guide| {
        seed.row(
            "INSERT INTO guide (title, link, description, sort_order) VALUES (?, ?, ?, ?)",
            &[&g.title, &g.link, &g.description, &g.sort_order],
        )
    })?;
    collection(dir, "projects.yaml", |p: Project| {
        seed.row(
            "INSERT INTO example_cards (title, folder, db_engine, description) VALUES (?, ?, ?, ?)",
            &[&p.title, &p.folder, &p.db_engine, &p.description],
        )
    })?;
    let shared_path = dir.join("shared-parameters.yaml");
    let shared: BTreeMap<String, Parameter> = at(&shared_path, || {
        let entries: BTreeMap<String, Parameter> = serde_yaml_ng::from_str(&read(&shared_path)?)?;
        for parameter in entries.values() {
            parameter.validate()?;
        }
        Ok(entries)
    })?;
    components(seed, &dir.join("components"), &shared)?;
    functions(seed, &dir.join("functions"))?;
    pages(&dir.join("blog"), |_, b: Blog, body| {
        ensure!(
            [&b.title, &b.date, &b.icon, &b.summary]
                .iter()
                .all(|s| !s.trim().is_empty()),
            "Expected title, date, icon and summary"
        );
        seed.row(
            "INSERT INTO blog_posts (title, created_at, icon, description, external_url, content)
             VALUES (?, ?, ?, ?, ?, ?)",
            &[
                &b.title,
                &b.date,
                &b.icon,
                &b.summary,
                &b.external_url,
                &(!body.is_empty()).then_some(body),
            ],
        )
    })
}

fn components(seed: &mut Seed, dir: &Path, shared: &BTreeMap<String, Parameter>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let mut reference = entry.file_name();
            reference.push(".md");
            ensure!(
                dir.join(&reference).is_file(),
                "{} has no corresponding reference {}",
                entry.path().display(),
                reference.to_string_lossy()
            );
        }
    }
    pages(dir, |name, c: Component, body| {
        seed.row(
            "INSERT INTO component (name, description, icon, introduced_in_version, catalog_category, catalog_summary)
             VALUES (?, ?, ?, ?, ?, ?)",
            &[&name, &body, &c.icon, &c.since, &c.category, &c.summary],
        )?;
        for (top, parameters) in [(true, c.top), (false, c.row)] {
            for p in &parameters {
                let p = match p {
                    ComponentParameter::Inline(p) => p,
                    ComponentParameter::Shared(key) => shared
                        .get(key)
                        .with_context(|| format!("Unknown shared parameter {key:?}"))?,
                };
                p.validate()?;
                seed.row(
                    "INSERT INTO parameter (component, top_level, name, type, description_md, optional)
                     VALUES (?, ?, ?, ?, ?, ?)",
                    &[&name, &top, &p.name, &p.r#type, &p.description, &!p.required],
                )?;
            }
        }
        let examples = dir.join(name).join("examples");
        if examples.try_exists()? {
            pages(&examples, |_, e: Example, description| {
                let properties = e
                    .rows
                    .map(|rows| serde_json::to_string(&rows))
                    .transpose()?;
                seed.row(
                    "INSERT INTO example (component, description, properties) VALUES (?, ?, ?)",
                    &[&name, &description, &properties],
                )
            })?;
        }
        Ok(())
    })
}

fn functions(seed: &mut Seed, dir: &Path) -> Result<()> {
    pages(dir, |name, f: Function, body| {
        seed.row(
            "INSERT INTO sqlpage_functions (name, icon, description_md, return_type,
                 introduced_in_version, catalog_category, catalog_summary) VALUES (?, ?, ?, ?, ?, ?, ?)",
            &[&name, &f.icon, &body, &f.return_type, &f.since, &f.category, &f.summary],
        )?;
        for (index, p) in (1_i64..).zip(f.parameters) {
            p.validate()?;
            seed.row(
                "INSERT INTO sqlpage_function_parameters (\"function\", \"index\", name, description_md, type, optional)
                 VALUES (?, ?, ?, ?, ?, ?)",
                &[&name, &index, &p.name, &p.description, &p.r#type, &!p.required],
            )?;
        }
        Ok(())
    })
}

fn collection<T: DeserializeOwned>(
    dir: &Path,
    file: &str,
    mut insert: impl FnMut(T) -> Result<()>,
) -> Result<()> {
    let path = dir.join(file);
    at(&path, || {
        for item in serde_yaml_ng::from_str::<Vec<T>>(&read(&path)?)? {
            insert(item)?;
        }
        Ok(())
    })
}

fn pages<T: Default + DeserializeOwned>(
    dir: &Path,
    mut visit: impl FnMut(&str, T, &str) -> Result<()>,
) -> Result<()> {
    for path in markdown_files(dir)? {
        at(&path, || {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .context("Invalid filename")?;
            let file = read(&path)?.replace("\r\n", "\n");
            let (meta, body) = if let Some(rest) = file.strip_prefix("---\n") {
                let mut offset = 0;
                let end = rest
                    .split_inclusive('\n')
                    .find_map(|line| {
                        let start = offset;
                        offset += line.len();
                        (line.trim_end_matches([' ', '\t', '\n']) == "---").then_some(start)
                    })
                    .context("Unclosed YAML front matter")?;
                (
                    serde_yaml_ng::from_str::<Option<T>>(&rest[..end])?.unwrap_or_default(),
                    &rest[offset..],
                )
            } else {
                (T::default(), file.as_str())
            };
            visit(name, meta, body.trim())
        })?;
    }
    Ok(())
}

fn at<T>(path: &Path, f: impl FnOnce() -> Result<T>) -> Result<T> {
    f().with_context(|| path.display().to_string())
}
