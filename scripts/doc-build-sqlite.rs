#!/usr/bin/env cargo +nightly -Zscript
//! Build SQLite database from documentation files
//! 
//! This script parses all documentation files and builds a single
//! SQLite database with the schema defined in the specification.

// Cargo.toml
[package]
name = "doc-build-sqlite"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
walkdir = "2.3"
regex = "1.10"
chrono = { version = "0.4", features = ["serde"] }
rusqlite = { version = "0.31", features = ["bundled"] }
tempfile = "3.8"

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use rusqlite::{Connection, Result as SqlResult};
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Frontmatter {
    // Component fields
    icon: Option<String>,
    introduced_in_version: Option<String>,
    deprecated_in_version: Option<String>,
    difficulty: Option<String>,
    
    // Function fields
    namespace: Option<String>,
    return_type: Option<String>,
    category: Option<String>,
    
    // Guide fields
    title: Option<String>,
    estimated_time: Option<String>,
    categories: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    prerequisites: Option<Vec<String>>,
    next: Option<Vec<String>>,
    
    // Blog fields
    author: Option<String>,
    featured: Option<bool>,
    preview_image: Option<String>,
    excerpt: Option<String>,
    
    // Configuration fields
    last_reviewed: Option<String>,
    last_updated: Option<String>,
}

#[derive(Debug)]
struct DocFile {
    path: PathBuf,
    frontmatter: Frontmatter,
    content: String,
    doc_type: DocType,
}

#[derive(Debug, Clone)]
enum DocType {
    Component,
    Function,
    Guide,
    Blog,
    Configuration,
    Architecture,
}

impl DocFile {
    fn from_path(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let (frontmatter, content) = parse_frontmatter(&content)?;
        let doc_type = determine_doc_type(path)?;
        
        Ok(DocFile {
            path: path.to_path_buf(),
            frontmatter,
            content,
            doc_type,
        })
    }
}

fn parse_frontmatter(content: &str) -> Result<(Frontmatter, String), Box<dyn std::error::Error>> {
    if !content.starts_with("---\n") {
        return Ok((Frontmatter {
            icon: None,
            introduced_in_version: None,
            deprecated_in_version: None,
            difficulty: None,
            namespace: None,
            return_type: None,
            category: None,
            title: None,
            estimated_time: None,
            categories: None,
            tags: None,
            prerequisites: None,
            next: None,
            author: None,
            featured: None,
            preview_image: None,
            excerpt: None,
            last_reviewed: None,
            last_updated: None,
        }, content.to_string()));
    }
    
    let end_marker = content.find("\n---\n").ok_or("Missing frontmatter end marker")?;
    let frontmatter_yaml = &content[4..end_marker];
    let content = content[end_marker + 5..].to_string();
    
    let frontmatter: Frontmatter = serde_yaml::from_str(frontmatter_yaml)?;
    Ok((frontmatter, content))
}

fn determine_doc_type(path: &Path) -> Result<DocType, Box<dyn std::error::Error>> {
    let path_str = path.to_string_lossy();
    
    if path_str.contains("/components/") {
        Ok(DocType::Component)
    } else if path_str.contains("/functions/") {
        Ok(DocType::Function)
    } else if path_str.contains("/guides/") {
        Ok(DocType::Guide)
    } else if path_str.contains("/blog/") {
        Ok(DocType::Blog)
    } else if path_str.contains("/configuration/") {
        Ok(DocType::Configuration)
    } else if path_str.contains("/architecture/") {
        Ok(DocType::Architecture)
    } else {
        Err("Unknown document type".into())
    }
}

fn extract_section(content: &str, section_name: &str) -> Option<String> {
    let pattern = format!(r"## {}\s*\n(.*?)(?=\n## |\z)", regex::escape(section_name));
    let regex = Regex::new(&pattern).ok()?;
    let captures = regex.captures(content)?;
    Some(captures[1].trim().to_string())
}

fn extract_sql_blocks(content: &str) -> Vec<String> {
    let sql_block_regex = Regex::new(r"```sql\n(.*?)\n```").unwrap();
    let mut blocks = Vec::new();
    
    for cap in sql_block_regex.captures_iter(content) {
        let sql = cap[1].trim();
        if !sql.is_empty() {
            blocks.push(sql.to_string());
        }
    }
    
    blocks
}

fn parse_parameter_table(content: &str, section_name: &str) -> Vec<HashMap<String, String>> {
    let section_content = extract_section(content, section_name).unwrap_or_default();
    let mut parameters = Vec::new();
    
    for line in section_content.lines() {
        if line.starts_with('|') && !line.starts_with("|---") {
            let parts: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
            if parts.len() >= 5 {
                let mut param = HashMap::new();
                param.insert("name".to_string(), parts[1].to_string());
                param.insert("type".to_string(), parts[2].to_string());
                param.insert("required".to_string(), parts[3].to_string());
                param.insert("default".to_string(), parts[4].to_string());
                if parts.len() > 5 {
                    param.insert("description".to_string(), parts[5].to_string());
                }
                parameters.push(param);
            }
        }
    }
    
    parameters
}

fn create_schema(conn: &Connection) -> SqlResult<()> {
    // Components table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS components (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            icon TEXT,
            introduced_in_version TEXT,
            deprecated_in_version TEXT,
            difficulty TEXT CHECK(difficulty IN ('beginner', 'intermediate', 'advanced')),
            overview_md TEXT,
            when_to_use_md TEXT,
            basic_usage_sql TEXT,
            related_json TEXT,
            changelog_md TEXT,
            last_reviewed TEXT,
            last_updated TEXT
        )",
        [],
    )?;

    // Component parameters table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS component_parameters (
            component_id INTEGER REFERENCES components(id) ON DELETE CASCADE,
            level TEXT CHECK(level IN ('top', 'row')) NOT NULL,
            name TEXT NOT NULL,
            type TEXT,
            required INTEGER CHECK(required IN (0, 1)) NOT NULL DEFAULT 0,
            default_value TEXT,
            description_md TEXT,
            version_introduced TEXT,
            PRIMARY KEY (component_id, level, name)
        )",
        [],
    )?;

    // Component examples table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS component_examples (
            id INTEGER PRIMARY KEY,
            component_id INTEGER REFERENCES components(id) ON DELETE CASCADE,
            title TEXT,
            description_md TEXT,
            sql TEXT,
            difficulty TEXT,
            compatibility_json TEXT,
            featured INTEGER CHECK(featured IN (0, 1)) DEFAULT 0
        )",
        [],
    )?;

    // Functions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS functions (
            id INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            namespace TEXT DEFAULT 'sqlpage',
            icon TEXT,
            return_type TEXT,
            introduced_in_version TEXT,
            deprecated_in_version TEXT,
            category TEXT,
            difficulty TEXT CHECK(difficulty IN ('beginner', 'intermediate', 'advanced')),
            signature_md TEXT,
            description_md TEXT,
            return_value_md TEXT,
            security_notes_md TEXT,
            related_json TEXT,
            last_reviewed TEXT,
            last_updated TEXT
        )",
        [],
    )?;

    // Function parameters table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS function_parameters (
            function_id INTEGER REFERENCES functions(id) ON DELETE CASCADE,
            position INTEGER NOT NULL,
            name TEXT NOT NULL,
            type TEXT,
            required INTEGER CHECK(required IN (0, 1)) NOT NULL DEFAULT 0,
            description_md TEXT,
            example_md TEXT,
            PRIMARY KEY (function_id, position)
        )",
        [],
    )?;

    // Function examples table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS function_examples (
            id INTEGER PRIMARY KEY,
            function_id INTEGER REFERENCES functions(id) ON DELETE CASCADE,
            title TEXT,
            description_md TEXT,
            sql TEXT
        )",
        [],
    )?;

    // Guides table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS guides (
            id INTEGER PRIMARY KEY,
            slug TEXT UNIQUE NOT NULL,
            title TEXT NOT NULL,
            difficulty TEXT CHECK(difficulty IN ('beginner', 'intermediate', 'advanced')),
            estimated_time TEXT,
            introduced_in_version TEXT,
            categories_json TEXT,
            tags_json TEXT,
            prerequisites_json TEXT,
            next_json TEXT,
            content_md TEXT,
            last_reviewed TEXT,
            last_updated TEXT
        )",
        [],
    )?;

    // Blog posts table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS blog_posts (
            id INTEGER PRIMARY KEY,
            slug TEXT UNIQUE NOT NULL,
            date TEXT NOT NULL,
            title TEXT NOT NULL,
            author TEXT,
            tags_json TEXT,
            categories_json TEXT,
            featured INTEGER CHECK(featured IN (0, 1)) DEFAULT 0,
            preview_image TEXT,
            excerpt TEXT,
            content_md TEXT
        )",
        [],
    )?;

    // Configuration pages table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS configuration_pages (
            id INTEGER PRIMARY KEY,
            slug TEXT UNIQUE NOT NULL,
            title TEXT NOT NULL,
            introduced_in_version TEXT,
            categories_json TEXT,
            tags_json TEXT,
            content_md TEXT
        )",
        [],
    )?;

    // Configuration settings table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS configuration_settings (
            page_id INTEGER REFERENCES configuration_pages(id) ON DELETE CASCADE,
            name TEXT NOT NULL,
            aliases_json TEXT,
            type TEXT,
            required INTEGER CHECK(required IN (0, 1)) DEFAULT 0,
            default_value TEXT,
            description_md TEXT,
            example_md TEXT,
            introduced_in_version TEXT,
            PRIMARY KEY (page_id, name)
        )",
        [],
    )?;

    // Search index table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS search_index (
            id INTEGER PRIMARY KEY,
            type TEXT,
            key TEXT,
            title TEXT,
            content_tsv TEXT
        )",
        [],
    )?;

    Ok(())
}

fn insert_component(conn: &Connection, doc: &DocFile) -> SqlResult<i64> {
    let name = doc.path.file_stem().unwrap().to_string_lossy();
    let overview_md = extract_section(&doc.content, "Overview");
    let when_to_use_md = extract_section(&doc.content, "When to Use");
    let basic_usage_sql = extract_sql_blocks(&doc.content).first().cloned();
    let related_json = extract_section(&doc.content, "Related")
        .map(|s| serde_json::to_string(&s).unwrap_or_default());
    let changelog_md = extract_section(&doc.content, "Changelog");

    conn.execute(
        "INSERT OR REPLACE INTO components 
         (name, icon, introduced_in_version, deprecated_in_version, difficulty,
          overview_md, when_to_use_md, basic_usage_sql, related_json, changelog_md,
          last_reviewed, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        [
            &name,
            &doc.frontmatter.icon.as_deref().unwrap_or(""),
            &doc.frontmatter.introduced_in_version.as_deref().unwrap_or(""),
            &doc.frontmatter.deprecated_in_version.as_deref().unwrap_or(""),
            &doc.frontmatter.difficulty.as_deref().unwrap_or(""),
            &overview_md.unwrap_or_default(),
            &when_to_use_md.unwrap_or_default(),
            &basic_usage_sql.unwrap_or_default(),
            &related_json.unwrap_or_default(),
            &changelog_md.unwrap_or_default(),
            &doc.frontmatter.last_reviewed.as_deref().unwrap_or(""),
            &doc.frontmatter.last_updated.as_deref().unwrap_or(""),
        ],
    )?;

    let component_id = conn.last_insert_rowid();

    // Insert parameters
    let top_params = parse_parameter_table(&doc.content, "Top-Level Parameters");
    for param in top_params {
        conn.execute(
            "INSERT OR REPLACE INTO component_parameters 
             (component_id, level, name, type, required, default_value, description_md)
             VALUES (?1, 'top', ?2, ?3, ?4, ?5, ?6)",
            [
                &component_id.to_string(),
                &param.get("name").unwrap_or(&"".to_string()),
                &param.get("type").unwrap_or(&"".to_string()),
                &(param.get("required").unwrap_or(&"false").to_lowercase() == "true").to_string(),
                &param.get("default").unwrap_or(&"".to_string()),
                &param.get("description").unwrap_or(&"".to_string()),
            ],
        )?;
    }

    let row_params = parse_parameter_table(&doc.content, "Row-Level Parameters");
    for param in row_params {
        conn.execute(
            "INSERT OR REPLACE INTO component_parameters 
             (component_id, level, name, type, required, default_value, description_md)
             VALUES (?1, 'row', ?2, ?3, ?4, ?5, ?6)",
            [
                &component_id.to_string(),
                &param.get("name").unwrap_or(&"".to_string()),
                &param.get("type").unwrap_or(&"".to_string()),
                &(param.get("required").unwrap_or(&"false").to_lowercase() == "true").to_string(),
                &param.get("default").unwrap_or(&"".to_string()),
                &param.get("description").unwrap_or(&"".to_string()),
            ],
        )?;
    }

    Ok(component_id)
}

fn insert_function(conn: &Connection, doc: &DocFile) -> SqlResult<i64> {
    let name = doc.path.file_stem().unwrap().to_string_lossy();
    let signature_md = extract_section(&doc.content, "Signature");
    let description_md = extract_section(&doc.content, "Description");
    let return_value_md = extract_section(&doc.content, "Return Value");
    let security_notes_md = extract_section(&doc.content, "Security Notes");
    let related_json = extract_section(&doc.content, "Related")
        .map(|s| serde_json::to_string(&s).unwrap_or_default());

    conn.execute(
        "INSERT OR REPLACE INTO functions 
         (name, namespace, icon, return_type, introduced_in_version, deprecated_in_version,
          category, difficulty, signature_md, description_md, return_value_md, security_notes_md,
          related_json, last_reviewed, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        [
            &name,
            &doc.frontmatter.namespace.as_deref().unwrap_or("sqlpage"),
            &doc.frontmatter.icon.as_deref().unwrap_or(""),
            &doc.frontmatter.return_type.as_deref().unwrap_or(""),
            &doc.frontmatter.introduced_in_version.as_deref().unwrap_or(""),
            &doc.frontmatter.deprecated_in_version.as_deref().unwrap_or(""),
            &doc.frontmatter.category.as_deref().unwrap_or(""),
            &doc.frontmatter.difficulty.as_deref().unwrap_or(""),
            &signature_md.unwrap_or_default(),
            &description_md.unwrap_or_default(),
            &return_value_md.unwrap_or_default(),
            &security_notes_md.unwrap_or_default(),
            &related_json.unwrap_or_default(),
            &doc.frontmatter.last_reviewed.as_deref().unwrap_or(""),
            &doc.frontmatter.last_updated.as_deref().unwrap_or(""),
        ],
    )?;

    let function_id = conn.last_insert_rowid();

    // Insert parameters
    let params = parse_parameter_table(&doc.content, "Parameters");
    for (i, param) in params.iter().enumerate() {
        conn.execute(
            "INSERT OR REPLACE INTO function_parameters 
             (function_id, position, name, type, required, description_md)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                &function_id.to_string(),
                &i.to_string(),
                &param.get("name").unwrap_or(&"".to_string()),
                &param.get("type").unwrap_or(&"".to_string()),
                &(param.get("required").unwrap_or(&"false").to_lowercase() == "true").to_string(),
                &param.get("description").unwrap_or(&"".to_string()),
            ],
        )?;
    }

    Ok(function_id)
}

fn insert_guide(conn: &Connection, doc: &DocFile) -> SqlResult<i64> {
    let slug = doc.path.file_stem().unwrap().to_string_lossy();
    let title = doc.frontmatter.title.as_deref().unwrap_or(&slug);
    let categories_json = doc.frontmatter.categories.as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default())
        .unwrap_or_default();
    let tags_json = doc.frontmatter.tags.as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default())
        .unwrap_or_default();
    let prerequisites_json = doc.frontmatter.prerequisites.as_ref()
        .map(|p| serde_json::to_string(p).unwrap_or_default())
        .unwrap_or_default();
    let next_json = doc.frontmatter.next.as_ref()
        .map(|n| serde_json::to_string(n).unwrap_or_default())
        .unwrap_or_default();

    conn.execute(
        "INSERT OR REPLACE INTO guides 
         (slug, title, difficulty, estimated_time, introduced_in_version,
          categories_json, tags_json, prerequisites_json, next_json, content_md,
          last_reviewed, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        [
            &slug,
            title,
            &doc.frontmatter.difficulty.as_deref().unwrap_or(""),
            &doc.frontmatter.estimated_time.as_deref().unwrap_or(""),
            &doc.frontmatter.introduced_in_version.as_deref().unwrap_or(""),
            &categories_json,
            &tags_json,
            &prerequisites_json,
            &next_json,
            &doc.content,
            &doc.frontmatter.last_reviewed.as_deref().unwrap_or(""),
            &doc.frontmatter.last_updated.as_deref().unwrap_or(""),
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn insert_blog_post(conn: &Connection, doc: &DocFile) -> SqlResult<i64> {
    let filename = doc.path.file_stem().unwrap().to_string_lossy();
    let parts: Vec<&str> = filename.split('-').collect();
    let date = if parts.len() >= 3 {
        format!("{}-{}-{}", parts[0], parts[1], parts[2])
    } else {
        "2024-01-01".to_string()
    };
    let slug = if parts.len() > 3 {
        parts[3..].join("-")
    } else {
        filename.to_string()
    };
    
    let title = doc.frontmatter.title.as_deref().unwrap_or(&slug);
    let tags_json = doc.frontmatter.tags.as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default())
        .unwrap_or_default();
    let categories_json = doc.frontmatter.categories.as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default())
        .unwrap_or_default();
    let featured = doc.frontmatter.featured.unwrap_or(false) as i32;

    conn.execute(
        "INSERT OR REPLACE INTO blog_posts 
         (slug, date, title, author, tags_json, categories_json, featured,
          preview_image, excerpt, content_md)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        [
            &slug,
            &date,
            title,
            &doc.frontmatter.author.as_deref().unwrap_or(""),
            &tags_json,
            &categories_json,
            &featured.to_string(),
            &doc.frontmatter.preview_image.as_deref().unwrap_or(""),
            &doc.frontmatter.excerpt.as_deref().unwrap_or(""),
            &doc.content,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn insert_configuration_page(conn: &Connection, doc: &DocFile) -> SqlResult<i64> {
    let slug = doc.path.file_stem().unwrap().to_string_lossy();
    let title = doc.frontmatter.title.as_deref().unwrap_or(&slug);
    let categories_json = doc.frontmatter.categories.as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default())
        .unwrap_or_default();
    let tags_json = doc.frontmatter.tags.as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default())
        .unwrap_or_default();

    conn.execute(
        "INSERT OR REPLACE INTO configuration_pages 
         (slug, title, introduced_in_version, categories_json, tags_json, content_md)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [
            &slug,
            title,
            &doc.frontmatter.introduced_in_version.as_deref().unwrap_or(""),
            &categories_json,
            &tags_json,
            &doc.content,
        ],
    )?;

    let page_id = conn.last_insert_rowid();

    // Parse settings table if present
    let settings = parse_parameter_table(&doc.content, "Settings");
    for setting in settings {
        conn.execute(
            "INSERT OR REPLACE INTO configuration_settings 
             (page_id, name, type, required, default_value, description_md)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            [
                &page_id.to_string(),
                &setting.get("name").unwrap_or(&"".to_string()),
                &setting.get("type").unwrap_or(&"".to_string()),
                &(setting.get("required").unwrap_or(&"false").to_lowercase() == "true").to_string(),
                &setting.get("default").unwrap_or(&"".to_string()),
                &setting.get("description").unwrap_or(&"".to_string()),
            ],
        )?;
    }

    Ok(page_id)
}

fn find_doc_files() -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new("docs") {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            // Skip the schema file
            if path.file_name().unwrap() != "_schema.md" {
                files.push(path.to_path_buf());
            }
        }
    }
    
    Ok(files)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Building SQLite database from documentation...");
    
    // Create temporary database
    let temp_file = NamedTempFile::new()?;
    let temp_path = temp_file.path();
    
    let conn = Connection::open(temp_path)?;
    create_schema(&conn)?;
    
    // Process all documentation files
    let doc_files = find_doc_files()?;
    let mut processed = 0;
    
    for file_path in doc_files {
        let doc = DocFile::from_path(&file_path)?;
        
        match doc.doc_type {
            DocType::Component => {
                insert_component(&conn, &doc)?;
                processed += 1;
            },
            DocType::Function => {
                insert_function(&conn, &doc)?;
                processed += 1;
            },
            DocType::Guide => {
                insert_guide(&conn, &doc)?;
                processed += 1;
            },
            DocType::Blog => {
                insert_blog_post(&conn, &doc)?;
                processed += 1;
            },
            DocType::Configuration => {
                insert_configuration_page(&conn, &doc)?;
                processed += 1;
            },
            DocType::Architecture => {
                // Architecture docs are not stored in the database yet
                // but could be added in the future
            },
        }
    }
    
    // Move temporary file to final location
    let final_path = "docs.sqlite";
    if std::path::Path::new(final_path).exists() {
        std::fs::remove_file(final_path)?;
    }
    std::fs::rename(temp_path, final_path)?;
    
    println!("✅ Successfully built docs.sqlite with {} documents", processed);
    Ok(())
}