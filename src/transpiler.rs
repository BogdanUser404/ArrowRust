// transpiler.rs
// Copyright (c) Bogdan Yachmenev (2026)
// License: MIT
// Version: 0.2.8 (все срезы через .get(), без паники)

use std::sync::atomic::{AtomicBool, Ordering};

static IGNORE_MODE: AtomicBool = AtomicBool::new(false);

pub fn transpile_line(line: &str) -> String {
    let trimmed = line.trim();

    if trimmed == "//#ARROW_IGNORE" {
        IGNORE_MODE.store(true, Ordering::Relaxed);
        return format!("{}\n", line);
    }
    if trimmed == "//#ARROW_NO_IGNORE" {
        IGNORE_MODE.store(false, Ordering::Relaxed);
        return format!("{}\n", line);
    }
    if IGNORE_MODE.load(Ordering::Relaxed) {
        return format!("{}\n", line);
    }

    let indent = line.chars().take_while(|&c| c == ' ').count();
    let code = line.trim();

    if code.is_empty() || code.starts_with("//!") {
        return format!("{}\n", line);
    }

    if code == "pass" {
        return format!("{}// pass\n", " ".repeat(indent));
    }

    if code == "import std;" {
        return format!(
            "{}use std::collections::*;\n{}use std::io::*;\n{}use std::fs::*;\n",
            " ".repeat(indent),
            " ".repeat(indent),
            " ".repeat(indent)
        );
    }

    // --- Отделяем комментарий (все операции через .get()) ---
    let (clean_code, comment) = if let Some(pos) = code.find("//") {
        let before_slash = code.get(..pos).unwrap_or("");
        if pos > 0 && before_slash.ends_with('!') {
            (code, "")
        } else {
            let c = code.get(..pos).unwrap_or("").trim_end();
            let cm = code.get(pos..).unwrap_or("");
            (c, cm)
        }
    } else {
        (code, "")
    };

    let mut current = clean_code
        .replace("ValueRes<", "Result<Option<")
        .replace("Value<", "Option<");

    // --- Специальные декларации ---
    if current.starts_with("tuple ") {
        return format!("{}{}{}\n", " ".repeat(indent), handle_tuple_decl(&current), comment);
    }
    if current.starts_with("customtype ") {
        return format!("{}{}{}\n", " ".repeat(indent), handle_custom_type_decl(&current), comment);
    }

    // --- Инициализация словаря ---
    if current.contains(": dict") && current.contains('{') && current.contains('=') {
        let dict_block = handle_dict_init(&current);
        return format!("{}{}{}\n", " ".repeat(indent), dict_block, comment);
    }

    current = current.replace(": dict", ": std::collections::HashMap");

    // --- input() ---
    if current.contains("input()") {
        current = handle_input_call(&current);
    }

    // --- cp-> -> .clone() ---
    current = replace_cp_arrow(&current);

    let has_semicolon = current.ends_with(';');
    let clean_current = current.trim_end_matches(';').to_string();

    let processed = match () {
        _ if should_skip(&clean_current) => clean_current.to_string(),
        _ if is_utilizer(&clean_current) => handle_utilizer(&clean_current),
        _ if clean_current.starts_with("repeat") => handle_repeat(&clean_current),
        _ if has_pipeline(&clean_current) => {
            let res = handle_pipeline(&clean_current);
            if !has_semicolon
                && !res.contains('=')
                && !res.is_empty()
                && !["break", "continue", "return", "let", "match", "{", "drop"]
                    .iter()
                    .any(|&x| res.starts_with(x))
            {
                format!("return {}", res)
            } else {
                res
            }
        }
        _ => clean_current.to_string(),
    };

    let suffix = if has_semicolon && !processed.ends_with(';') && !processed.ends_with('}') {
        ";"
    } else {
        ""
    };
    format!("{}{}{}{}\n", " ".repeat(indent), processed, suffix, comment)
}

// --- Замена cp-> на .clone() (безопасная, через char_indices, срезы через .get()) ---
fn replace_cp_arrow(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.char_indices().peekable();
    let mut last = 0;

    while let Some((i, c)) = chars.next() {
        if c.is_alphabetic() || c == '_' {
            let start = i;
            let mut end = i;
            while let Some(&(j, ch)) = chars.peek() {
                if ch.is_alphanumeric() || ch == '_' {
                    end = j;
                    chars.next();
                } else {
                    break;
                }
            }
            let ident = s.get(start..=end).unwrap_or("");
            if let Some(&(_, '.')) = chars.peek() {
                let mut after_dot = chars.clone();
                after_dot.next();
                if let Some((_, 'c')) = after_dot.peek() {
                    let mut after_c = after_dot.clone();
                    after_c.next();
                    if let Some((_, 'p')) = after_c.peek() {
                        let mut after_p = after_c.clone();
                        after_p.next();
                        if let Some((_, '-')) = after_p.peek() {
                            let mut after_minus = after_p.clone();
                            after_minus.next();
                            if let Some((_, '>')) = after_minus.peek() {
                                result.push_str(s.get(last..start).unwrap_or(""));
                                result.push_str(ident);
                                result.push_str(".clone()");
                                for _ in 0..5 { chars.next(); }
                                last = chars.peek().map(|&(pos, _)| pos).unwrap_or(s.len());
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }
    if last < s.len() {
        result.push_str(s.get(last..).unwrap_or(""));
    }
    result
}

// --- Вспомогательные ---
fn should_skip(s: &str) -> bool {
    if s.starts_with("fn ") || s.starts_with("pub fn ")
        || s.starts_with("use ") || s.starts_with("extern ") || s.starts_with("mod ")  || s.starts_with("unsafe")
        || s.starts_with("impl ") || s.starts_with("pub impl ") 
        || s.starts_with("struct ") || s.starts_with("enum ") || s.starts_with("trait ")
    {
        return true;
    }
    let p = ["if ", "else ", "match ", "while ", "for ", "loop "];
    p.iter().any(|&x| s.starts_with(x)) && !has_pipeline(s)
}

fn is_utilizer(s: &str) -> bool {
    s.starts_with('-') && !s.starts_with("->") && s.contains('>')
}

fn has_pipeline(s: &str) -> bool {
    ["->", "+>", "!->", "&->", "mv->"].iter().any(|&a| s.contains(a))
}

fn handle_utilizer(s: &str) -> String {
    let id = s[1..].split('>').next().unwrap_or("").trim();
    format!("drop({})", id)
}

// --- Пайплайн со всеми срезами через .get() ---
fn handle_pipeline(s: &str) -> String {
    let mut current = s.to_string();
    let mut acc = String::new();
    let mut target = String::new();

    if let Some(eq_pos) = current.find('=') {
        let left = current.get(..eq_pos).unwrap_or("");
        if !left.contains("->") && !left.contains("+>") && !current.get(eq_pos..).unwrap_or("").starts_with("==") {
            target = format!("{} = ", left.trim());
            if let Some(rest) = current.get(eq_pos + 1..) {
                current = rest.trim().to_string();
            } else {
                return format!("{}{}", target, acc);
            }
        }
    }

    loop {
        let (content, arrow) = parse_step(&current);

        match arrow.as_str() {
            "->|" => return format!("{}break;", target),
            "->^" => return format!("{}continue;", target),
            "->()" => return format!("{}return ();", target),

            "!->" => {
                let pos = match current.find("!->") {
                    Some(p) => p,
                    None => break,
                };
                let after = match current.get(pos + 3..) {
                    Some(a) => a,
                    None => break,
                };
                let (_msg, next_arrow) = parse_step(after);
                let msg_end = if next_arrow == "NONE" {
                    after.len()
                } else {
                    match after.find(&next_arrow) {
                        Some(p) => p,
                        None => after.len(),
                    }
                };
                let msg = match after.get(..msg_end) {
                    Some(m) => m.trim().to_string(),
                    None => String::new(),
                };
                current = match after.get(msg_end..) {
                    Some(r) => r.trim().to_string(),
                    None => String::new(),
                };

                let base = if acc.is_empty() { content.clone() } else { acc.clone() };
                acc = format!("{}.expect({})", base, msg);
                continue;
            }

            "&->" => {
                let prev = if acc.is_empty() { content.clone() } else { acc.clone() };
                acc = format!("&{}", prev);
            }

            "mv->" => {
                let var = if acc.is_empty() { content.clone() } else { acc.clone() };
                let body = if acc.is_empty() { content.clone() } else { wrap_call(&content, &acc) };
                acc = format!("{{ let __tmp = {}; drop({}); __tmp }}", body, var);
            }

            "+>" => {
                let inner = content.trim_matches(|c| c == '[' || c == ']');
                if s.contains("set_bit") {
                    let p: Vec<&str> = inner.split(',').map(|x| x.trim()).collect();
                    if p.len() == 3 {
                        let var_clean = p[0].replace('&', "");
                        acc = format!(
                            "(({} & !(1 << {})) | (({} & 1) << {}))",
                            var_clean, p[1], p[2], p[1]
                        );
                    }
                } else {
                    acc = inner.to_string();
                }
            }

            "->" => {
                acc = if acc.is_empty() { content } else { wrap_call(&content, &acc) };
            }

            "NONE" => {
                let mut content = content;
                let has_semicolon = content.ends_with(';');
                if has_semicolon {
                    if let Some(c) = content.get(..content.len() - 1) {
                        content = c.trim().to_string();
                    } else {
                        content = content.trim().to_string();
                    }
                }
                let is_identifier = !content.is_empty() && content.chars().all(|c| c.is_alphanumeric() || c == '_');
                if is_identifier && target.is_empty() {
                    let res = if acc.is_empty() { content } else { format!("{} = {}", content, acc) };
                    return format!("{}{}", target, res);
                } else {
                    let res = if acc.is_empty() { content } else { wrap_call(&content, &acc) };
                    return format!("{}{}", target, res);
                }
            }

            _ => {}
        }

        if arrow != "NONE" && arrow != "!->" {
            let pos = match current.find(&arrow) {
                Some(p) => p,
                None => break,
            };
            if let Some(rest) = current.get(pos + arrow.len()..) {
                current = rest.trim().to_string();
            } else {
                break;
            }
            if current.is_empty() { break; }
        }
    }

    format!("{}{}", target, acc)
}
fn wrap_call(f: &str, args: &str) -> String {
    let f = f.trim();
    if f.is_empty() || f == "set_bit" {
        return args.to_string();
    }

    if f.ends_with('!') {
        // Макросы вывода требуют форматную строку
        if f == "println!" || f == "print!" || f == "format!" {
            return format!("{}(\"{{}}\", {})", f, args);
        }
        return format!("{}({})", f, args);
    }

    let rust_methods = [
        "unwrap", "collect", "parse", "trim", "to_string", "len", "as_str", "expect", "insert", "push",
    ];
    if rust_methods.iter().any(|&m| f.contains(m)) {
        if f.contains('(') {
            format!("{}.{}", args, f)
        } else {
            format!("{}.{}()", args, f)
        }
    } else {
        format_standard_call(f, args)
    }
}
fn handle_tuple_decl(s: &str) -> String {
    let mut content = s.get("tuple ".len()..).unwrap_or("").trim().to_string();
    if content.starts_with("clone ") { content = content.get("clone ".len()..).unwrap_or("").trim().to_string(); }
    let (name, types) = match content.find('(') {
        Some(pos) => (
            content.get(..pos).unwrap_or("").trim(),
            content.get(pos..).unwrap_or("").trim(),
        ),
        None => (content.trim(), "()"),
    };
    format!("type {} = {};", name, types.trim_end_matches(';'))
}

fn handle_custom_type_decl(s: &str) -> String {
    let mut content = s.get("customtype ".len()..).unwrap_or("").trim().to_string();
    if content.starts_with("clone ") { content = content.get("clone ".len()..).unwrap_or("").trim().to_string(); }
    let parts: Vec<&str> = content.split_whitespace().collect();
    let name = parts.get(0).unwrap_or(&"Unknown");
    let base = parts.get(1).unwrap_or(&"i32").trim_end_matches(';');
    format!("type {} = {};", name, base)
}

fn handle_input_call(s: &str) -> String {
    if let Some(pos) = s.find("->") {
        if let Some(after_arrow) = s.get(pos + 2..) {
            let var = after_arrow.trim().trim_end_matches(';');
            return format!(
                "{{ let mut __b = String::new(); std::io::stdin().read_line(&mut __b).unwrap(); {} = __b.trim().to_string(); }}",
                var
            );
        }
    }
    s.to_string()
}

fn handle_dict_init(s: &str) -> String {
    if let (Some(open), Some(close)) = (s.find('{'), s.rfind('}')) {
        let prefix = s.get(..open).unwrap_or("").trim_end_matches('=').trim();
        let var = prefix.split(':').next().unwrap_or("").trim().replace("let mut ", "").replace("let ", "");
        let dict_body = s.get(open + 1..close).unwrap_or("");
        let mut res = format!("let mut {} = std::collections::HashMap::new();\n", var);
        for pair in dict_body.split(',') {
            if let Some(pos) = pair.find(':') {
                res.push_str(&format!(
                    "{}.insert({}, {});\n",
                    var,
                    pair.get(..pos).unwrap_or("").trim(),
                    pair.get(pos + 1..).unwrap_or("").trim()
                ));
            }
        }
        res.trim_end().to_string()
    } else {
        s.to_string()
    }
}

fn handle_repeat(s: &str) -> String {
    let body = s["repeat ".len()..].trim();
    if body.is_empty() || body == "{" {
        return "loop{".to_string();
    }
    let (expr_part, brace) = match body.find('{') {
        Some(p) => (body[..p].trim(), " {"),
        None => (body, ""),
    };
    let expr = expr_part.split(" as ").next().unwrap_or("0").trim();
    let var = expr_part.split(" as ").nth(1).unwrap_or("i").trim();
    format!("for {} in 0..{} {}", var, expr, brace)
}

fn format_standard_call(f: &str, args: &str) -> String {
    if f.contains("arg1") { return f.replace("arg1", args); }
    if let Some(p) = f.rfind(')') {
        let head = f.get(..p).unwrap_or("").trim();
        let sep = if head.ends_with('(') { "" } else { ", " };
        format!("{}{}{})", head, sep, args)
    } else {
        format!("{}({})", f, args)
    }
}

// --- Поиск стрелки с безопасным взятием среза ---
fn parse_step(input: &str) -> (String, String) {
    let arrows = ["->|", "->^", "->()", "!->", "&->", "mv->", "+>", "->"];
    for a in arrows {
        if let Some(p) = input.find(a) {
            if let Some(part) = input.get(..p) {
                return (part.trim().to_string(), a.to_string());
            }
        }
    }
    (input.to_string(), "NONE".to_string())
}