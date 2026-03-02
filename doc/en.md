# 🏹 ArrowRust (ARS) Documentation

ArrowRust treats code as a directed flow of data. All preprocessor operators are expanded into standard Rust constructs during the transpilation phase.

## Core Operators

### 1. Transporter (`->`)
Passes ownership of data to the next node.
- `data -> func` => `func(data)`
- `data -> var;` => `let var = data;`

### 2. Sensor (`<-&`)
Passes an immutable reference. The original data remains owned by the caller.
- `data <-& func` => `func(&data)`

### 3. Assembler (`+>`)
Unpacks an array or tuple into function arguments.
- `[a, b] +> func` => `func(a, b)`

### 4. Replicator (`<n>`)
Creates a loop with `n` iterations.
- Syntax: `data <n> { ... }`
- Internal variables:
  - `it`: the input data.
  - `idx`: current iteration index (0..n).

### 5. Utilizer (`-x>`)
Explicitly terminates an object's lifetime.
- `-my_var>` => `drop(my_var);`

## Command Line Interface

- `-f <path>`: Process a specific file.
- `-o <path>`: Specify output `.rs` file name.
- `-fwf <path>`: Folder-Wide-Factory mode (processes all `.ars` files in a directory).
- `-b`: Invoke `rustc` to build a binary.
- `-t`: Transpilation mode (code generation only, default).
