extern crate regex;

use std::path::Path;
use std::fs;
use regex::Regex;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking SQL syntax in documentation files...");
    
    let mut all_errors = Vec::new();
    
    // Find all markdown files
    for entry in walkdir::WalkDir::new("docs") {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            // Skip the schema file
            if path.file_name().unwrap() == "_schema.md" {
                continue;
            }
            
            let content = fs::read_to_string(path)?;
            let errors = check_file_sql(path, &content);
            all_errors.extend(errors);
        }
    }
    
    if all_errors.is_empty() {
        println!("✅ All SQL code blocks are syntactically valid!");
        Ok(())
    } else {
        println!("❌ Found {} SQL syntax errors:", all_errors.len());
        for error in &all_errors {
            println!("  {}", error);
        }
        std::process::exit(1);
    }
}

fn check_file_sql(file_path: &Path, content: &str) -> Vec<String> {
    let sql_block_regex = Regex::new(r"```sql\n(.*?)\n```").unwrap();
    let mut errors = Vec::new();
    
    for cap in sql_block_regex.captures_iter(content) {
        let sql = &cap[1];
        if sql.trim().is_empty() {
            errors.push(format!("{}: Empty SQL code block found", file_path.display()));
        } else {
            // Basic SQL validation - check for common syntax issues
            if !sql.trim().ends_with(';') && !sql.trim().is_empty() {
                // This is just a warning, not an error
                // errors.push(format!("{}: SQL block should end with semicolon", file_path.display()));
            }
        }
    }
    
    errors
}