use std::{env, fs, process::Command, path::Path};
pub mod transpiler;
pub mod macro_processor;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { return; }

    let mut input_path = String::new();
    let mut output_path = None;
    let mut is_folder = false;
    let mut build = false;
    let mut lib_root = ".".to_string(); // значение по умолчанию

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-f" => {
                input_path = args[i+1].clone();
                i += 1;
            }
            "-fwf" => {
                input_path = args[i+1].clone();
                is_folder = true;
                i += 1;
            }
            "-o" => {
                output_path = Some(args[i+1].clone());
                i += 1;
            }
            "-b" => build = true,
            "-arrowpath" => {
                if i + 1 < args.len() {
                    lib_root = args[i+1].clone();
                    i += 1;
                } else {
                    eprintln!("Warning: -arrowpath requires a path argument");
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Если путь не задан через аргумент, пробуем взять из переменной окружения
    if lib_root == "." {
        if let Ok(env_path) = env::var("ARROWPATH") {
            lib_root = env_path;
        }
    }

    if is_folder {
        process_dir(Path::new(&input_path), build, &lib_root);
    } else {
        process_file(Path::new(&input_path), output_path, build, &lib_root);
    }
}

fn process_file(path: &Path, out: Option<String>, build: bool, lib_root: &str) {
    let abs_path = fs::canonicalize(path).unwrap();
    let content = fs::read_to_string(&abs_path).unwrap();
    println!("Started Preprocessor");
    let expanded = macro_processor::process_dialect(content, lib_root);
    println!("End preprocessor,start traslator");
    let mut final_code = String::new();
    for line in expanded.lines() {
        final_code.push_str(&transpiler::transpile_line(line));
    }
    println!("Translator end!");

    let out_path = out.unwrap_or_else(|| format!("{}.rs", path.file_stem().unwrap().to_str().unwrap()));
    fs::write(&out_path, final_code).unwrap();

    if build {
        Command::new("rustc").arg(&out_path).status().unwrap();
        println!("Compiling Sucesfull");
    }
    println!("End programm");
}

fn process_dir(dir: &Path, build: bool, lib_root: &str) {
    if let Ok(entries) = fs::read_dir(dir) {
        for e in entries.flatten() {
            let p = e.path();
            if p.extension().map_or(false, |s| s == "ars") {
                process_file(&p, None, build, lib_root);
            }
        }
    }
}