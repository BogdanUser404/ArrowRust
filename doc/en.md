### Documentation for the ArrowRust Rust Dialect

## Arrows
ArrowRust uses arrows as the primary language operators. Schematically, they can be represented as:
`arguments -> function() -> return`, but that's not all. The transpiler converts this into valid Rust code that runs just as well as native Rust.

| Arrow  | Description                                      | Example                                              |
|--------|--------------------------------------------------|------------------------------------------------------|
| `+>`   | Passes multiple arguments to a function          | `[arg1, arg2] +> function()`                         |
| `->`   | Passes a value to a function or to a variable    | `function() -> var1 -> function2()`                  |
| `-variable->` | Deinitializes a variable (drop)               | `-variable->`                                         |
| `mv->` | Works like `->`, but also drops the value        | `variable mv-> function()`                            |
| `!->`  | Analogous to `.expect("text")` as an arrow       | `function() !-> "Error message"`                      |
| `->\|` | Analogous to `break` in Rust                     | `while true { println!("text"); ->\| }`               |
| `->^`  | Analogous to `continue` in Rust                  | `while true { println!("Hello!"); if true { ->^ } }`  |
| `->()` | Returns an empty tuple from a function           | `->()`                                                 |
| `&->`  | Works like `->` but passes a reference           | `variable &-> function()`                              |

## New Keywords
- `pass` – does nothing, literally becomes a comment.
- `import std;` – imports basic features from Rust's standard library.
- `tuple name (tuple)` – a wrapper over tuple structs.
- `customtype name type` – a wrapper over `type` aliases.

## Loops
ArrowRust introduces two new loops: `repeat` and `enumeration`.

### Repeat
```rust
repeat count as where_to_put_index { // where_to_put_index is optional, defaults to i
    // code
}
```
Infinite loop variant:

```rust
repeat {
    // code
}
```
Enumeration
A built-in loop for iterating over types that support the Index trait and have a .len() method.

```rust
enumeration from variable1 to variable2 {
    // code
}
```
### Standard Library (STD)
ArrowRust provides a set of ready-to-use modules for common tasks:

std/Base.rs – for working with safe types and the dynamic Value type.

std/Random.rs – for generating random characters and numbers.

std/Bits.rs – for bitwise operations (setting/reading a specific bit).

std/Terminal.rs – for color and terminal manipulation.

### std/Base
New types:

 - SafeString – stores strings as a vector of char instead of a byte sequence.

 - SafeVector – a dynamically growing array without strict typing.

 - Value – a type that can hold other types.

 - SafeString and SafeVector provide the usual functionality of strings and vectors, respectively.

 - Value can contain:

 - integers: i8–i128, u8–u128

 - floats: f32, f64

 - Vec<u8>

 - String

 - char

 - bool

 - None

Value::None is a character (char) used for error handling and zeroing out data. It is recommended to use the following five basic values:

- '\0' – zeroing bytes and returning “nothing”.

- 'r' – read error.

- 'b' – corrupt data.

- 's' – success but nothing to return.

Other values are free for custom codes.