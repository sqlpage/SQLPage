extern crate rusqlite;
extern crate serde;
extern crate serde_yaml;
extern crate serde_json;
extern crate walkdir;
extern crate regex;

use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use serde::{Deserialize, Serialize};
use rusqlite::{Connection, Result as SqlResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Frontmatter {
    icon: Option<String>,
    introduced_in_version: Option<String>,
    deprecated_in_version: Option<String>,
    difficulty: Option<String>,
    namespace: Option<String>,
    return_type: Option<String>,
    category: Option<String>,
    title: Option<String>,
    estimated_time: Option<String>,
    categories: Option<Vec<String>>,
    tags: Option<Vec<String>>,
    prerequisites: Option<Vec<String>>,
    next: Option<Vec<String>>,
    author: Option<String>,
    featured: Option<bool>,
    preview_image: Option<String>,
    excerpt: Option<String>,
    last_reviewed: Option<String>,
    last_updated: Option<String>,
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

    Ok(())
}

fn insert_component(conn: &Connection, name: &str, frontmatter: &Frontmatter, content: &str) -> SqlResult<i64> {
    let overview_md = extract_section(content, "Overview");
    let when_to_use_md = extract_section(content, "When to Use");
    let basic_usage_sql = extract_sql_blocks(content).first().cloned();
    let related_json = extract_section(content, "Related")
        .map(|s| serde_json::to_string(&s).unwrap_or_default());
    let changelog_md = extract_section(content, "Changelog");

    conn.execute(
        "INSERT OR REPLACE INTO components 
         (name, icon, introduced_in_version, deprecated_in_version, difficulty,
          overview_md, when_to_use_md, basic_usage_sql, related_json, changelog_md,
          last_reviewed, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        [
            name,
            &frontmatter.icon.as_deref().unwrap_or(""),
            &frontmatter.introduced_in_version.as_deref().unwrap_or(""),
            &frontmatter.deprecated_in_version.as_deref().unwrap_or(""),
            &frontmatter.difficulty.as_deref().unwrap_or(""),
            &overview_md.unwrap_or_default(),
            &when_to_use_md.unwrap_or_default(),
            &basic_usage_sql.unwrap_or_default(),
            &related_json.unwrap_or_default(),
            &changelog_md.unwrap_or_default(),
            &frontmatter.last_reviewed.as_deref().unwrap_or(""),
            &frontmatter.last_updated.as_deref().unwrap_or(""),
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn insert_function(conn: &Connection, name: &str, frontmatter: &Frontmatter, content: &str) -> SqlResult<i64> {
    let signature_md = extract_section(content, "Signature");
    let description_md = extract_section(content, "Description");
    let return_value_md = extract_section(content, "Return Value");
    let security_notes_md = extract_section(content, "Security Notes");
    let related_json = extract_section(content, "Related")
        .map(|s| serde_json::to_string(&s).unwrap_or_default());

    conn.execute(
        "INSERT OR REPLACE INTO functions 
         (name, namespace, icon, return_type, introduced_in_version, deprecated_in_version,
          category, difficulty, signature_md, description_md, return_value_md, security_notes_md,
          related_json, last_reviewed, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        [
            name,
            &frontmatter.namespace.as_deref().unwrap_or("sqlpage"),
            &frontmatter.icon.as_deref().unwrap_or(""),
            &frontmatter.return_type.as_deref().unwrap_or(""),
            &frontmatter.introduced_in_version.as_deref().unwrap_or(""),
            &frontmatter.deprecated_in_version.as_deref().unwrap_or(""),
            &frontmatter.category.as_deref().unwrap_or(""),
            &frontmatter.difficulty.as_deref().unwrap_or(""),
            &signature_md.unwrap_or_default(),
            &description_md.unwrap_or_default(),
            &return_value_md.unwrap_or_default(),
            &security_notes_md.unwrap_or_default(),
            &related_json.unwrap_or_default(),
            &frontmatter.last_reviewed.as_deref().unwrap_or(""),
            &frontmatter.last_updated.as_deref().unwrap_or(""),
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn insert_guide(conn: &Connection, slug: &str, frontmatter: &Frontmatter, content: &str) -> SqlResult<i64> {
    let title = frontmatter.title.as_deref().unwrap_or(slug);
    let categories_json = frontmatter.categories.as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default())
        .unwrap_or_default();
    let tags_json = frontmatter.tags.as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default())
        .unwrap_or_default();
    let prerequisites_json = frontmatter.prerequisites.as_ref()
        .map(|p| serde_json::to_string(p).unwrap_or_default())
        .unwrap_or_default();
    let next_json = frontmatter.next.as_ref()
        .map(|n| serde_json::to_string(n).unwrap_or_default())
        .unwrap_or_default();

    conn.execute(
        "INSERT OR REPLACE INTO guides 
         (slug, title, difficulty, estimated_time, introduced_in_version,
          categories_json, tags_json, prerequisites_json, next_json, content_md,
          last_reviewed, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        [
            slug,
            title,
            &frontmatter.difficulty.as_deref().unwrap_or(""),
            &frontmatter.estimated_time.as_deref().unwrap_or(""),
            &frontmatter.introduced_in_version.as_deref().unwrap_or(""),
            &categories_json,
            &tags_json,
            &prerequisites_json,
            &next_json,
            content,
            &frontmatter.last_reviewed.as_deref().unwrap_or(""),
            &frontmatter.last_updated.as_deref().unwrap_or(""),
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn insert_blog_post(conn: &Connection, filename: &str, frontmatter: &Frontmatter, content: &str) -> SqlResult<i64> {
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
    
    let title = frontmatter.title.as_deref().unwrap_or(&slug);
    let tags_json = frontmatter.tags.as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default())
        .unwrap_or_default();
    let categories_json = frontmatter.categories.as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default())
        .unwrap_or_default();
    let featured = frontmatter.featured.unwrap_or(false) as i32;

    conn.execute(
        "INSERT OR REPLACE INTO blog_posts 
         (slug, date, title, author, tags_json, categories_json, featured,
          preview_image, excerpt, content_md)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        [
            &slug,
            &date,
            title,
            &frontmatter.author.as_deref().unwrap_or(""),
            &tags_json,
            &categories_json,
            &featured.to_string(),
            &frontmatter.preview_image.as_deref().unwrap_or(""),
            &frontmatter.excerpt.as_deref().unwrap_or(""),
            content,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn insert_configuration_page(conn: &Connection, slug: &str, frontmatter: &Frontmatter, content: &str) -> SqlResult<i64> {
    let title = frontmatter.title.as_deref().unwrap_or(slug);
    let categories_json = frontmatter.categories.as_ref()
        .map(|c| serde_json::to_string(c).unwrap_or_default())
        .unwrap_or_default();
    let tags_json = frontmatter.tags.as_ref()
        .map(|t| serde_json::to_string(t).unwrap_or_default())
        .unwrap_or_default();

    conn.execute(
        "INSERT OR REPLACE INTO configuration_pages 
         (slug, title, introduced_in_version, categories_json, tags_json, content_md)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [
            slug,
            title,
            &frontmatter.introduced_in_version.as_deref().unwrap_or(""),
            &categories_json,
            &tags_json,
            content,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Building SQLite database from documentation...");
    
    // Create database
    let conn = Connection::open("docs.sqlite")?;
    create_schema(&conn)?;
    
    // Process all documentation files
    let mut processed = 0;
    
    for entry in WalkDir::new("docs") {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            // Skip the schema file
            if path.file_name().unwrap() == "_schema.md" {
                continue;
            }
            
            let content = fs::read_to_string(path)?;
            let (frontmatter, content) = parse_frontmatter(&content)?;
            
            let path_str = path.to_string_lossy();
            let name = path.file_stem().unwrap().to_string_lossy();
            
            if path_str.contains("/components/") {
                insert_component(&conn, &name, &frontmatter, &content)?;
                processed += 1;
            } else if path_str.contains("/functions/") {
                insert_function(&conn, &name, &frontmatter, &content)?;
                processed += 1;
            } else if path_str.contains("/guides/") {
                insert_guide(&conn, &name, &frontmatter, &content)?;
                processed += 1;
            } else if path_str.contains("/blog/") {
                insert_blog_post(&conn, &name, &frontmatter, &content)?;
                processed += 1;
            } else if path_str.contains("/configuration/") {
                insert_configuration_page(&conn, &name, &frontmatter, &content)?;
                processed += 1;
            }
        }
    }
    
    println!("✅ Successfully built docs.sqlite with {} documents", processed);
    Ok(())
}