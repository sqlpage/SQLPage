#!/usr/bin/env cargo +nightly -Zscript
//! Validate SQLPage documentation against schema rules

// Cargo.toml
[package]
name = "doc-validate-simple"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_yaml = "0.9"
walkdir = "2.3"
regex = "1.10"

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;

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

#[derive(Debug)]
struct ValidationError {
    file: PathBuf,
    message: String,
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

fn validate_version(version: &str) -> bool {
    let version_regex = Regex::new(r"^\d+\.\d+\.\d+$").unwrap();
    version_regex.is_match(version)
}

fn validate_difficulty(difficulty: &str) -> bool {
    matches!(difficulty, "beginner" | "intermediate" | "advanced")
}

fn validate_required_sections(content: &str, doc_type: &str) -> Vec<String> {
    let mut missing = Vec::new();
    
    match doc_type {
        "component" => {
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
        "function" => {
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
        _ => {}
    }
    
    missing
}

fn validate_doc_file(path: &Path, frontmatter: &Frontmatter, content: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    
    // Determine document type
    let path_str = path.to_string_lossy();
    let doc_type = if path_str.contains("/components/") {
        "component"
    } else if path_str.contains("/functions/") {
        "function"
    } else if path_str.contains("/guides/") {
        "guide"
    } else if path_str.contains("/blog/") {
        "blog"
    } else if path_str.contains("/configuration/") {
        "configuration"
    } else {
        "other"
    };
    
    // Validate frontmatter fields
    if let Some(version) = &frontmatter.introduced_in_version {
        if !validate_version(version) {
            errors.push(ValidationError {
                file: path.to_path_buf(),
                message: format!("Invalid version format: {}", version),
            });
        }
    }
    
    if let Some(version) = &frontmatter.deprecated_in_version {
        if !validate_version(version) {
            errors.push(ValidationError {
                file: path.to_path_buf(),
                message: format!("Invalid version format: {}", version),
            });
        }
    }
    
    if let Some(difficulty) = &frontmatter.difficulty {
        if !validate_difficulty(difficulty) {
            errors.push(ValidationError {
                file: path.to_path_buf(),
                message: format!("Invalid difficulty: {}", difficulty),
            });
        }
    }
    
    // Validate required sections
    let missing_sections = validate_required_sections(content, doc_type);
    for section in missing_sections {
        errors.push(ValidationError {
            file: path.to_path_buf(),
            message: format!("Missing required section: {}", section),
        });
    }
    
    // Validate required frontmatter fields
    match doc_type {
        "guide" | "blog" | "configuration" => {
            if frontmatter.title.is_none() {
                errors.push(ValidationError {
                    file: path.to_path_buf(),
                    message: format!("{} missing required 'title' field", doc_type),
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
        let content = fs::read_to_string(&file_path)?;
        let (frontmatter, content) = parse_frontmatter(&content)?;
        
        // Check for duplicate names/slugs
        let name = file_path.file_stem().unwrap().to_string_lossy();
        if !names.insert(name.to_string()) {
            all_errors.push(ValidationError {
                file: file_path.clone(),
                message: format!("Duplicate name/slug: {}", name),
            });
        }
        
        // Validate the document
        let errors = validate_doc_file(&file_path, &frontmatter, &content);
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