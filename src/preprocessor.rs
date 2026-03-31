use std::fs;
use std::collections::HashSet;
use std::path::PathBuf;

pub fn load_source(path: &str, loaded: &mut HashSet<String>) -> String {
    let mut p = path.replace(";", "").replace("\"", "").trim().to_string();
    if !p.ends_with(".qa") { p.push_str(".qa"); }
    
    let abs = fs::canonicalize(&p).unwrap_or(PathBuf::from(&p));
    let abs_str = abs.to_string_lossy().to_string();
    
    if loaded.contains(&abs_str) { return String::new(); }
    loaded.insert(abs_str);

    let mut clean = String::new();
    let content = fs::read_to_string(&p).expect("Error occured while reading content!");

    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("import") {
            clean.push_str(&load_source(&t[6..], loaded));
        } else {
            let (mut r, mut q) = (String::new(), false);
            let chars: Vec<char> = line.chars().collect();
            for i in 0..chars.len() {
                if chars[i] == '"' { q = !q; }
                if !q && (chars[i] == '#' || (chars[i] == '/' && i+1 < chars.len() && chars[i+1] == '/')) { break; }
                r.push(chars[i]);
            }
            clean.push_str(&r); clean.push('\n');
        }
    }
    clean
}
