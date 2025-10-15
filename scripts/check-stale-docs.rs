extern crate walkdir;
extern crate regex;

use std::fs;
use walkdir::WalkDir;
use regex::Regex;

fn find_functions_in_code() -> Vec<String> {
    let mut functions = Vec::new();
    
    // Look for functions in the functions.rs file
    if let Ok(content) = fs::read_to_string("src/webserver/database/sqlpage_functions/functions.rs") {
        let function_regex = Regex::new(r"make_function!\s*\(\s*(\w+)").unwrap();
        for cap in function_regex.captures_iter(&content) {
            functions.push(cap[1].to_string());
        }
    }
    
    functions
}

fn find_components_in_code() -> Vec<String> {
    let mut components = Vec::new();
    
    // Look for template files
    for entry in WalkDir::new("sqlpage/templates") {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "handlebars") {
                if let Some(name) = path.file_stem() {
                    components.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    
    components
}

fn find_documented_functions() -> Vec<String> {
    let mut functions = Vec::new();
    
    for entry in WalkDir::new("docs/functions") {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                if let Some(name) = path.file_stem() {
                    functions.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    
    functions
}

fn find_documented_components() -> Vec<String> {
    let mut components = Vec::new();
    
    for entry in WalkDir::new("docs/components") {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                if let Some(name) = path.file_stem() {
                    components.push(name.to_string_lossy().to_string());
                }
            }
        }
    }
    
    components
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Checking for stale documentation...");
    
    let mut errors = Vec::new();
    
    // Check functions
    let code_functions = find_functions_in_code();
    let doc_functions = find_documented_functions();
    
    for func in &code_functions {
        if !doc_functions.contains(func) {
            errors.push(format!("Function '{}' is implemented but not documented", func));
        }
    }
    
    for func in &doc_functions {
        if !code_functions.contains(func) {
            errors.push(format!("Function '{}' is documented but not implemented", func));
        }
    }
    
    // Check components
    let code_components = find_components_in_code();
    let doc_components = find_documented_components();
    
    for comp in &code_components {
        if !doc_components.contains(comp) {
            errors.push(format!("Component '{}' is implemented but not documented", comp));
        }
    }
    
    for comp in &doc_components {
        if !code_components.contains(comp) {
            errors.push(format!("Component '{}' is documented but not implemented", comp));
        }
    }
    
    if errors.is_empty() {
        println!("✅ All code and documentation are in sync!");
        Ok(())
    } else {
        println!("❌ Found {} stale documentation issues:", errors.len());
        for error in &errors {
            println!("  {}", error);
        }
        std::process::exit(1);
    }
}