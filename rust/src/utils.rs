use std::str::FromStr;
use std::sync::{Arc, Mutex};

use lazy_static::lazy_static;
use primitive_types::U256;
use serde_json::Value;

lazy_static! {
    pub static ref MEMORY: Arc<Mutex<Vec<u8>>> = Arc::new(Mutex::new(vec![]));
    pub static ref MSIZE: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
}

pub fn jump(dst: usize, mut pc: usize, mut code: &[u8]) -> Option<&[u8]> {
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

    pc += dst;
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
    *value = vec![] // Reset to a new value
}

pub fn reset_msize_var() {
    let mut value = MSIZE.lock().unwrap();
    *value = 0 // Reset to a new value
}

pub fn get_blockchain_data(tx: &Option<Value>, value: &str) -> U256 {
    let value_obj = tx.clone().unwrap();
    let value_str = value_obj.get(value).unwrap();
    let value_str = value_str.as_str().unwrap();
    U256::from_str(value_str).unwrap()
}

pub fn get_state_data(state: &Value, address: &str, value: &str) -> U256 {
    let value_obj = state.clone();
    let address_obj = value_obj.get(address).unwrap();
    let balance_obj = address_obj.get(value).unwrap();
    let balance = balance_obj.as_str().unwrap();
    U256::from_str(balance).unwrap()
}

pub fn get_one_mut_val(v: &mut Vec<U256>) -> U256 {
    let a = v.pop().unwrap();
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
