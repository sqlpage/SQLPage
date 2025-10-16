extern crate rusqlite;
extern crate walkdir;
extern crate regex;

use std::fs;
use walkdir::WalkDir;
use regex::Regex;

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

fn create_schema(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
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

fn parse_frontmatter(content: &str) -> (String, String, String, String, String) {
    if !content.starts_with("---\n") {
        return ("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string());
    }
    
    let end_marker = match content.find("\n---\n") {
        Some(pos) => pos,
        None => return ("".to_string(), "".to_string(), "".to_string(), "".to_string(), "".to_string()),
    };
    
    let frontmatter = &content[4..end_marker];
    let content = content[end_marker + 5..].to_string();
    
    // Simple parsing - extract title, difficulty, introduced_in_version
    let title = extract_yaml_field(frontmatter, "title");
    let difficulty = extract_yaml_field(frontmatter, "difficulty");
    let introduced_in_version = extract_yaml_field(frontmatter, "introduced_in_version");
    let author = extract_yaml_field(frontmatter, "author");
    let featured = extract_yaml_field(frontmatter, "featured");
    
    (title, difficulty, introduced_in_version, author, featured)
}

fn extract_yaml_field(yaml: &str, field: &str) -> String {
    let pattern = format!(r"{}:\s*(.+)", field);
    let regex = Regex::new(&pattern).ok();
    if let Some(regex) = regex {
        if let Some(captures) = regex.captures(yaml) {
            return captures[1].trim().trim_matches('"').to_string();
        }
    }
    "".to_string()
}

fn insert_component(conn: &rusqlite::Connection, name: &str, title: &str, difficulty: &str, introduced_in_version: &str, content: &str) -> rusqlite::Result<i64> {
    let overview_md = extract_section(content, "Overview").unwrap_or_default();
    let when_to_use_md = extract_section(content, "When to Use").unwrap_or_default();
    let basic_usage_sql = extract_sql_blocks(content).first().cloned().unwrap_or_default();
    let related_json = extract_section(content, "Related").unwrap_or_default();
    let changelog_md = extract_section(content, "Changelog").unwrap_or_default();

    conn.execute(
        "INSERT OR REPLACE INTO components 
         (name, icon, introduced_in_version, deprecated_in_version, difficulty,
          overview_md, when_to_use_md, basic_usage_sql, related_json, changelog_md,
          last_reviewed, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        [
            name,
            "",
            introduced_in_version,
            "",
            difficulty,
            &overview_md,
            &when_to_use_md,
            &basic_usage_sql,
            &related_json,
            &changelog_md,
            "",
            "",
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn insert_function(conn: &rusqlite::Connection, name: &str, title: &str, difficulty: &str, introduced_in_version: &str, content: &str) -> rusqlite::Result<i64> {
    let signature_md = extract_section(content, "Signature").unwrap_or_default();
    let description_md = extract_section(content, "Description").unwrap_or_default();
    let return_value_md = extract_section(content, "Return Value").unwrap_or_default();
    let security_notes_md = extract_section(content, "Security Notes").unwrap_or_default();
    let related_json = extract_section(content, "Related").unwrap_or_default();

    conn.execute(
        "INSERT OR REPLACE INTO functions 
         (name, namespace, icon, return_type, introduced_in_version, deprecated_in_version,
          category, difficulty, signature_md, description_md, return_value_md, security_notes_md,
          related_json, last_reviewed, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
        [
            name,
            "sqlpage",
            "",
            "",
            introduced_in_version,
            "",
            "",
            difficulty,
            &signature_md,
            &description_md,
            &return_value_md,
            &security_notes_md,
            &related_json,
            "",
            "",
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn insert_guide(conn: &rusqlite::Connection, slug: &str, title: &str, difficulty: &str, introduced_in_version: &str, content: &str) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT OR REPLACE INTO guides 
         (slug, title, difficulty, estimated_time, introduced_in_version,
          categories_json, tags_json, prerequisites_json, next_json, content_md,
          last_reviewed, last_updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        [
            slug,
            if title.is_empty() { slug } else { title },
            difficulty,
            "",
            introduced_in_version,
            "",
            "",
            "",
            "",
            content,
            "",
            "",
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn insert_blog_post(conn: &rusqlite::Connection, filename: &str, title: &str, author: &str, featured: &str, content: &str) -> rusqlite::Result<i64> {
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
    
    let featured_int = if featured == "true" { 1 } else { 0 };
    let final_title = if title.is_empty() { slug.clone() } else { title.to_string() };
    let final_author = author.to_string();

    conn.execute(
        "INSERT OR REPLACE INTO blog_posts 
         (slug, date, title, author, tags_json, categories_json, featured,
          preview_image, excerpt, content_md)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        [
            &slug,
            &date,
            &final_title,
            &final_author,
            "",
            "",
            &featured_int.to_string(),
            "",
            "",
            content,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn insert_configuration_page(conn: &rusqlite::Connection, slug: &str, title: &str, introduced_in_version: &str, content: &str) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT OR REPLACE INTO configuration_pages 
         (slug, title, introduced_in_version, categories_json, tags_json, content_md)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        [
            slug,
            if title.is_empty() { slug } else { title },
            introduced_in_version,
            "",
            "",
            content,
        ],
    )?;

    Ok(conn.last_insert_rowid())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Building SQLite database from documentation...");
    
    // Create database
    let conn = rusqlite::Connection::open("docs.sqlite")?;
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
            let (title, difficulty, introduced_in_version, author, featured) = parse_frontmatter(&content);
            
            let path_str = path.to_string_lossy();
            let name = path.file_stem().unwrap().to_string_lossy();
            
            if path_str.contains("/components/") {
                insert_component(&conn, &name, &title, &difficulty, &introduced_in_version, &content)?;
                processed += 1;
            } else if path_str.contains("/functions/") {
                insert_function(&conn, &name, &title, &difficulty, &introduced_in_version, &content)?;
                processed += 1;
            } else if path_str.contains("/guides/") {
                insert_guide(&conn, &name, &title, &difficulty, &introduced_in_version, &content)?;
                processed += 1;
            } else if path_str.contains("/blog/") {
                insert_blog_post(&conn, &name, &title, &author, &featured, &content)?;
                processed += 1;
            } else if path_str.contains("/configuration/") {
                insert_configuration_page(&conn, &name, &title, &introduced_in_version, &content)?;
                processed += 1;
            }
        }
    }
    
    println!("✅ Successfully built docs.sqlite with {} documents", processed);
    Ok(())
}