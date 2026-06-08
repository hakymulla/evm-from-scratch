mod utils;

use std::{ops::Div, str::FromStr};

use keccak_hash::{self, keccak};
use primitive_types::U256;
use serde_json::Value;

use utils::{
    get_blockchain_data, get_mut_val, get_n_bytes, get_one_mut_val, get_state_data,
    get_three_mut_val, jump, reset_memory_var, reset_msize_var, to_signed, to_unsigned, MEMORY,
    MSIZE,
};

#[derive(Debug)]
pub struct EvmResult {
    pub stack: Vec<U256>,
    pub success: bool,
}

#[derive(Debug)]
enum EvmError {
    OpCodeError(String),
}

#[derive(Debug)]
enum Opcodes {
    STOP,
    ADD,
    MUL,
    SUB,
    DIV,
    SDIV,
    MOD,
    SMOD,
    ADDMOD,
    MULMOD,
    EXP,
    SIGNEXTEND,
    LT,
    GT,
    SLT,
    SGT,
    EQ,
    ISZERO,
    AND,
    OR,
    XOR,
    NOT,
    BYTE,
    SHL,
    SHR,
    SAR,
    SHA3,
    ADDRESS,
    BALANCE,
    ORIGIN,
    CALLER,
    CALLVALUE,
    CALLDATALOAD,
    CALLDATASIZE,
    CALLDATACOPY,
    CODESIZE,
    CODECOPY,
    GASPRICE,
    BLOCKHASH,
    COINBASE,
    TIMESTAMP,
    NUMBER,
    DIFFICULTY,
    GASLIMIT,
    CHAINID,
    BASEFEE,
    JUMP,
    PC,
    PUSH0,
    PUSHX,
    POP,
    MLOAD,
    MSTORE,
    MSTORE8,
    JUMPI,
    MSIZE,
    GAS,
    JUMPDEST,
    DUP,
    SWAP,
    INVALID,
}

impl TryFrom<&u8> for Opcodes {
    type Error = EvmError;

    fn try_from(value: &u8) -> Result<Self, Self::Error> {
        match value {
            0x00 => Ok(Opcodes::STOP),
            0x01 => Ok(Opcodes::ADD),
            0x02 => Ok(Opcodes::MUL),
            0x03 => Ok(Opcodes::SUB),
            0x04 => Ok(Opcodes::DIV),
            0x05 => Ok(Opcodes::SDIV),
            0x06 => Ok(Opcodes::MOD),
            0x07 => Ok(Opcodes::SMOD),
            0x08 => Ok(Opcodes::ADDMOD),
            0x09 => Ok(Opcodes::MULMOD),
            10 => Ok(Opcodes::EXP),
            11 => Ok(Opcodes::SIGNEXTEND),
            16 => Ok(Opcodes::LT),
            17 => Ok(Opcodes::GT),
            18 => Ok(Opcodes::SLT),
            19 => Ok(Opcodes::SGT),
            20 => Ok(Opcodes::EQ),
            21 => Ok(Opcodes::ISZERO),
            22 => Ok(Opcodes::AND),
            23 => Ok(Opcodes::OR),
            24 => Ok(Opcodes::XOR),
            25 => Ok(Opcodes::NOT),
            26 => Ok(Opcodes::BYTE),
            27 => Ok(Opcodes::SHL),
            28 => Ok(Opcodes::SHR),
            29 => Ok(Opcodes::SAR),
            32 => Ok(Opcodes::SHA3),
            48 => Ok(Opcodes::ADDRESS),
            49 => Ok(Opcodes::BALANCE),
            50 => Ok(Opcodes::ORIGIN),
            51 => Ok(Opcodes::CALLER),
            52 => Ok(Opcodes::CALLVALUE),
            53 => Ok(Opcodes::CALLDATALOAD),
            54 => Ok(Opcodes::CALLDATASIZE),
            55 => Ok(Opcodes::CALLDATACOPY),
            56 => Ok(Opcodes::CODESIZE),
            57 => Ok(Opcodes::CODECOPY),
            58 => Ok(Opcodes::GASPRICE),
            64 => Ok(Opcodes::BLOCKHASH),
            65 => Ok(Opcodes::COINBASE),
            66 => Ok(Opcodes::TIMESTAMP),
            67 => Ok(Opcodes::NUMBER),
            68 => Ok(Opcodes::DIFFICULTY),
            69 => Ok(Opcodes::GASLIMIT),
            70 => Ok(Opcodes::CHAINID),
            72 => Ok(Opcodes::BASEFEE),
            86 => Ok(Opcodes::JUMP),
            88 => Ok(Opcodes::PC),
            80 => Ok(Opcodes::POP),
            81 => Ok(Opcodes::MLOAD),
            82 => Ok(Opcodes::MSTORE),
            83 => Ok(Opcodes::MSTORE8),
            87 => Ok(Opcodes::JUMPI),
            89 => Ok(Opcodes::MSIZE),
            90 => Ok(Opcodes::GAS),
            91 => Ok(Opcodes::JUMPDEST),
            95 => Ok(Opcodes::PUSH0),
            96..=127 => Ok(Opcodes::PUSHX),
            128..=143 => Ok(Opcodes::DUP),
            144..=159 => Ok(Opcodes::SWAP),
            254 => Ok(Opcodes::INVALID),

            _ => Err(EvmError::OpCodeError(value.to_string())),
        }
    }
}

fn run(
    code: &[u8],
    mut pc: usize,
    original_code: &[u8],
    mut v: Vec<U256>,
    tx: &Option<Value>,
    block: &Option<Value>,
    state: &Option<Value>,
) -> Option<Vec<U256>> {
    let mut code = code;

    println!("begininnng code: {:?}", code);
    if code.is_empty() {
        return Some(v);
    }

    let (opcode, _) = code.split_first().unwrap();
    println!("opcode: {}", opcode);
    let ops = Opcodes::try_from(opcode).expect("Invalid Opcode");

    match ops {
        Opcodes::STOP => return Some(v),
        Opcodes::ADD => {
            println!("ADD");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            let res = a.overflowing_add(b);
            v.push(res.0);
        }

        Opcodes::MUL => {
            println!("MUL");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            let res = a.overflowing_mul(b);
            v.push(res.0);
        }
        Opcodes::SUB => {
            println!("SUB");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            let res = a.overflowing_sub(b);
            v.push(res.0);
        }
        Opcodes::DIV => {
            println!("DIV");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            println!("a : {:?}", a);
            println!("b : {:?}", b);
            if b == U256::zero() {
                v.push(U256::zero());
            } else {
                let res = a.div(b);
                v.push(res);
            }
        }
        Opcodes::SDIV => {
            println!("SDIV");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            println!("a : {:?}", a);
            println!("b : {:?}", b);
            let (a_neg, a_mag) = to_signed(a);
            let (b_neg, b_mag) = to_signed(b);

            // RULE 3: overflow special case
            // -2²⁵⁵ / -1 → return -2²⁵⁵ (same bits back)
            let min_value = U256::one() << 255; // 0x80...00

            if b == U256::zero() {
                v.push(U256::zero());
            } else if a_neg && a_mag == min_value && b_neg && b_mag == U256::one() {
                // return min_value; // bits of -2²⁵⁵
                v.push(min_value);
            } else {
                // RULE 4: divide magnitudes
                let result_mag = a_mag / b_mag;

                // RULE 5: result is negative only if signs differ
                let result_neg = a_neg != b_neg;

                // RULE 6: convert back to raw bits
                let res = to_unsigned(result_neg, result_mag);
                v.push(res);
            }
        }

        Opcodes::MOD => {
            println!("MOD");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            if b == U256::zero() {
                v.push(U256::zero());
            } else {
                let res = a.div_mod(b);
                v.push(res.1);
            }
        }
        Opcodes::SMOD => {
            println!("SMOD");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            let (a_neg, a_mag) = to_signed(a);
            let (_, b_mag) = to_signed(b);

            if b == U256::zero() {
                v.push(U256::zero());
            } else {
                let result_mag = a_mag % b_mag;
                let result_neg = a_neg;
                let res = to_unsigned(result_neg, result_mag);
                v.push(res);
            }
        }
        Opcodes::ADDMOD => {
            println!("ADDMOD");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b, c) = get_three_mut_val(&mut v);

            let res = (a.saturating_add(b)).div_mod(c);
            println!("res: {:?}", res);

            v.push(res.1);
        }
        Opcodes::MULMOD => {
            println!("MULMOD");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b, c) = get_three_mut_val(&mut v);
            let res = ((a % c) * (b % c)) % c;
            v.push(res);
        }

        Opcodes::EXP => {
            println!("EXP");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            let res = a.pow(b);
            v.push(res);
        }
        Opcodes::SIGNEXTEND => {
            println!("SIGNEXTEND");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (_, b) = get_mut_val(&mut v);

            let mut bytes = [0u8; 32];
            b.to_big_endian(&mut bytes);

            let hex_string = hex::encode(&bytes);
            let byte_count = (b.bits() + 7) / 8;

            // Check a specific bit (e.g. bit 15, the sign bit of a 16-bit int)
            let bit_shift = 7 * byte_count + (byte_count - 1);
            let bit = (b.as_usize() >> bit_shift) & 1;

            if bit == 1 {
                let hex_string = hex_string.replacen("00", "ff", 32 - byte_count);
                v.push(U256::from_str(&hex_string).unwrap());
            } else {
                let b_decoded = hex::encode(&[b.as_usize() as u8]);
                println!("b_decoded: {:?}", b_decoded);
                v.push(U256::from_str(&hex_string).unwrap());
            }
        }
        Opcodes::LT => {
            println!("LT");
            pc += 1;
            // Some(("0x0".to_string(), 0))
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            if a < b {
                v.push(U256::one());
            } else {
                v.push(U256::zero());
            }
        }

        Opcodes::GT => {
            println!("GT");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            if a > b {
                v.push(U256::one());
            } else {
                v.push(U256::zero());
            }
        }

        Opcodes::SLT => {
            println!("SLT");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);

            let (a_neg, _) = to_signed(a);
            let (b_neg, _) = to_signed(b);
            if a_neg > b_neg {
                v.push(U256::one());
            } else {
                v.push(U256::zero());
            }
        }

        Opcodes::SGT => {
            println!("SGT");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            let (b_neg, _) = to_signed(b);
            let (a_neg, _) = to_signed(a);

            if a_neg < b_neg {
                v.push(U256::one());
            } else if a_neg > b_neg {
                v.push(U256::zero());
            } else {
                if a > b {
                    v.push(U256::one());
                } else {
                    v.push(U256::zero());
                }
            }
        }

        Opcodes::EQ => {
            println!("EQ");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);

            if a == b {
                v.push(U256::one());
            } else {
                v.push(U256::zero());
            }
        }
        Opcodes::ISZERO => {
            println!("ISZERO");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let a = get_one_mut_val(&mut v);

            if a == U256::zero() {
                v.push(U256::one());
            } else {
                v.push(U256::zero());
            }
        }
        Opcodes::AND => {
            println!("AND");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            v.push(a & b);
        }
        Opcodes::OR => {
            println!("OR");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            v.push(a | b);
        }
        Opcodes::XOR => {
            println!("XOR");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            v.push(a ^ b);
        }
        Opcodes::NOT => {
            println!("NOT");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let a = get_one_mut_val(&mut v);

            v.push(!a);
        }
        Opcodes::BYTE => {
            println!("BYTE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (i, x) = get_mut_val(&mut v);

            if i > U256::from(32) {
                v.push(U256::zero());
            } else {
                v.push(
                    (x >> (U256::from(248) - i * U256::from(8))) & U256::from_str("0xFF").unwrap(),
                );
            }
        }

        Opcodes::SHL => {
            println!("SHL");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (shift, a) = get_mut_val(&mut v);

            if shift >= U256::from(256) {
                v.push(U256::zero());
            } else if a < U256::max_value() {
                v.push(a << shift);
            }
        }

        Opcodes::SHR => {
            println!("SHR");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (shift, a) = get_mut_val(&mut v);

            if shift >= U256::from(256) {
                v.push(U256::zero());
            } else if a < U256::max_value() {
                v.push(a >> shift);
            }
        }

        Opcodes::SAR => {
            println!("SAR");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (shift, a) = get_mut_val(&mut v);

            let sign_bit = a.bit(255);

            if shift >= U256::from(256) {
                if sign_bit {
                    v.push(U256::max_value());
                } else {
                    v.push(U256::zero());
                };
            } else {
                if !sign_bit {
                    v.push(a >> shift);
                } else {
                    let shifted = a >> shift;

                    let mask = U256::MAX << (U256::from(256) - shift);

                    v.push(shifted | mask);
                }
            }
        }

        Opcodes::SHA3 => {
            println!("SHA3");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (offset, length) = get_mut_val(&mut v);
            let offset = offset.as_usize();
            let memory = MEMORY.lock().unwrap();
            let byte = &memory[offset..offset + length.as_usize()];
            let keccak_hash = keccak(byte);
            v.push(U256::from(keccak_hash.0));
        }

        Opcodes::ADDRESS => {
            println!("ADDRESS");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(tx, "to");

            v.push(value);
        }

        Opcodes::BALANCE => {
            println!("BALANCE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let address = get_one_mut_val(&mut v);
            let address = format!("0x{:x}", address);
            let balance = match state {
                Some(value) => get_state_data(value, &address, "balance"),
                None => U256::zero(),
            };

            v.push(balance);
        }

        Opcodes::ORIGIN => {
            println!("ORIGIN");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(tx, "origin");

            v.push(value);
        }

        Opcodes::CALLER => {
            println!("CALLER");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(tx, "from");

            v.push(value);
        }

        Opcodes::CALLVALUE => {
            println!("CALLVALUE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(tx, "value");

            v.push(value);
        }

        Opcodes::CALLDATALOAD => {
            println!("CALLDATALOAD");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let idx = get_one_mut_val(&mut v).as_usize();
            let value = get_blockchain_data(tx, "data");
            let value_str = format!("000{:x}", value);

            let data = hex::decode(value_str).unwrap();

            let mut buf = [0u8; 32];
            for i in 0..32 {
                let data_idx = idx + i;
                if data_idx < data.len() {
                    buf[i] = data[data_idx];
                }
            }

            let result = format!("{:x}", U256::from_big_endian(&buf));
            let u256_value = U256::from_str(&result).unwrap();
            v.push(u256_value);
        }

        Opcodes::CALLDATASIZE => {
            println!("CALLDATASIZE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let size = match tx {
                Some(tx) => {
                    let data = get_blockchain_data(&Some(tx.clone()), "data");
                    let data_str = format!("000{:x}", data);
                    data_str.len() / 2
                }
                None => 0,
            };
            v.push(U256::from(size));
        }

        Opcodes::CALLDATACOPY => {
            println!("CALLDATACOPY");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (dst_ost, ost, len) = get_three_mut_val(&mut v);
            let data = get_blockchain_data(tx, "data");
            let mut bytes = [0u8; 32];
            data.to_big_endian(&mut bytes);

            let first_n = &bytes[ost.as_usize()..ost.as_usize() + len.as_usize()];
            let mut memory = MEMORY.lock().unwrap();
            let mut msize = MSIZE.lock().unwrap();
            if memory.len() < dst_ost.as_usize() + len.as_usize() {
                memory.resize(first_n.len(), 0u8);
                *msize = first_n.len();
            }
            memory[dst_ost.as_usize()..].copy_from_slice(first_n);
        }

        Opcodes::CODESIZE => {
            println!("CODESIZE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            v.push(U256::from(original_code.len()));
        }

        Opcodes::CODECOPY => {
            println!("CODECOPY");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (dst_ost, ost, len) = get_three_mut_val(&mut v);
            let dst_ost = dst_ost.as_usize();
            let ost = ost.as_usize();
            let len = len.as_usize();

            let mut original_code_vec: Vec<u8> = vec![0; 32];
            if original_code.len() > 32 {
                original_code_vec.resize((original_code.len() / 33 + 1) * 32, 0);
            }

            original_code_vec[..original_code.len()].copy_from_slice(original_code);

            let mut memory = MEMORY.lock().unwrap();

            // let mut msize = MSIZE.lock().unwrap(); // TODO
            let resize = ((dst_ost + 32) / 33 + 1) * 32;

            if memory.len() < dst_ost + len {
                memory.resize(resize, 0u8);
                // *msize = first_n.len();
            }

            memory[dst_ost..dst_ost + len].copy_from_slice(&original_code_vec[ost..ost + len]);
            println!("memory: {:?}", memory);
        }

        Opcodes::GASPRICE => {
            println!("GASPRICE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(tx, "gasprice");

            v.push(value);
        }
        Opcodes::BASEFEE => {
            println!("BASEFEE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "basefee");

            v.push(value);
        }

        Opcodes::BLOCKHASH => {
            println!("BLOCKHASH");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let _ = get_one_mut_val(&mut v);

            v.push(U256::zero());
        }

        Opcodes::COINBASE => {
            println!("COINBASE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "coinbase");

            v.push(value);
        }

        Opcodes::TIMESTAMP => {
            println!("TIMESTAMP");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "timestamp");

            v.push(value);
        }
        Opcodes::NUMBER => {
            println!("NUMBER");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "number");

            v.push(value);
        }
        Opcodes::DIFFICULTY => {
            println!("DIFFICULTY");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "difficulty");

            v.push(value);
        }
        Opcodes::GASLIMIT => {
            println!("GASLIMIT");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "gaslimit");

            v.push(value);
        }
        Opcodes::CHAINID => {
            println!("CHAINID");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "chainid");

            v.push(value);
        }

        Opcodes::JUMP => {
            println!("JUMP");

            let (_, new_code) = get_n_bytes(&code, 1);
            println!("code: {:?}", code);
            println!("v: {:?}", v);
            println!("new code: {:?}", new_code);
            code = new_code;

            let dst: usize = v.pop().unwrap().as_usize();
            let result = jump(dst, pc, code);
            match result {
                Some(x) => {
                    code = x;
                }
                None => return None,
            }
        }

        Opcodes::PC => {
            println!("PC");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            println!("pc: {:?}", pc);
            v.push(U256::from(pc));
        }

        Opcodes::PUSH0 => {
            println!("PUSH0");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            v.push(U256::from_str(&"0x0").unwrap());
        }
        Opcodes::PUSHX => {
            println!("PUSHX IN MATCHER");
            let push: usize = (opcode - 95) as usize;
            pc += push + 1;
            let (bytes, new_code) = get_n_bytes(&code, push + 1);
            code = new_code;
            let (_, byte) = bytes.split_first().unwrap();
            let enc = hex::encode(&byte);
            v.push(U256::from_str(&enc).unwrap());
        }
        Opcodes::POP => {
            println!("POPPed");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let _ = v.pop();
        }
        Opcodes::MLOAD => {
            println!("MLOAD");
            pc += 1;

            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let offset = get_one_mut_val(&mut v);
            let offset = offset.as_usize();
            println!("offset: {}", offset);

            let required = ((offset + 32) / 33 + 1) * 32;
            println!("required: {}", (offset + 32));
            println!("required: {}", (offset + 32) / 33);
            println!("required: {}", (offset + 32) / 33 + 1);
            println!("required: {}", ((offset + 32) / 33 + 1) * 32);

            let mut memory = MEMORY.lock().unwrap();
            let mut msize = MSIZE.lock().unwrap();
            println!("required: {}", required);
            if memory.len() < required {
                memory.resize(required, 0u8);
                *msize = required;
            }

            let word = U256::from_big_endian(&memory[offset..offset + 32]);
            println!(
                "&memory[offset..offset + 32]: {:?}",
                &memory[offset..offset + 32]
            );
            println!("word: {:?}", word);
            v.push(word);
        }
        Opcodes::MSTORE => {
            println!("MSTORE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (offset, b) = get_mut_val(&mut v);
            let mut bytes = [0_u8; 32];

            b.to_big_endian(&mut bytes);
            let offset = offset.as_usize();

            let required = offset + 32;
            let mut memory = MEMORY.lock().unwrap();
            if memory[offset..].len() < required {
                memory.resize(required, 0u8);
            }
            memory[offset..offset + 32].copy_from_slice(&bytes);
        }

        Opcodes::MSTORE8 => {
            println!("MSTORE8");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (offset, value) = get_mut_val(&mut v);
            let mut bytes = [0_u8; 32];
            value.to_little_endian(&mut bytes);
            let offset = offset.as_usize();

            let mut memory = MEMORY.lock().unwrap();
            let mut msize = MSIZE.lock().unwrap();
            let required = offset + 1;
            if memory.len() < required {
                let size = ((required + 31) / 32) * 32;
                memory.resize(size, 0u8);
                *msize = size;
            }
            memory[offset..].copy_from_slice(&bytes[..1]);
        }
        Opcodes::JUMPI => {
            println!("JUMPI");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (dst, condition) = get_mut_val(&mut v);
            if condition == U256::zero() {
                pc += 1;
            } else {
                let dst: usize = dst.as_usize();
                let result = jump(dst, pc, code);
                match result {
                    Some(x) => {
                        code = x;
                    }
                    None => return None,
                }
            }
        }
        Opcodes::MSIZE => {
            println!("MSIZE");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let msize = MSIZE.lock().unwrap();
            v.push(U256::from(*msize));
        }
        Opcodes::GAS => {
            println!("GAS");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            v.push(
                U256::from_str(
                    "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                )
                .unwrap(),
            );
        }

        Opcodes::JUMPDEST => {
            println!("JUMPDEST");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
        }
        Opcodes::DUP => {
            println!("DUP");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let first_value = v.first().unwrap();
            v.push(*first_value);
        }
        Opcodes::SWAP => {
            println!("SWAP");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let first_value = v.first().unwrap().clone();
            let last_value = v.last().unwrap().clone();
            if let Some(first) = v.first_mut() {
                *first = last_value;
            }
            v.pop().unwrap();
            v.push(first_value);
        }
        Opcodes::INVALID => {
            println!("INVALID");
            return None;
        }
        _ => {
            todo!();
        }
    }

    run(&code, pc, original_code, v, tx, block, state)
}

pub fn evm(
    code: impl AsRef<[u8]>,
    tx: &Option<Value>,
    block: &Option<Value>,
    state: &Option<Value>,
) -> EvmResult {
    let stack: Vec<U256> = Vec::new();
    let v: Vec<U256> = vec![];

    let pc = 0;
    let code = code.as_ref();
    let original_code = code.clone();

    let mut evm_result = EvmResult {
        stack: vec![],
        success: true,
    };
    reset_memory_var();
    reset_msize_var();

    while pc < code.len() {
        // let opcode = code[pc];
        let res = run(code, pc, original_code, v, tx, block, state);

        match res {
            Some(mut value) => {
                value.reverse();
                evm_result.stack = value
            }
            None => evm_result.success = false,
        }
        return evm_result;
    }

    return EvmResult {
        stack: stack,
        success: false,
    };
}
