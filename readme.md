# 🏹 ArrowRust (ARS)

**A high-performance pipeline preprocessor for the Rust programming language.**

ArrowRust (ARS) is a system-level CLI tool that introduces declarative data-flow syntax to Rust. It transforms visual "pipeline" blocks and engineering-style operators into optimized, native Rust source code.

## 🚀 Quick Start

1. Define your logic in a `.ars` file:
```rust
fn main() {
    "hello" -> String::from -> to_uppercase -> result;
    result -> println!("{}", arg1);
}
```
2. Install arrowc

```bash
git clone https://github.com/BogdanUser404/ArrowRust
cd ArrowRust
python install.py #Only on linux/*nix
```

