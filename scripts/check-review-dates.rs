extern crate walkdir;
extern crate regex;

use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use std::collections::HashMap;

fn parse_frontmatter(content: &str) -> HashMap<String, String> {
    let mut fields = HashMap::new();
    
    if !content.starts_with("---\n") {
        return fields;
    }
    
    let end_marker = match content.find("\n---\n") {
        Some(pos) => pos,
        None => return fields,
    };
    
    let frontmatter = &content[4..end_marker];
    
    // Simple parsing - extract key-value pairs
    for line in frontmatter.lines() {
        if line.contains(':') {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let value = parts[1].trim().trim_matches('"');
                fields.insert(key.to_string(), value.to_string());
            }
        }
    }
    
    fields
}

fn is_date_old(date_str: &str, days_threshold: i64) -> bool {
    if date_str.is_empty() {
        return true; // Missing date is considered old
    }
    
    // Parse ISO8601 date (YYYY-MM-DD)
    let date_regex = Regex::new(r"^(\d{4})-(\d{2})-(\d{2})$").unwrap();
    if let Some(captures) = date_regex.captures(date_str) {
        let year: i32 = captures[1].parse().unwrap_or(0);
        let month: u32 = captures[2].parse().unwrap_or(0);
        let day: u32 = captures[3].parse().unwrap_or(0);
        
        // Simple date comparison (not perfect but good enough for this check)
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        // Approximate days since epoch for the date
        let doc_date_epoch = (year as i64 - 1970) * 365 + (month as i64 - 1) * 30 + day as i64;
        let days_old = (now / 86400) - doc_date_epoch;
        
        return days_old > days_threshold;
    }
    
    true // Invalid date format is considered old
}

fn check_doc_file(path: &std::path::Path, days_threshold: i64) -> Vec<String> {
    let mut issues = Vec::new();
    
    if let Ok(content) = fs::read_to_string(path) {
        let fields = parse_frontmatter(&content);
        
        let last_reviewed = fields.get("last_reviewed").map(|s| s.as_str()).unwrap_or("");
        let last_updated = fields.get("last_updated").map(|s| s.as_str()).unwrap_or("");
        
        if last_reviewed.is_empty() && last_updated.is_empty() {
            issues.push(format!("Missing both 'last_reviewed' and 'last_updated' fields"));
        } else if last_reviewed.is_empty() {
            issues.push(format!("Missing 'last_reviewed' field"));
        } else if is_date_old(last_reviewed, days_threshold) {
            issues.push(format!("'last_reviewed' is older than {} days: {}", days_threshold, last_reviewed));
        }
        
        if !last_updated.is_empty() && is_date_old(last_updated, days_threshold) {
            issues.push(format!("'last_updated' is older than {} days: {}", days_threshold, last_updated));
        }
    }
    
    issues
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let days_threshold = 90; // 3 months
    println!("Checking documentation review dates (older than {} days)...", days_threshold);
    
    let mut all_issues = Vec::new();
    
    for entry in WalkDir::new("docs") {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
            // Skip the schema file
            if path.file_name().unwrap() == "_schema.md" {
                continue;
            }
            
            let issues = check_doc_file(path, days_threshold);
            for issue in issues {
                all_issues.push(format!("{}: {}", path.display(), issue));
            }
        }
    }
    
    if all_issues.is_empty() {
        println!("✅ All documentation is up to date!");
        Ok(())
    } else {
        println!("❌ Found {} documentation review issues:", all_issues.len());
        for issue in &all_issues {
            println!("  {}", issue);
        }
        std::process::exit(1);
    }
}