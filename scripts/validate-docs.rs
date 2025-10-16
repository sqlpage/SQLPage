extern crate walkdir;
extern crate regex;

use std::collections::HashSet;
use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Validating SQLPage documentation...");
    
    let mut all_errors = Vec::new();
    let mut names = HashSet::new();
    
    // Find all markdown files
    for entry in WalkDir::new("docs") {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            // Skip the schema file
            if path.file_name().unwrap() == "_schema.md" {
                continue;
            }
            
            let content = fs::read_to_string(path)?;
            
            // Check for duplicate names/slugs
            let name = path.file_stem().unwrap().to_string_lossy();
            if !names.insert(name.to_string()) {
                all_errors.push(format!("Duplicate name/slug: {} in {}", name, path.display()));
            }
            
            // Basic validation - check for required sections based on path
            let path_str = path.to_string_lossy();
            if path_str.contains("/components/") {
                validate_component(&content, path, &mut all_errors);
            } else if path_str.contains("/functions/") {
                validate_function(&content, path, &mut all_errors);
            } else if path_str.contains("/guides/") {
                validate_guide(&content, path, &mut all_errors);
            } else if path_str.contains("/blog/") {
                validate_blog(&content, path, &mut all_errors);
            } else if path_str.contains("/configuration/") {
                validate_configuration(&content, path, &mut all_errors);
            }
        }
    }
    
    if all_errors.is_empty() {
        println!("✅ All documentation files are valid!");
        Ok(())
    } else {
        println!("❌ Found {} validation errors:", all_errors.len());
        for error in &all_errors {
            println!("  {}", error);
        }
        std::process::exit(1);
    }
}

fn validate_component(content: &str, path: &Path, errors: &mut Vec<String>) {
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
            errors.push(format!("{}: Missing required section: {}", path.display(), section));
        }
    }
    
    // Check for SQL code blocks
    let sql_regex = Regex::new(r"```sql\n.*?\n```").unwrap();
    if !sql_regex.is_match(content) {
        errors.push(format!("{}: Component should have SQL code blocks", path.display()));
    }
}

fn validate_function(content: &str, path: &Path, errors: &mut Vec<String>) {
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
            errors.push(format!("{}: Missing required section: {}", path.display(), section));
        }
    }
    
    // Check for SQL code blocks
    let sql_regex = Regex::new(r"```sql\n.*?\n```").unwrap();
    if !sql_regex.is_match(content) {
        errors.push(format!("{}: Function should have SQL code blocks", path.display()));
    }
}

fn validate_guide(content: &str, path: &Path, errors: &mut Vec<String>) {
    // Check for frontmatter with title
    if !content.starts_with("---\n") {
        errors.push(format!("{}: Guide should have YAML frontmatter", path.display()));
        return;
    }
    
    let end_marker = content.find("\n---\n");
    if end_marker.is_none() {
        errors.push(format!("{}: Guide frontmatter should end with ---", path.display()));
        return;
    }
    
    let frontmatter = &content[4..end_marker.unwrap()];
    if !frontmatter.contains("title:") {
        errors.push(format!("{}: Guide should have 'title' in frontmatter", path.display()));
    }
}

fn validate_blog(content: &str, path: &Path, errors: &mut Vec<String>) {
    // Check filename format (YYYY-MM-DD-slug.md)
    let filename = path.file_stem().unwrap().to_string_lossy();
    let date_regex = Regex::new(r"^\d{4}-\d{2}-\d{2}-").unwrap();
    if !date_regex.is_match(&filename) {
        errors.push(format!("{}: Blog post filename should be YYYY-MM-DD-slug.md", path.display()));
    }
    
    // Check for frontmatter with title
    if !content.starts_with("---\n") {
        errors.push(format!("{}: Blog post should have YAML frontmatter", path.display()));
        return;
    }
    
    let end_marker = content.find("\n---\n");
    if end_marker.is_none() {
        errors.push(format!("{}: Blog post frontmatter should end with ---", path.display()));
        return;
    }
    
    let frontmatter = &content[4..end_marker.unwrap()];
    if !frontmatter.contains("title:") {
        errors.push(format!("{}: Blog post should have 'title' in frontmatter", path.display()));
    }
}

fn validate_configuration(content: &str, path: &Path, errors: &mut Vec<String>) {
    // Check for frontmatter with title
    if !content.starts_with("---\n") {
        errors.push(format!("{}: Configuration page should have YAML frontmatter", path.display()));
        return;
    }
    
    let end_marker = content.find("\n---\n");
    if end_marker.is_none() {
        errors.push(format!("{}: Configuration page frontmatter should end with ---", path.display()));
        return;
    }
    
    let frontmatter = &content[4..end_marker.unwrap()];
    if !frontmatter.contains("title:") {
        errors.push(format!("{}: Configuration page should have 'title' in frontmatter", path.display()));
    }
}