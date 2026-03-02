// Copyright (c) Bogdan Yachmenev (2026)
// License: MIT

use std::{env, fs, process::Command, path::Path};
mod parser;

struct Config {
    input_path: String,
    output_path: Option<String>,
    is_folder: bool,
    build_binary: bool,
    text_only: bool,
}

fn process_single_file(input_path: &Path, output_path: Option<String>, build: bool) {
    let stem = input_path.file_stem().unwrap().to_str().unwrap();
    let rs_path = output_path.unwrap_or_else(|| format!("{}.rs", stem));
    
    println!("[ArrowRust] Transpiling: {:?} -> {}", input_path, rs_path);
    
    let input_content = fs::read_to_string(input_path).expect("Read Error");
    let mut final_code = String::new();

    // Standard factory pipeline processing
    for line in input_content.lines() {
        let processed = parser::compile_arrow_line(line);
        final_code.push_str(&processed);
    }

    fs::write(&rs_path, &final_code).expect("Write Error");

    if build {
        println!("[ArrowRust] Compiling binary for {}...", rs_path);
        let status = Command::new("rustc")
            .arg(&rs_path)
            .status()
            .expect("rustc execution error");
        
        if status.success() {
            println!("[ArrowRust] Build Successful!");
        }
    }
}

fn process_folder(dir_path: &Path) {
    println!("[ArrowRust] Processing folder: {:?}", dir_path);
    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |s| s == "ars") {
                process_single_file(&path, None, false);
            }
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("\n[ArrowRust] CLI Help:");
        println!("  -f <path>     Build specific file");
        println!("  -o <path>     Set output path");
        println!("  -fwf <path>   Build all .ars files in folder");
        println!("  -b            Compile into binary (using rustc)");
        println!("  -t            Generate .rs code only (default)\n");
        return;
    }

    let mut config = Config {
        input_path: String::new(),
        output_path: None,
        is_folder: false,
        build_binary: false,
        text_only: true,
    };

    // Simple arg parser
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-f" => {
                config.input_path = args[i+1].clone();
                config.is_folder = false;
                i += 1;
            },
            "-o" => {
                config.output_path = Some(args[i+1].clone());
                i += 1;
            },
            "-fwf" => {
                config.input_path = args[i+1].clone();
                config.is_folder = true;
                i += 1;
            },
            "-b" => {
                config.build_binary = true;
                config.text_only = false;
            },
            "-t" => {
                config.build_binary = false;
                config.text_only = true;
            },
            _ => {}
        }
        i += 1;
    }

    if config.is_folder {
        process_folder(Path::new(&config.input_path));
    } else if !config.input_path.is_empty() {
        process_single_file(
            Path::new(&config.input_path), 
            config.output_path, 
            config.build_binary
        );
    }
}
