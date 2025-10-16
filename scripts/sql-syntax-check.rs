#!/usr/bin/env cargo +nightly -Zscript
//! Check SQL syntax in documentation files
//! 
//! This script extracts SQL code blocks from documentation files
//! and validates their syntax using sqlparser.

// Cargo.toml
[package]
name = "sql-syntax-check"
version = "0.1.0"
edition = "2021"

[dependencies]
sqlparser = "0.45"
walkdir = "2.3"
regex = "1.10"

use std::path::Path;
use walkdir::WalkDir;
use regex::Regex;
use std::fs;

#[derive(Debug)]
struct SqlError {
    file: String,
    sql: String,
    error: String,
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

fn validate_sql_syntax(sql: &str) -> Result<(), String> {
    use sqlparser::dialect::GenericDialect;
    use sqlparser::parser::Parser;
    
    let dialect = GenericDialect {};
    let mut parser = Parser::new(&dialect);
    
    match parser.try_with_sql(sql) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("SQL syntax error: {}", e)),
    }
}

fn check_file_sql(file_path: &Path) -> Vec<SqlError> {
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(_) => return vec![],
    };
    
    let sql_blocks = extract_sql_blocks(&content);
    let mut errors = Vec::new();
    
    for sql in sql_blocks {
        if let Err(error) = validate_sql_syntax(&sql) {
            errors.push(SqlError {
                file: file_path.to_string_lossy().to_string(),
                sql: sql.clone(),
                error,
            });
        }
    }
    
    errors
}

fn find_doc_files() -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    
    for entry in WalkDir::new("docs") {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                files.push(path.to_path_buf());
            }
        }
    }
    
    files
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking SQL syntax in documentation files...");
    
    let doc_files = find_doc_files();
    let mut all_errors = Vec::new();
    
    for file_path in doc_files {
        let errors = check_file_sql(&file_path);
        all_errors.extend(errors);
    }
    
    if all_errors.is_empty() {
        println!("✅ All SQL code blocks are syntactically valid!");
        Ok(())
    } else {
        println!("❌ Found {} SQL syntax errors:", all_errors.len());
        for error in &all_errors {
            println!("  File: {}", error.file);
            println!("  SQL: {}", error.sql);
            println!("  Error: {}", error.error);
            println!();
        }
        std::process::exit(1);
    }
}