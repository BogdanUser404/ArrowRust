// std/Base.rs
// Базовая стандартная библиотека ArrowRust
// Содержит основные типы: Value, OpStatus, ToValue, Dict, SafeString, SafeVector
// Битовые операции вынесены в отдельный модуль bit_ops.rs
//#ARROW_IGNORE
use std::fmt;
use std::ops::{Index, IndexMut, Add, AddAssign};
use std::iter::FromIterator;
use std::str::FromStr;

// ========================== OpStatus ==========================

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OpStatus {
    Success,
    IndexError,
    InvalidState,
    TypeMismatch,
}

impl fmt::Display for OpStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpStatus::Success => write!(f, "Success"),
            OpStatus::IndexError => write!(f, "Index out of bounds"),
            OpStatus::InvalidState => write!(f, "Invalid state"),
            OpStatus::TypeMismatch => write!(f, "Type mismatch"),
        }
    }
}

// ========================== Value ==========================

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Bool(bool),
    U8(u8), U16(u16), U32(u32), U64(u64), U128(u128), Usize(usize),
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128), Isize(isize),
    F32(f32), F64(f64),
    Char(char),
    String(String),
    Bytes(Vec<u8>),
}

// ========================== ToValue ==========================

pub trait ToValue {
    fn to_value(self) -> Value;
}

impl ToValue for bool { fn to_value(self) -> Value { Value::Bool(self) } }
impl ToValue for u8 { fn to_value(self) -> Value { Value::U8(self) } }
impl ToValue for u16 { fn to_value(self) -> Value { Value::U16(self) } }
impl ToValue for u32 { fn to_value(self) -> Value { Value::U32(self) } }
impl ToValue for u64 { fn to_value(self) -> Value { Value::U64(self) } }
impl ToValue for u128 { fn to_value(self) -> Value { Value::U128(self) } }
impl ToValue for usize { fn to_value(self) -> Value { Value::Usize(self) } }
impl ToValue for i8 { fn to_value(self) -> Value { Value::I8(self) } }
impl ToValue for i16 { fn to_value(self) -> Value { Value::I16(self) } }
impl ToValue for i32 { fn to_value(self) -> Value { Value::I32(self) } }
impl ToValue for i64 { fn to_value(self) -> Value { Value::I64(self) } }
impl ToValue for i128 { fn to_value(self) -> Value { Value::I128(self) } }
impl ToValue for isize { fn to_value(self) -> Value { Value::Isize(self) } }
impl ToValue for f32 { fn to_value(self) -> Value { Value::F32(self) } }
impl ToValue for f64 { fn to_value(self) -> Value { Value::F64(self) } }
impl ToValue for char { fn to_value(self) -> Value { Value::Char(self) } }
impl ToValue for String { fn to_value(self) -> Value { Value::String(self) } }
impl ToValue for &str { fn to_value(self) -> Value { Value::String(self.to_string()) } }
impl ToValue for Vec<u8> { fn to_value(self) -> Value { Value::Bytes(self) } }

pub fn to_value<T: ToValue>(v: T) -> Value {
    v.to_value()
}

// ========================== Dict ==========================

#[derive(Debug, Clone)]
pub struct Dict {
    keys: Vec<Value>,
    values: Vec<Value>,
}

impl Dict {
    pub fn new() -> Self {
        Dict { keys: Vec::new(), values: Vec::new() }
    }

    pub fn len(&self) -> usize { self.keys.len() }

    pub fn is_empty(&self) -> bool { self.keys.is_empty() }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.values.clear();
    }

    pub fn add(&mut self, key: Value, value: Value) {
        self.keys.push(key);
        self.values.push(value);
    }

    pub fn get(&self, key: &Value) -> Result<Value, OpStatus> {
        for (i, k) in self.keys.iter().enumerate() {
            if k == key {
                return Ok(self.values[i].clone());
            }
        }
        Err(OpStatus::IndexError)
    }

    pub fn set(&mut self, key: &Value, value: Value) -> Result<(), OpStatus> {
        for (i, k) in self.keys.iter().enumerate() {
            if k == key {
                self.values[i] = value;
                return Ok(());
            }
        }
        Err(OpStatus::IndexError)
    }

    pub fn contains_key(&self, key: &Value) -> bool {
        self.keys.iter().any(|k| k == key)
    }

    pub fn remove(&mut self, key: &Value) -> Result<Value, OpStatus> {
        for i in (0..self.keys.len()).rev() {
            if &self.keys[i] == key {
                let value = self.values.remove(i);
                self.keys.remove(i);
                return Ok(value);
            }
        }
        Err(OpStatus::IndexError)
    }

    pub fn keys(&self) -> impl Iterator<Item = &Value> {
        self.keys.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &Value> {
        self.values.iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.keys.iter().zip(self.values.iter())
    }

    pub fn replace_contents(&mut self, new_keys: Vec<Value>, new_values: Vec<Value>) -> Result<(), &'static str> {
        if new_keys.len() != new_values.len() {
            return Err("keys and values length mismatch");
        }
        self.keys = new_keys;
        self.values = new_values;
        Ok(())
    }

    pub unsafe fn set_keys(&mut self, new_keys: Vec<Value>) {
        self.keys = new_keys;
    }

    pub unsafe fn set_values(&mut self, new_values: Vec<Value>) {
        self.values = new_values;
    }
}

// ========================== SafeString ==========================

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SafeString {
    content: Vec<char>,
}

impl SafeString {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }

    pub fn get_symbol(&self, index: u64) -> Result<char, OpStatus> {
        self.content.get(index as usize).copied().ok_or(OpStatus::IndexError)
    }

    pub fn set_symbol(&mut self, index: u64, symbol: char) -> Result<(), OpStatus> {
        let idx = index as usize;
        if idx < self.content.len() {
            self.content[idx] = symbol;
            Ok(())
        } else {
            Err(OpStatus::IndexError)
        }
    }

    pub fn push_str(&mut self, input: &str) {
        self.content.extend(input.chars());
    }

    pub fn set_str(&mut self, input: &str) {
        self.content = input.chars().collect();
    }

    pub fn from_str(s: &str) -> Self {
        Self::from(s)
    }
}

impl fmt::Display for SafeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s: String = self.content.iter().collect();
        write!(f, "{}", s)
    }
}

impl Index<usize> for SafeString {
    type Output = char;
    fn index(&self, index: usize) -> &Self::Output {
        &self.content[index]
    }
}

impl From<&str> for SafeString {
    fn from(s: &str) -> Self {
        Self { content: s.chars().collect() }
    }
}

impl FromIterator<char> for SafeString {
    fn from_iter<I: IntoIterator<Item = char>>(iter: I) -> Self {
        Self { content: iter.into_iter().collect() }
    }
}

impl FromStr for SafeString {
    type Err = std::convert::Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl Add<SafeString> for SafeString {
    type Output = Self;
    fn add(mut self, rhs: SafeString) -> Self {
        self.content.extend(rhs.content);
        self
    }
}

impl AddAssign<&str> for SafeString {
    fn add_assign(&mut self, rhs: &str) {
        self.push_str(rhs);
    }
}

impl AsRef<[char]> for SafeString {
    fn as_ref(&self) -> &[char] {
        &self.content
    }
}

// ========================== SafeVector ==========================

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SafeVector {
    content: Vec<Value>,
}

impl SafeVector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn clear(&mut self) {
        self.content.clear();
    }

    pub fn get_value(&self, index: u64) -> Result<Value, OpStatus> {
        self.content.get(index as usize).cloned().ok_or(OpStatus::IndexError)
    }

    pub fn set_value(&mut self, index: u64, val: Value) -> Result<(), OpStatus> {
        let idx = index as usize;
        if idx < self.content.len() {
            self.content[idx] = val;
            Ok(())
        } else {
            Err(OpStatus::IndexError)
        }
    }

    pub fn push(&mut self, val: Value) {
        self.content.push(val);
    }

    pub fn pop(&mut self) -> Option<Value> {
        self.content.pop()
    }
}

impl fmt::Display for SafeVector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, val) in self.content.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{:?}", val)?;
        }
        write!(f, "]")
    }
}

impl Index<usize> for SafeVector {
    type Output = Value;
    fn index(&self, index: usize) -> &Self::Output {
        &self.content[index]
    }
}

impl IndexMut<usize> for SafeVector {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.content[index]
    }
}

impl From<Vec<Value>> for SafeVector {
    fn from(v: Vec<Value>) -> Self {
        Self { content: v }
    }
}

impl FromIterator<Value> for SafeVector {
    fn from_iter<I: IntoIterator<Item = Value>>(iter: I) -> Self {
        Self { content: iter.into_iter().collect() }
    }
}

impl IntoIterator for SafeVector {
    type Item = Value;
    type IntoIter = std::vec::IntoIter<Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.into_iter()
    }
}

impl<'a> IntoIterator for &'a SafeVector {
    type Item = &'a Value;
    type IntoIter = std::slice::Iter<'a, Value>;

    fn into_iter(self) -> Self::IntoIter {
        self.content.iter()
    }
}

impl Extend<Value> for SafeVector {
    fn extend<I: IntoIterator<Item = Value>>(&mut self, iter: I) {
        self.content.extend(iter);
    }
}

impl Add<SafeVector> for SafeVector {
    type Output = Self;
    fn add(mut self, rhs: SafeVector) -> Self {
        self.content.extend(rhs.content);
        self
    }
}

impl AddAssign<Value> for SafeVector {
    fn add_assign(&mut self, rhs: Value) {
        self.push(rhs);
    }
}

impl AsRef<[Value]> for SafeVector {
    fn as_ref(&self) -> &[Value] {
        &self.content
    }
}
//#ARROW_NO_IGNORE