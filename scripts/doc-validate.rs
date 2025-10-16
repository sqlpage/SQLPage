#!/usr/bin/env cargo +nightly -Zscript
//! Validate SQLPage documentation against schema rules
//! 
//! This script validates all documentation files in the docs/ directory
//! against the schema defined in docs/_schema.md.

// Cargo.toml
[package]
name = "doc-validate"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
walkdir = "2.3"
regex = "1.10"
chrono = { version = "0.4", features = ["serde"] }

fn main() {
    println!("Documentation validation script");
}

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;

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
struct ValidationError {
    file: PathBuf,
    line: Option<usize>,
    message: String,
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

fn validate_version(version: &str) -> bool {
    let version_regex = Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
    version_regex.is_match(version)
}

fn validate_difficulty(difficulty: &str) -> bool {
    matches!(difficulty, "beginner" | "intermediate" | "advanced")
}

fn validate_required_sections(content: &str, doc_type: &DocType) -> Vec<String> {
    let mut missing = Vec::new();
    
    match doc_type {
        DocType::Component => {
            let required_sections = [
                "## Overview",
                "## When to Use", 
                "## Basic Usage",
                "## Top-Level Parameters",
                "## Row-Level Parameters",
                "## Examples",
                "## Related",
                "## Changelog",
            ];
            
            for section in &required_sections {
                if !content.contains(section) {
                    missing.push(section.to_string());
                }
            }
        },
        DocType::Function => {
            let required_sections = [
                "## Signature",
                "## Description",
                "## Parameters",
                "## Return Value",
                "## Security Notes",
                "## Examples",
                "## Related",
            ];
            
            for section in &required_sections {
                if !content.contains(section) {
                    missing.push(section.to_string());
                }
            }
        },
        DocType::Guide => {
            // Guides have flexible content structure
        },
        DocType::Blog => {
            // Blog posts have flexible content structure
        },
        DocType::Configuration => {
            // Configuration pages may need Settings section
            if content.contains("Settings") && !content.contains("## Settings") {
                missing.push("## Settings".to_string());
            }
        },
        DocType::Architecture => {
            // Architecture docs have flexible content structure
        },
    }
    
    missing
}

fn validate_sql_blocks(content: &str) -> Vec<String> {
    let mut errors = Vec::new();
    let sql_block_regex = Regex::new(r"```sql\n(.*?)\n```").unwrap();
    
    for cap in sql_block_regex.captures_iter(content) {
        let sql = &cap[1];
        if sql.trim().is_empty() {
            errors.push("Empty SQL code block found".to_string());
        }
    }
    
    errors
}

fn validate_doc_file(doc: &DocFile) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    
    // Validate frontmatter fields
    if let Some(version) = &doc.frontmatter.introduced_in_version {
        if !validate_version(version) {
            errors.push(ValidationError {
                file: doc.path.clone(),
                line: None,
                message: format!("Invalid version format: {}", version),
            });
        }
    }
    
    if let Some(version) = &doc.frontmatter.deprecated_in_version {
        if !validate_version(version) {
            errors.push(ValidationError {
                file: doc.path.clone(),
                line: None,
                message: format!("Invalid version format: {}", version),
            });
        }
    }
    
    if let Some(difficulty) = &doc.frontmatter.difficulty {
        if !validate_difficulty(difficulty) {
            errors.push(ValidationError {
                file: doc.path.clone(),
                line: None,
                message: format!("Invalid difficulty: {}", difficulty),
            });
        }
    }
    
    // Validate required sections
    let missing_sections = validate_required_sections(&doc.content, &doc.doc_type);
    for section in missing_sections {
        errors.push(ValidationError {
            file: doc.path.clone(),
            line: None,
            message: format!("Missing required section: {}", section),
        });
    }
    
    // Validate SQL blocks
    let sql_errors = validate_sql_blocks(&doc.content);
    for error in sql_errors {
        errors.push(ValidationError {
            file: doc.path.clone(),
            line: None,
            message: error,
        });
    }
    
    // Validate required frontmatter fields
    match doc.doc_type {
        DocType::Guide => {
            if doc.frontmatter.title.is_none() {
                errors.push(ValidationError {
                    file: doc.path.clone(),
                    line: None,
                    message: "Guide missing required 'title' field".to_string(),
                });
            }
        },
        DocType::Blog => {
            if doc.frontmatter.title.is_none() {
                errors.push(ValidationError {
                    file: doc.path.clone(),
                    line: None,
                    message: "Blog post missing required 'title' field".to_string(),
                });
            }
        },
        DocType::Configuration => {
            if doc.frontmatter.title.is_none() {
                errors.push(ValidationError {
                    file: doc.path.clone(),
                    line: None,
                    message: "Configuration page missing required 'title' field".to_string(),
                });
            }
        },
        _ => {}
    }
    
    errors
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
    println!("Validating SQLPage documentation...");
    
    let doc_files = find_doc_files()?;
    let mut all_errors = Vec::new();
    let mut names = HashSet::new();
    
    for file_path in doc_files {
        let doc = DocFile::from_path(&file_path)?;
        
        // Check for duplicate names/slugs
        let name = doc.path.file_stem().unwrap().to_string_lossy();
        if !names.insert(name.to_string()) {
            all_errors.push(ValidationError {
                file: doc.path.clone(),
                line: None,
                message: format!("Duplicate name/slug: {}", name),
            });
        }
        
        // Validate the document
        let errors = validate_doc_file(&doc);
        all_errors.extend(errors);
    }
    
    if all_errors.is_empty() {
        println!("✅ All documentation files are valid!");
        Ok(())
    } else {
        println!("❌ Found {} validation errors:", all_errors.len());
        for error in &all_errors {
            println!("  {}: {}", error.file.display(), error.message);
        }
        std::process::exit(1);
    }
}