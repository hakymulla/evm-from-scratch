use std::any::Any;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use primitive_types::U256;
use serde_json::Value;
use crate::types::ExpectData;
use std::string::String;

lazy_static! {
    pub static ref MEMORY: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![]));
    pub static ref MSIZE: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    pub static ref STORAGE: Arc<Mutex<HashMap<U256, U256>>> = Arc::new(Mutex::new(HashMap::new()));
    pub static ref RETURNDATA: Arc<Mutex<ExpectData>> = Arc::new(Mutex::new(ExpectData { stack: vec![], logs: vec![], ret: String::new() }));

}

pub fn jump(dst: usize, pc: usize, mut code: &[u8]) -> Option<&[u8]> {
    if dst > (code.len() + pc) {
        return None;
    }

    if code[dst - pc - 1] != 91 {
        return None;
    }

    println!("code[dst - pc - 1]: {:?}", code[dst - pc - 2]);
    if code[dst - pc - 2] >= 96 && code[dst - pc - 2] <= 127 {
        return None;
    }

    let (_, new_code) = code.split_at(dst - pc);
    code = new_code;

    Some(code)
}

pub fn get_n_bytes(value: &[u8], byte: usize) -> (&[u8], &[u8]) {
    (&value[..byte], &value[byte..])
}

pub fn to_signed(x: U256) -> (bool, U256) {
    let sign_bit = x.bit(255);

    if !sign_bit {
        // positive, magnitude is just x
        (false, x)
    } else {
        // negative, two's complement to get magnitude
        let magnitude = (!x) + U256::one();
        (true, magnitude)
    }
}

pub fn to_unsigned(is_negative: bool, magnitude: U256) -> U256 {
    if !is_negative {
        // positive, bits are just the magnitude
        magnitude
    } else {
        // negative, reverse two's complement
        (!magnitude) + U256::one()
    }
}

pub fn reset_memory_var() {
    let mut value = MEMORY.lock().unwrap();
    *value = vec![]
}

pub fn reset_msize_var() {
    let mut value = MSIZE.lock().unwrap();
    *value = 0
}


pub fn reset_return_data() {
    let mut return_data = RETURNDATA.lock().unwrap();
    return_data.logs.clear();
    return_data.ret.clear();
    return_data.stack.clear();
}


pub fn get_blockchain_data(tx: &Option<Value>, value: &str) -> Result<U256, Value> {
    let ret = serde_json::Value::String("failed".to_string());
    let value_obj = tx.clone().unwrap_or_else(|| ret.clone());
    let value_str = value_obj.get(value).unwrap_or_else(|| &ret).clone();
    let value_str = value_str.as_str().unwrap();
    let res = U256::from_str(value_str).unwrap_or_default();
    Ok(res)
}

pub fn get_state_data(state: &Value, address: &str, value: &str) -> U256 {
    let value_obj = state.clone();
    let address_obj = value_obj.get(address).unwrap();
    let balance_obj = address_obj.get(value).unwrap();
    let balance = balance_obj.as_str().unwrap();
    U256::from_str(balance).unwrap()
}

pub fn get_bin_state_data<'a>(state: &'a Value, address: &'a str) -> &'a str {
    let value_obj = state;
    let address_obj = value_obj.get(address).unwrap();
    let code_obj = address_obj.get("code").unwrap_or_default();
    let bin_obj = code_obj.get("bin").unwrap_or_default().as_str().unwrap_or_default();
    bin_obj
}

pub fn get_one_mut_val(v: &mut Vec<U256>) -> U256 {
    let a = v.pop().unwrap_or(U256::max_value());
    a
}

pub fn get_mut_val(v: &mut Vec<U256>) -> (U256, U256) {
    let a = v.pop().unwrap();
    let b = v.pop().unwrap();
    (a, b)
}

pub fn get_three_mut_val(v: &mut Vec<U256>) -> (U256, U256, U256) {
    let a = v.pop().unwrap();
    let b = v.pop().unwrap();
    let c = v.pop().unwrap();
    (a, b, c)
}

pub fn get_four_mut_val(v: &mut Vec<U256>) -> (U256, U256, U256, U256) {
    let a = v.pop().unwrap();
    let b = v.pop().unwrap();
    let c = v.pop().unwrap();
    let d = v.pop().unwrap_or(U256::max_value());
    (a, b, c, d)
}


pub fn pop_v(v: &mut Vec<U256>, n: usize) -> Vec<U256> {
    (0..n).map(|_| v.pop().unwrap()).collect()
}


pub fn is_static_violation(code: &[u8]) -> bool {
    let mut i = 0;
    while i < code.len() {
        let op = code[i];
        match op {
            0xf5 |          // CREATE2
            0xf0 |          // CREATE
            0x55 |          // SSTORE
            0xa0 |          // LOG0
            0xa1 |          // LOG1
            0xa2 |          // LOG2
            0xa3 |          // LOG3
            0xa4 |          // LOG4
            0xff |          // SELFDESTRUCT
            0x5d            // TSTORE
            => {       
                return true;
            }
            0x60..=0x7f => {
                let push_size = (op - 0x60 + 1) as usize;
                i += push_size;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}