// std/bit_ops.rs
// Модуль для побитовых операций над Value
// Требует наличия Base.rs (Value, OpStatus)

use crate::base::{Value, OpStatus};

// Установка бита в значении
pub fn set_bit(mut source: Value, bit_idx: u32, state: u8) -> Result<Value, OpStatus> {
    if state > 1 {
        return Err(OpStatus::InvalidState);
    }

    match &mut source {
        Value::Bool(ref mut v) => apply_raw(v, bit_idx, state),
        Value::U8(ref mut v)   => apply_raw(v, bit_idx, state),
        Value::U16(ref mut v)  => apply_raw(v, bit_idx, state),
        Value::U32(ref mut v)  => apply_raw(v, bit_idx, state),
        Value::U64(ref mut v)  => apply_raw(v, bit_idx, state),
        Value::U128(ref mut v) => apply_raw(v, bit_idx, state),
        Value::Usize(ref mut v) => apply_raw(v, bit_idx, state),
        Value::I8(ref mut v)   => apply_raw(v, bit_idx, state),
        Value::I16(ref mut v)  => apply_raw(v, bit_idx, state),
        Value::I32(ref mut v)  => apply_raw(v, bit_idx, state),
        Value::I64(ref mut v)  => apply_raw(v, bit_idx, state),
        Value::I128(ref mut v) => apply_raw(v, bit_idx, state),
        Value::Isize(ref mut v) => apply_raw(v, bit_idx, state),
        Value::F32(ref mut v)  => apply_raw(v, bit_idx, state),
        Value::F64(ref mut v)  => apply_raw(v, bit_idx, state),
        Value::Char(ref mut v) => apply_raw(v, bit_idx, state),
        Value::String(_) | Value::Bytes(_) => Err(OpStatus::TypeMismatch),
    }?;
    Ok(source)
}

unsafe fn apply_raw<T>(obj: *mut T, bit_idx: u32, state: u8) -> Result<(), OpStatus> {
    let size = std::mem::size_of::<T>();
    let slice = std::slice::from_raw_parts_mut(obj as *mut u8, size);
    modify_slice(slice, bit_idx, state)
}

fn modify_slice(bytes: &mut [u8], bit_idx: u32, state: u8) -> Result<(), OpStatus> {
    let byte_idx = (bit_idx / 8) as usize;
    let bit_in_byte = bit_idx % 8;

    if byte_idx < bytes.len() {
        let mask = 1 << bit_in_byte;
        if state == 1 {
            bytes[byte_idx] |= mask;
        } else {
            bytes[byte_idx] &= !mask;
        }
        Ok(())
    } else {
        Err(OpStatus::IndexError)
    }
}

// Получение бита из значения
pub fn get_bit(source: &Value, bit_idx: u32) -> Result<u8, OpStatus> {
    match source {
        Value::I128(v) => get_bit_from_number(v, bit_idx),
        Value::I64(v) => get_bit_from_number(v, bit_idx),
        Value::I32(v) => get_bit_from_number(v, bit_idx),
        Value::I16(v) => get_bit_from_number(v, bit_idx),
        Value::I8(v) => get_bit_from_number(v, bit_idx),
        Value::U128(v) => get_bit_from_number(v, bit_idx),
        Value::U64(v) => get_bit_from_number(v, bit_idx),
        Value::U32(v) => get_bit_from_number(v, bit_idx),
        Value::U16(v) => get_bit_from_number(v, bit_idx),
        Value::U8(v) => get_bit_from_number(v, bit_idx),
        Value::F64(v) => get_bit_from_number(v, bit_idx),
        Value::F32(v) => get_bit_from_number(v, bit_idx),
        Value::Bool(v) => get_bit_from_number(v, bit_idx),
        Value::Isize(v) => get_bit_from_number(v, bit_idx),
        Value::Usize(v) => get_bit_from_number(v, bit_idx),
        Value::Char(v) => get_bit_from_number(v, bit_idx),
        Value::String(_) | Value::Bytes(_) => Err(OpStatus::TypeMismatch),
    }
}

fn get_bit_from_number<T: Copy>(val: &T, bit_idx: u32) -> Result<u8, OpStatus> {
    let size = std::mem::size_of::<T>();
    let ptr = val as *const T as *const u8;
    let byte_idx = (bit_idx / 8) as usize;
    let bit_in_byte = bit_idx % 8;
    if byte_idx < size {
        unsafe {
            let byte = *ptr.add(byte_idx);
            Ok((byte >> bit_in_byte) & 1)
        }
    } else {
        Err(OpStatus::IndexError)
    }
}