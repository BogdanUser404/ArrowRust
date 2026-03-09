// macro_processor.rs
use std::collections::{HashMap, HashSet};
use std::fs;

pub fn process_dialect(input: String, lib_root: &str) -> String {
    let macros = collect_definitions(&input);
    let mut output = String::with_capacity(input.len());
    let mut visited = HashSet::new();
    let mut enabled = false;

    for line in input.lines() {
        let trimmed = line.trim_start();

        if trimmed.starts_with("//#") {
            if trimmed.starts_with("//#ENABLE_MACRO") {
                enabled = true;
                output.push_str(line);
            } else if trimmed.starts_with("//#DISABLE_MACRO") {
                enabled = false;
                output.push_str(line);
            } else if trimmed.starts_with("//#INCLUDE") {
                // Вырезаем путь через итератор символов, чтобы не упасть на Юникоде
                let path_part = trimmed.chars().skip(10).collect::<String>().trim().to_string();
                handle_include(&path_part, lib_root, &mut visited, enabled, &macros, &mut output);
                continue; 
            } else if trimmed.starts_with("//#DEFINE") {
                // Дефайны в итоговый код не пускаем
                continue;
            } else {
                output.push_str(line);
            }
        } else if enabled {
            apply_macros(line, &macros, &mut output);
        } else {
            output.push_str(line);
        }
        output.push('\n');
    }
    output
}

fn collect_definitions(input: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in input.lines() {
        let t = line.trim_start();
        if t.starts_with("//#DEFINE") {
            let parts: Vec<String> = t.split_whitespace().map(|s| s.to_string()).collect();
            if parts.len() >= 3 {
                let value = parts[2..].join(" ");
                map.insert(parts[1].clone(), value);
            }
        }
    }
    map
}

fn handle_include(
    path: &str,
    lib_root: &str,
    visited: &mut HashSet<String>,
    enabled: bool,
    macros: &HashMap<String, String>,
    out: &mut String,
) {
    // Формируем полный путь
    let full_path = if path.starts_with('/') {
        path.to_string()
    } else {
        // Если lib_root не заканчивается на '/', добавляем
        let root = if lib_root.ends_with('/') || lib_root.is_empty() {
            lib_root.to_string()
        } else {
            format!("{}/", lib_root)
        };
        format!("{}{}", root, path)
    };

    // Защита от повторного включения
    if !visited.insert(full_path.clone()) {
        return;
    }

    // Читаем файл
    if let Ok(content) = fs::read_to_string(&full_path) {
        out.push_str(&format!("// --- Start of Include: {} ---\n", full_path));
        for inc_line in content.lines() {
            if enabled && !inc_line.trim_start().starts_with("//!") {
                apply_macros(inc_line, macros, out);
            } else {
                out.push_str(inc_line);
            }
            out.push('\n');
        }
        out.push_str(&format!("// --- End of Include: {} ---\n", full_path));
    } else {
        out.push_str(&format!("// ERROR: Could not find include file: {}\n", path));
        eprintln!("Error: Cannot find file at {}", full_path);
    }
    // visited не удаляем, чтобы избежать зацикливания
}

fn apply_macros(line: &str, macros: &HashMap<String, String>, out: &mut String) {
    let mut current_word = String::new();

    for c in line.chars() {
        if c.is_alphanumeric() || c == '_' {
            current_word.push(c);
        } else {
            if !current_word.is_empty() {
                let replacement = macros.get(&current_word).cloned().unwrap_or(current_word.clone());
                out.push_str(&replacement);
                current_word.clear();
            }
            out.push(c);
        }
    }

    if !current_word.is_empty() {
        let replacement = macros.get(&current_word).cloned().unwrap_or(current_word);
        out.push_str(&replacement);
    }
}