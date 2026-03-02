// Copyright (c) Bogdan Yachmenev (2026)
// License: MIT

pub fn compile_arrow_line(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() { return "\n".to_string(); }

    // --- 1. SKIP DEFINITIONS ---
    // If it's a function or a comment, don't look for arrows
    if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") || trimmed.starts_with("//") {
        return format!("{}\n", trimmed);
    }

    // --- 2. REPEAT LOGIC ---
    if trimmed.contains('<') && trimmed.contains('>') && trimmed.contains('{') {
        if let (Some(s), Some(e)) = (trimmed.find('<'), trimmed.find('>')) {
            let data = trimmed[..s].trim();
            let count = trimmed[s + 1..e].trim();
            return format!("for idx in 0..{} {{ let it = {};\n", count, data);
        }
    }

    // --- 3. PIPELINE LOGIC (+>) ---
    if trimmed.contains("+>") {
        let parts: Vec<&str> = trimmed.split("+>").map(|s| s.trim()).collect();
        if parts.len() == 2 {
            let args = parts[0].trim_matches(|c| c == '[' || c == ']');
            let func = parts[1].trim_end_matches(';');
            return format!("{};\n", emit_clean_call(func, args));
        }
    }

    // --- 4. PIPELINE LOGIC (->) ---
    if trimmed.contains("->") {
        let parts: Vec<&str> = trimmed.split("->").map(|s| s.trim()).collect();
        let mut res = parts[0].to_string();
        for i in 1..parts.len() {
            let node = parts[i].trim_end_matches(';');
            res = format!("{}({})", node, res);
        }
        return format!("{};\n", res);
    }

    format!("{}\n", trimmed)
}

fn emit_clean_call(func: &str, args: &str) -> String {
    let mut f = func.to_string();
    for i in 1..10 { f = f.replace(&format!("arg{}", i), ""); }
    f = f.replace(", ,", "").replace(",,", ",");
    
    if let Some(pos) = f.rfind(')') {
        let (head, _) = f.split_at(pos);
        let h = head.trim().trim_end_matches(',').trim();
        if h.ends_with('(') { format!("{}{})", h, args) } else { format!("{}, {})", h, args) }
    } else {
        format!("{}({})", f, args)
    }
}
