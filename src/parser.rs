// Copyright (c) Bogdan Yachmenev (2026)
// License: MIT

pub fn compile_arrow_line(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() { return "\n".to_string(); }

    // --- 1. SMART SKIP ---
    if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") 
        || trimmed.starts_with("use ") || trimmed.starts_with("//") 
        || (trimmed.starts_with("let ") && !trimmed.contains("->")) {
        return format!("{}\n", trimmed);
    }

    // --- 2. UTILIZER ---
    if trimmed.starts_with('-') && trimmed.contains('>') {
        let name = trimmed[1..].split('>').next().unwrap_or("").trim().trim_end_matches(';');
        return format!("drop({});\n", name);
    }

    // --- 3. PIPELINE ENGINE ---
    if trimmed.contains("->") || trimmed.contains("+>") {
        let mut line_str = trimmed.trim_end_matches(';').to_string();
        let mut buffer = String::new();
        let mut emitted = String::new();

        loop {
            let (content, arrow) = parse_step(&line_str);
            if arrow == "NULL" {
                if !buffer.is_empty() {
                    if is_pure_variable(&content) {
                        emitted.push_str(&format!("let {} = {};", content.replace(":", " : "), buffer));
                    } else {
                        emitted.push_str(&format!("{};", emit_final_call(&content, &buffer)));
                    }
                }
                break;
            }
            match arrow.as_str() {
                "->" => {
                    if buffer.is_empty() { buffer = content; }
                    else { buffer = emit_final_call(&content, &buffer); }
                },
                "+>" => { buffer = content.trim_matches(|c| c == '[' || c == ']').to_string(); },
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

fn emit_final_call(func: &str, args: &str) -> String {
    let f = func.trim();
    let methods = vec!["unwrap", "collect", "parse", "trim", "to_string", "len", "as_str"];
    
    let is_method = methods.iter().any(|&m| f.starts_with(m));

    if f.contains('!') {
        // Логика макросов (оставляем как есть)
        if f.contains("arg1") { f.replace("arg1", args) }
        else if let Some(pos) = f.rfind(')') {
            let (head, _) = f.split_at(pos);
            if head.trim().ends_with('(') { format!("{}{})", head, args) } 
            else { format!("{}, {})", head, args) }
        } else { format!("{}({})", f, args) }
    } else if is_method {
        // FIX: Если это метод, и у него нет скобок, добавляем их
        let method_call = if f.contains('(') { f.to_string() } else { format!("{}()", f) };
        format!("{}.{}", args, method_call)
    } else {
        // Обычная функция (оставляем как есть)
        if f.contains('(') {
            let base = f.trim_end_matches(')');
            if base.ends_with('(') { format!("{}{})", base, args) } 
            else { format!("{}, {})", base, args) }
        } else { format!("{}({})", f, args) }
    }
}


fn parse_step(input: &str) -> (String, String) {
    let arrows = vec!["+>", "->"];
    for a in arrows { if let Some(p) = input.find(a) { return (input[..p].trim().to_string(), a.to_string()); } }
    (input.to_string(), "NULL".to_string())
}

fn is_pure_variable(s: &str) -> bool {
    let s = s.trim();
    if s.contains('(') || s.contains('!') || s.contains('"') { return false; }
    if s.contains(' ') { return s.contains(':'); }
    !s.is_empty()
}
