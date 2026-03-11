// macro_processor.rs
use std::collections::{HashMap, HashSet};
use std::fs;

// ANSI color codes for terminal output
const YELLOW: &str = "\x1b[33m";
const BOLD_RED: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";

pub fn process_dialect(input: String, lib_root: &str) -> String {
	// Collect macros from the whole file first (including includes will be handled later)
	let mut macros: HashMap<String, String> = collect_definitions(&input);
	// Built-in macro "var" (cannot be removed)
	macros.insert("var".to_string(), "let mut".to_string());

	let mut output: String = String::with_capacity(input.len());
	let mut visited: HashSet<String> = HashSet::new();
	let mut enabled: bool = false;
	let mut strict_tabs: bool = false; // Inquisition mode

	for (line_idx, line) in input.lines().enumerate() {
		let line_num = line_idx + 1;
		let trimmed = line.trim_start();

		// 1. Handle control directives
		if trimmed.starts_with("//#") {
			if trimmed.starts_with("//#NO_SPACE") {
				strict_tabs = true;
				output.push_str(line);
			} else if trimmed.starts_with("//#ENABLE_MACRO") {
				enabled = true;
				output.push_str(line);
			} else if trimmed.starts_with("//#DISABLE_MACRO") {
				enabled = false;
				output.push_str(line);
			} else if trimmed.starts_with("//#INCLUDE") {
				let path_part = trimmed
					.chars()
					.skip(10)
					.collect::<String>()
					.trim()
					.to_string();
				handle_include(
					&path_part,
					lib_root,
					&mut visited,
					enabled,
					&mut macros, // pass macros as mutable to collect new definitions
					&mut output,
				);
				output.push('\n');
				continue;
			} else if trimmed.starts_with("//#DEFINE") {
				// definitions are already collected at the beginning, but we also need to handle
				// definitions inside includes. They will be processed via handle_include.
				// Here we just skip the line.
				continue;
			} else if trimmed.starts_with("//#CLEAR_MACROES") {
				// Clear all macros except the built-in "var"
				macros.retain(|k, _| k == "var");
				// Do not write this directive to output
				continue;
			} else {
				output.push_str(line);
			}
		} else {
			// 2. Space/tab check in NO_SPACE mode
			if strict_tabs && !line.is_empty() {
				if line.starts_with(' ') {
					eprintln!(
						"{}[Style Error] Line {}: Detected spaces instead of tabs!{}",
						YELLOW, line_num, RESET
					);
					panic!(
						"\n{}STYLE PANIC:{} \n\t\
						Line {}: Indentation must use TABS ONLY.\n\t\
						Spacebars are for Windows users. \n\t\
						Real Arch-warriors use TABS (8 chars width recommended).",
						BOLD_RED, RESET, line_num
					);
				}
			}

			// 3. Apply macros if enabled
			if enabled {
				apply_macros(line, &macros, &mut output);
			} else {
				output.push_str(line);
			}
		}
		output.push('\n');
	}
	output
}

/// Collect macro definitions from the given text.
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

/// Process an include directive.
fn handle_include(
	path: &str,
	lib_root: &str,
	visited: &mut HashSet<String>,
	enabled: bool,
	macros: &mut HashMap<String, String>,
	out: &mut String,
) {
	// Build full path
	let full_path = if path.starts_with('/') {
		path.to_string()
	} else {
		let root = if lib_root.ends_with('/') || lib_root.is_empty() {
			lib_root.to_string()
		} else {
			format!("{}/", lib_root)
		};
		format!("{}{}", root, path)
	};

	// Prevent circular includes
	if !visited.insert(full_path.clone()) {
		return;
	}

	// Read the included file
	if let Ok(content) = fs::read_to_string(&full_path) {
		// First, collect any macro definitions inside the included file
		let new_macros = collect_definitions(&content);
		for (k, v) in new_macros {
			macros.insert(k, v);
		}

		// Now process the content line by line
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
	// Do not remove from visited to avoid re‑inclusion.
}

/// Apply macros to a line of code.
fn apply_macros(line: &str, macros: &HashMap<String, String>, out: &mut String) {
	let mut current_word = String::new();

	for c in line.chars() {
		if c.is_alphanumeric() || c == '_' || c == ')' || c == ':' || c == '=' || c == '>' || c == '-' {
			current_word.push(c);
		} else {
			if !current_word.is_empty() {
				let replacement = macros
					.get(&current_word)
					.cloned()
					.unwrap_or(current_word.clone());
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