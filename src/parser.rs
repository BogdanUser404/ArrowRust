// Copyright (c) Bogdan Yachmenev (2026)
// License: MIT

use std::sync::atomic::{AtomicUsize, Ordering};

static UNIQUE_ID: AtomicUsize = AtomicUsize::new(0);

fn get_unique_name() -> String {
    let id = UNIQUE_ID.fetch_add(1, Ordering::SeqCst);
    format!("_flow_item_{}", id)
}

pub fn compile_arrow_line(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() { return "\n".to_string(); }

    // --- 1. UTILIZER: -name> ---
    if trimmed.starts_with('-') && trimmed.contains('>') {
        let name = trimmed[1..].split('>').next().unwrap_or("").trim().trim_end_matches(';');
        return format!("drop({});\n", name);
    }

    // --- 2. DEFINITIONS & COMMENTS ---
    if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("//") {
        return format!("{}\n", trimmed);
    }

    // --- 3. REPEAT (<n>) ---
    if trimmed.contains('<') && trimmed.contains('>') && trimmed.contains('{') {
        if let (Some(s), Some(e)) = (trimmed.find('<'), trimmed.find('>')) {
            let data = trimmed[..s].trim();
            let count = trimmed[s + 1..e].trim();
            return format!("for idx in 0..{} {{ #[allow(unused_variables)] let it = {};\n", count, data);
        }
    }

    // --- 4. PIPELINE ENGINE (-> and +>) ---
    if trimmed.contains("->") || trimmed.contains("+>") {
        let mut line_str = trimmed.trim_end_matches(';').to_string();
        let mut buffer = String::new();
        let mut emitted = String::new();

        loop {
            let (content, arrow) = parse_step(&line_str);
            
            if arrow == "NULL" {
                if !buffer.is_empty() {
                    // Check if it's a variable assignment (including types like var: Type)
                    if is_pure_variable(&content) {
                        // Replace : with := for internal handling or just ensure let syntax
                        let var_part = content.replace(":", " : ");
                        emitted.push_str(&format!("let {} = {};", var_part, buffer));
                    } else {
                        emitted.push_str(&format!("{};", emit_final_call(&content, &buffer)));
                    }
                }
                break;
            }

            match arrow.as_str() {
                "->" => {
                    if buffer.is_empty() {
                        buffer = content; 
                    } else {
                        buffer = emit_final_call(&content, &buffer);
                    }
                },
                "+>" => {
                    buffer = content.trim_matches(|c| c == '[' || c == ']').to_string();
                },
                _ => {}
            }

            let pos = line_str.find(&arrow).unwrap();
            line_str = line_str[pos + arrow.len()..].trim().to_string();
            if line_str.is_empty() { break; }
        }
        return format!("{}\n", emitted);
    }

    format!("{}\n", trimmed)
}

fn parse_step(input: &str) -> (String, String) {
    let arrows = vec!["+>", "->"];
    for a in arrows {
        if let Some(p) = input.find(a) {
            return (input[..p].trim().to_string(), a.to_string());
        }
    }
    (input.to_string(), "NULL".to_string())
}

fn emit_final_call(func: &str, args: &str) -> String {
    let f = func.trim();
    if f.contains('!') {
        if f.contains("arg1") {
            f.replace("arg1", args)
        } else if let Some(pos) = f.rfind(')') {
            let (head, _) = f.split_at(pos);
            if head.trim().ends_with('(') { format!("{}{})", head, args) } 
            else { format!("{}, {})", head, args) }
        } else {
            format!("{}({})", f, args)
        }
    } else {
        if f.contains('(') {
            let base = f.trim_end_matches(')');
            if base.ends_with('(') { format!("{}{})", base, args) } 
            else { format!("{}, {})", base, args) }
        } else {
            format!("{}({})", f, args)
        }
    }
}

fn is_pure_variable(s: &str) -> bool {
    let s = s.trim();
    // A pure variable shouldn't contain parentheses, macros, or quotes
    // But it CAN contain a colon for type specification (e.g., args: Vec<String>)
    if s.contains('(') || s.contains('!') || s.contains('"') {
        return false;
    }
    
    // If it has spaces, it might be a multi-word expression (not a simple var)
    // UNLESS it's a type definition with spaces like "name : Type"
    if s.contains(' ') {
        return s.contains(':');
    }

    !s.is_empty()
}
