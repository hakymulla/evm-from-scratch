mod utils;
mod types;

use std::{ops::Div, str::FromStr, vec};
use std::panic;
// use anyhow::Ok;
// use anyhow::Ok;
use std::result::Result::Ok;
use ethers::core::k256::elliptic_curve::consts::U2;
use ethers::utils::hex::encode;
// use anyhow::Ok;
use keccak_hash::{self, keccak};
// use rlp::{self, Rlp};
use rlp::RlpStream;

// use ethers::core::utils::rlp;
use primitive_types::U256;
use serde_json::Value;
use serde_json::json;

use utils::{
    get_bin_state_data, get_blockchain_data, get_four_mut_val, get_mut_val, get_n_bytes,
    get_one_mut_val, get_state_data, get_three_mut_val, is_static_violation, jump, reset_memory_var, reset_msize_var, 
    to_signed, to_unsigned, MEMORY, MSIZE, RETURNDATA, STORAGE,
};

use crate::utils::{pop_v, reset_return_data};
use types::{CheckResult,EvmResult, EvmError, ExpectData, Log};

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
    EXTCODESIZE,
    EXTCODECOPY,
    RETURNDATASIZE,
    RETURNDATACOPY,
    EXTCODEHASH,
    BLOCKHASH,
    COINBASE,
    TIMESTAMP,
    NUMBER,
    DIFFICULTY,
    GASLIMIT,
    CHAINID,
    SELFBALANCE,
    BASEFEE,
    SLOAD,
    SSTORE,
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
    LOG0,
    LOG1,
    LOGX,
    CREATE,
    CALL,
    RETURN,
    DELEGATECALL,
    STATICCALL,
    REVERT,
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
            59 => Ok(Opcodes::EXTCODESIZE),
            60 => Ok(Opcodes::EXTCODECOPY),
            61 => Ok(Opcodes::RETURNDATASIZE),
            62 => Ok(Opcodes::RETURNDATACOPY),
            63 => Ok(Opcodes::EXTCODEHASH),
            64 => Ok(Opcodes::BLOCKHASH),
            65 => Ok(Opcodes::COINBASE),
            66 => Ok(Opcodes::TIMESTAMP),
            67 => Ok(Opcodes::NUMBER),
            68 => Ok(Opcodes::DIFFICULTY),
            69 => Ok(Opcodes::GASLIMIT),
            70 => Ok(Opcodes::CHAINID),
            71 => Ok(Opcodes::SELFBALANCE),
            72 => Ok(Opcodes::BASEFEE),
            84 => Ok(Opcodes::SLOAD),
            85 => Ok(Opcodes::SSTORE),
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
            160 => Ok(Opcodes::LOG0),
            161 => Ok(Opcodes::LOG1),
            162..=164 => Ok(Opcodes::LOGX),
            240 => Ok(Opcodes::CREATE),
            241 => Ok(Opcodes::CALL),
            243 => Ok(Opcodes::RETURN),
            244 => Ok(Opcodes::DELEGATECALL),
            250 => Ok(Opcodes::STATICCALL),
            253 => Ok(Opcodes::REVERT),
            254 => Ok(Opcodes::INVALID),

            _ => Err(EvmError::OpCodeError(value.to_string())),
        }
    }
}

fn run<>(
    code: &[u8],
    mut pc: usize,
    original_code: &[u8],
    mut v: Vec<U256>,
    mut logs: Vec<Log>,
    mut ret: String,
    tx: &Option<Value>,
    block: &Option<Value>,
    state: &mut Option<Value>,
) -> CheckResult {
    let mut code = code;

    println!("begininnng code: {:?}", code);
    if code.is_empty() {
        let r = ExpectData {
            stack: v,
            logs: logs,
            ret: ret
        };
        return CheckResult::Success(r);
    }

    let (opcode, _) = code.split_first().unwrap();
    println!("opcode: {}", opcode);
    let ops = Opcodes::try_from(opcode).expect("Invalid Opcode");

    match ops {
        Opcodes::STOP => {
            let r =  ExpectData {
                stack: v,
                logs: logs,
                ret: ret
            };
            return CheckResult::Success(r);
        }
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

            let value = get_blockchain_data(tx, "to").unwrap();

            v.push(value);
        }

        Opcodes::BALANCE => {
            println!("BALANCE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let mut address = get_one_mut_val(&mut v);
            println!("address in BALANCE: {}", address);
            if address == U256::max_value() {
                address = match state {
                    Some(_) => get_blockchain_data(tx, "to").unwrap(),
                    None => get_blockchain_data(tx, "from").unwrap()
                };
                println!("address in BALANCE: {}", address);

            }
                println!("address in BALANCE: {}", address);


            let address = format!("0x{:x}", address);
                println!("address in BALANCE: {}", address);

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

            let value = get_blockchain_data(tx, "origin").unwrap();

            v.push(value);
        }

        Opcodes::CALLER => {
            println!("CALLER");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = match state {
                Some(_) => get_blockchain_data(tx, "to").unwrap(),
                None => get_blockchain_data(tx, "from").unwrap()
            };

            v.push(value);
        }

        Opcodes::CALLVALUE => {
            println!("CALLVALUE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(tx, "value").unwrap();

            v.push(value);
        }

        Opcodes::CALLDATALOAD => {
            println!("CALLDATALOAD");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let idx = get_one_mut_val(&mut v).as_usize();
            let value = get_blockchain_data(tx, "data").unwrap();
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
                    let data = get_blockchain_data(&Some(tx.clone()), "data").unwrap();
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
            let data = get_blockchain_data(tx, "data").unwrap();
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

            let value = get_blockchain_data(tx, "gasprice").unwrap();

            v.push(value);
        }

        Opcodes::EXTCODESIZE => {
            println!("EXTCODESIZE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let address = get_one_mut_val(&mut v);
            let address = format!("0x{:x}", address);

            let val = match state {
                Some(value) => {
                    // let address_obj = value.get(address).unwrap();
                    // let code_obj = address_obj.get("code").unwrap();
                    // let bin = code_obj.get("bin").unwrap().as_str().unwrap();
                    let bin = get_bin_state_data(value, &address);
                    let code_size = bin.len() / 2;
                    let code_size = U256::from(code_size);
                    code_size
                }
                None => U256::zero(),
            };

            v.push(val);
        }

        Opcodes::EXTCODECOPY => {
            println!("EXTCODECOPY");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            println!("EXTCODECOPY");

            let addr = match state {
                Some(_) => get_blockchain_data(tx, "to").unwrap(),
                None => get_blockchain_data(tx, "from").unwrap()
            };

            let mut address = U256::default();
            let mut ost = U256::default();
            let mut len = U256::default();
            let mut dst_ost = U256::default();

            if addr == U256::default() {
                let (address_val, dst_ost_val, ost_val, len_val) = get_four_mut_val(&mut v);
                address = address_val;
                ost = ost_val;
                len = len_val;
                dst_ost = dst_ost_val;
            } else {
                let (dst_ost_val, ost_val, len_val) = get_three_mut_val(&mut v);
                println!("addr extcode: {}", addr);
                address = addr;
                ost = ost_val;
                len = len_val;
                dst_ost = dst_ost_val;
            }

            println!("address extcode: {}", address);
            println!("dst_ost: {}", dst_ost);
            println!("ost: {}", ost);
            println!("len: {}", len);

            let dst_ost = dst_ost.as_usize();
            let ost = ost.as_usize();
            let len = len.as_usize();

            if address == U256::max_value() {
                address = match state {
                    Some(_) => get_blockchain_data(tx, "to").unwrap(),
                    None => get_blockchain_data(tx, "from").unwrap()
                };
                println!("address in BALANCE: {}", address);
            }

            let address = format!("0x{:x}", address);

            let code_bytes = match state {
                Some(s) => hex::decode(get_bin_state_data(s, &address)).ok().unwrap_or_default(),
                    // .and_then(|bin| hex::decode(bin).ok())
                    // .unwrap_or_default(),
                None => vec![],  // ← no state = empty code
            };

            let mut memory = MEMORY.lock().unwrap();

            println!("code_bytes in extcode: {:?}", code_bytes);
            println!("memory in extcode: {:?}", memory);

            // expand memory if needed
            let required = dst_ost + len;
            if required > 0 {
                let words = (required + 31) / 32;
                if memory.len() < words * 32 {
                    memory.resize(words * 32, 0u8);
                }
            }

            // copy bytes, zero-pad if offset+size exceeds code length
            for i in 0..len {
                let code_idx = ost + i;
                memory[ost + i] = if code_idx < code_bytes.len() {
                    code_bytes[code_idx]
                } else {
                    0x00  // ← out of bounds = 0
                };
            }

            // let bin = match state {
            //     Some(value) => get_bin_state_data(value, &address),
            //     None => &String::new(),
            // };
            // let bin_hex = hex::decode(bin).unwrap();

            // let mut original_code_vec: Vec<u8> = vec![0; 32];
            // if bin.len() > 32 {
            //     original_code_vec.resize((original_code.len() / 33 + 1) * 32, 0);
            // }

            // original_code_vec[..bin_hex.len()].copy_from_slice(&bin_hex);

            // let mut memory = MEMORY.lock().unwrap();
            // // let mut msize = MSIZE.lock().unwrap(); // TODO
            // let resize = ((dst_ost + 32) / 33 + 1) * 32;

            // if memory.len() < dst_ost + len {
            //     memory.resize(resize, 0u8);
            //     // *msize = first_n.len();
            // }

            // memory[dst_ost..dst_ost + len].copy_from_slice(&original_code_vec[ost..ost + len]);
        }

        Opcodes::RETURNDATASIZE => {
            println!("RETURNDATASIZE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            println!("CODE:{:?}", code);


            let return_data = RETURNDATA.lock().unwrap();
            println!("return_data v: {:?}", return_data);

            if return_data.logs.is_empty() && return_data.ret.is_empty() && return_data.stack.is_empty(){
                v.push(U256::zero());
            } else if !return_data.ret.is_empty() {
                let byte_size = return_data.ret.as_bytes().len() / 2;
                v.push(U256::from(byte_size));
            } else {
                let n = return_data.stack.first().unwrap();
                let byte_size = (n.bits() + 7) / 8;
                v.push(U256::from(byte_size));
            }
        }

        Opcodes::RETURNDATACOPY => {
            println!("RETURNDATACOPY");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let input = pop_v(&mut v, 3);
            let (dst_ost, ost, len) = (input[0], input[1], input[2]);
            let dst_ost = dst_ost.as_usize();
            let len = len.as_usize();
            let ost: usize = ost.as_usize();

            let return_data = RETURNDATA.lock().unwrap();
            let return_data_bytes = hex::decode(&return_data.ret).unwrap();

            let mut memory = MEMORY.lock().unwrap();

            let resize = ((dst_ost + 32) / 33 + 1) * 32;

            if memory.len() < resize {
                memory.resize(resize, 0u8);
            }

            memory[dst_ost..dst_ost + len].copy_from_slice(&return_data_bytes[ost..]);
        }

        Opcodes::EXTCODEHASH => {
            println!("EXTCODEHASH");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let addr = get_one_mut_val(&mut v);
            let address = format!("0x{:x}", addr);

            let keccak_hash = match state {
                Some(value) => {
                    let bin = get_bin_state_data(value, &address);
                    let bin_hex = hex::decode(bin).unwrap();
                    let keccak_hash = keccak(bin_hex);
                    U256::from(keccak_hash.0)
                }
                None => U256::zero(),
            };
            v.push(keccak_hash);
        }

        Opcodes::BASEFEE => {
            println!("BASEFEE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "basefee").unwrap();

            v.push(value);
        }

        Opcodes::SLOAD => {
            println!("V : {:?}", v);

            println!("SLOAD");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let key = get_one_mut_val(&mut v);
            let storage = STORAGE.lock().unwrap();
            let val = storage.get(&key);
            let value = match val {
                Some(v) => v,
                None => &U256::from(0),
            };
            println!("V : {:?}", v);

            v.push(*value);
        }

        Opcodes::SSTORE => {
            println!("SSTORE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (key, val) = get_mut_val(&mut v);

            let mut storage = STORAGE.lock().unwrap();
            let _ = storage.entry(key).insert_entry(val);
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

            let value = get_blockchain_data(block, "coinbase").unwrap();

            v.push(value);
        }

        Opcodes::TIMESTAMP => {
            println!("TIMESTAMP");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "timestamp").unwrap();

            v.push(value);
        }
        Opcodes::NUMBER => {
            println!("NUMBER");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "number").unwrap();

            v.push(value);
        }
        Opcodes::DIFFICULTY => {
            println!("DIFFICULTY");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "difficulty").unwrap();

            v.push(value);
        }
        Opcodes::GASLIMIT => {
            println!("GASLIMIT");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "gaslimit").unwrap();

            v.push(value);
        }
        Opcodes::CHAINID => {
            println!("CHAINID");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(block, "chainid").unwrap();

            v.push(value);
        }

        Opcodes::SELFBALANCE => {
            println!("SELFBALANCE");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(tx, "to").unwrap();
            let address = format!("0x{:x}", value);

            let balance = match state {
                Some(value) => get_state_data(value, &address, "balance"),
                None => U256::zero(),
            };
            v.push(balance);
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
                None => return CheckResult::FailureNoData
                // panic!()
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
            if memory.len() < required {
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
                    None => return CheckResult::FailureNoData,
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

        Opcodes::LOG0 => {
            println!("LOG0");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let value = get_blockchain_data(tx, "to").unwrap();
            let address = format!("0x{:x}", value);

            let input = pop_v(&mut v, 2);
            let (offset, length) = (input[0], input[1]);

            let memory = MEMORY.lock().unwrap();
            let data =
                hex::encode(&memory[offset.as_usize()..offset.as_usize() + length.as_usize()]);
            let log = Log {
                address,
                data,
                topics: vec![],
            };
            logs.push(log);
        }
        Opcodes::LOG1 => {
            println!("LOG1");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let input = pop_v(&mut v, 3);
            let (offset, length, topic) = (input[0], input[1], input[2]);

            let value = get_blockchain_data(tx, "to").unwrap();
            let address = format!("0x{:x}", value);

            let memory = MEMORY.lock().unwrap();
            let data =
                hex::encode(&memory[offset.as_usize()..offset.as_usize() + length.as_usize()]);

            let mut topics: Vec<String> = vec![];
            let mut bytes = [0u8; 32];

            topic.to_big_endian(&mut bytes);
            let topic = format!("0x{}", hex::encode(bytes));

            topics.push(topic);

            let log = Log {
                address,
                data,
                topics,
            };
            logs.push(log);
        }
        Opcodes::LOGX => {
            println!("LOGX");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let log_size: usize = (opcode - 161) as usize;
            let n = 3 + log_size;

            let input = pop_v(&mut v, n);
            let (offset, length) = (input[0], input[1]);
            let value = get_blockchain_data(tx, "to").unwrap();
            let address = format!("0x{:x}", value);

            let memory = MEMORY.lock().unwrap();
            let data =
                hex::encode(&memory[offset.as_usize()..offset.as_usize() + length.as_usize()]);

            let mut topics: Vec<String> = vec![];
            for i in 2..n {
                let topic = input[i];
                let mut bytes = [0u8; 32];
                topic.to_big_endian(&mut bytes);
                let topic = format!("0x{}", hex::encode(bytes));
                topics.push(topic);
            }
            let log = Log {
                address,
                data,
                topics,
            };
            logs.push(log);
        }

        Opcodes::CREATE => {
            println!("CREATE");
            println!("v: {:?}", v);

            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let input = pop_v(&mut v, 3);
            let (value, offset, length,) = (input[0], input[1], input[2]);
            println!("value: {:?}", value);
            println!("offset: {:?}", offset);
            println!("length: {:?}", length);

            let addr = get_blockchain_data(tx, "to").unwrap();
            let address = format!("{:x}", addr);
            println!("address: {:?}", address);
            println!("value: {:?}", value);


            let memory = MEMORY.lock().unwrap();
            println!("memory: {:?}", memory);

            let data = hex::encode(&memory[offset.as_usize()..offset.as_usize() + length.as_usize()]);

            let address_bytes = hex::decode(&address).unwrap();
            let nonce: u8 = 0;
            let mut stream = RlpStream::new_list(2);
            stream.append(&address_bytes);
            stream.append(&nonce);
            let rlp_encode = stream.out();

            let contract_address = hex::encode(&keccak(&rlp_encode)[12..]);
            println!("{:?}", contract_address);

            let new_state: Option<serde_json::Value> = Some(json!({
                format!("0x{:x}", addr): {
                    "balance": value
                }
            }));


            *state = new_state;

            println!("state: {:?}", state);
        }

        Opcodes::CALL => {
            println!("CALL");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let input = pop_v(&mut v, 7);
            let (gas, addr, val, argost, arglen, retost, retlen) = (input[0], input[1], input[2], input[3], input[4], input[5], input[6]);
           
            let address = format!("0x{:x}", addr);

            let bin = match state {
                Some(value) => get_bin_state_data(value, &address),
                None => &String::new(),
            };

            let state_code: Vec<u8> = hex::decode(&bin).unwrap();

            let check_result = run(&state_code, pc, original_code, v.clone(), logs.clone(), ret.clone(), &tx.clone(), block, state);

            let memory = MEMORY.lock().unwrap();

            let result = panic::catch_unwind(|| {
                let input_data = hex::encode(&memory[argost.as_usize()..argost.as_usize() + arglen.as_usize()]);
                let ret_data = hex::encode(&memory[retost.as_usize()..retost.as_usize() + retlen.as_usize()]);
            });

            let mut return_data = RETURNDATA.lock().unwrap();
            return_data.logs.clear();
            return_data.ret.clear();
            return_data.stack.clear();

            match (check_result, result) {
                (CheckResult::Success(data), _) => {
                    v.push(U256::one());
                    *return_data = data;
                }
                (CheckResult::Failure(data), _) => {
                    v.push(U256::zero());
                    *return_data = data;
                }
                (CheckResult::FailureNoData, _) => {
                    v.push(U256::zero());
                }
                (_, Err(_)) => {
                    v.push(U256::zero());
                }
                (_, _) => v.push(U256::one())
            }
        }

        Opcodes::RETURN => {
            println!("RETURN");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let input = pop_v(&mut v, 2);
            let (offset, length) = (input[0], input[1]);

            let memory = MEMORY.lock().unwrap();
            let data = hex::encode(&memory[offset.as_usize()..offset.as_usize() + length.as_usize()]);

            println!("data: {}", data);
            ret.push_str(&data);
        }

        Opcodes::DELEGATECALL => {
            println!("DELEGATECALL");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let input = pop_v(&mut v, 6);
            let (gas, addr, argost, arglen, retost, retlen) = (input[0], input[1], input[2], input[3], input[4], input[5]);
            let addr = format!("0x{:x}", addr);

            let value = get_blockchain_data(tx, "to").unwrap();
            let address = format!("0x{:x}", value);

            let bin = match state {
                Some(value) => get_bin_state_data(value, &addr),
                None => &String::new(),
            };
            println!("bin: {}", bin);

            let state_code: Vec<u8> = hex::decode(&bin).unwrap();
            let check_result = run(&state_code, pc, original_code, v.clone(), logs.clone(), ret.clone(), &tx.clone(), block, state);
            match check_result{
                CheckResult::Success(_) => {
                    v.push(U256::one());
                }
                CheckResult::SuccessNoData => {
                    v.push(U256::one());
                }
                _ => {
                    v.push(U256::zero());
                }
            }
        }

         Opcodes::STATICCALL => {
            println!("STATICCALL");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let input = pop_v(&mut v, 6);
            let (gas, addr, argost, arglen, retost, retlen) = (input[0], input[1], input[2], input[3], input[4], input[5]);
            let addr = format!("0x{:x}", addr);

            let bin = match state {
                Some(value) => get_bin_state_data(value, &addr),
                None => &String::new(),
            };
            println!("bin: {}", bin);

            let state_code: Vec<u8> = hex::decode(&bin).unwrap();
            println!("bin: {:?}", bin);
            println!("state_code: {:?}", state_code);
            println!("before check result: {:?}", opcode);
            if is_static_violation(&state_code) {
                v.push(U256::zero());
            } else {
                let check_result = run(&state_code, pc, original_code, v.clone(), logs.clone(), ret.clone(), &tx.clone(), block, state);
                println!("after check result: {:?}", opcode);

                println!("check_result: {:?}", check_result);

                match check_result{
                    CheckResult::Success(_) => {
                        v.push(U256::one());
                    }
                    CheckResult::SuccessNoData => {
                        v.push(U256::one());
                    }
                    _ => {
                        v.push(U256::zero());
                    }
                }
            }
            
        }

        Opcodes::REVERT => {
            println!("REVERT");
            pc += 1;
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let input = pop_v(&mut v, 2);
            let (offset, length) = (input[0], input[1]);

            let memory = MEMORY.lock().unwrap();
            let data = hex::encode(&memory[offset.as_usize()..offset.as_usize() + length.as_usize()]);

            ret.push_str(&data);
            println!("data: {}", data);
            // return Err()
            let r = ExpectData {
                stack: v,
                logs: logs,
                ret: ret
            };
            return CheckResult::Failure(r);
        }

        Opcodes::INVALID => {
            println!("INVALID");
            // panic!();
            return CheckResult::FailureNoData;

        }
    }

    run(&code, pc, original_code, v, logs, ret, tx, block, state)
    
}

pub fn evm(
    code: impl AsRef<[u8]>,
    tx: &Option<Value>,
    block: &Option<Value>,
    state: &mut Option<Value>,
) -> EvmResult {
    // let stack: Vec<U256> = Vec::new();
    let v: Vec<U256> = vec![];
    let logs: Vec<Log> = vec![];
    let ret = String::new();

    let pc = 0;
    let code = code.as_ref();
    let original_code = code;

    let mut evm_result = EvmResult {
        stack: vec![],
        logs: vec![],
        ret: String::new(),
        success: true,
    };
    reset_memory_var();
    reset_msize_var();
    reset_return_data();

    let _ = while pc < code.len() {
        // let opcode = code[pc];
        let check_result = run(code, pc, original_code, v, logs, ret, tx, block, state);
        println!("RESULTTTT: {:?}", check_result);

        // let mut res: Option<ExpectData> = None;

        // let result_unwind = panic::catch_unwind(|| {
        //     run(code, pc, original_code, v, logs, ret, tx, block, state)
        // });
        println!("remaining code: {:?}", code);

        match check_result{
            CheckResult::Success(value) => {
                let mut stack_value = value.stack;
                let logs_value = value.logs;
                let ret_value = value.ret;
                stack_value.reverse();
                evm_result.stack = stack_value;
                evm_result.logs = logs_value;
                evm_result.ret = ret_value.clone();
            }
            CheckResult::SuccessNoData => {}
            CheckResult::Failure(value) => {
                let ret_value = value.ret;
                evm_result.ret = ret_value.clone();
                evm_result.success = false
            }
            CheckResult::FailureNoData => {
                evm_result.success = false
            }

        }


        // match res {
        //     Ok(result) => {
        //         println!("result_unwind: {:?}", result);
        //         match result {
        //             Some(value) => {
        //                 let mut stack_value = value.stack;
        //                 let logs_value = value.logs;
        //                 let ret_value = value.ret;
        //                 stack_value.reverse();
        //                 evm_result.stack = stack_value;
        //                 evm_result.logs = logs_value;
        //                 evm_result.ret = ret_value.clone();
        //             }
        //             None => {
        //                 evm_result.success = false
        //             }
        //         }
        //     }
        //     Err(value) => {
        //         let ret_value = value.unwrap().ret;
        //         evm_result.ret = ret_value.clone();
        //         evm_result.success = false
        //     }
        // }

        return evm_result
    };

    return EvmResult {
        stack: v,
        logs,
        ret,
        success: false,
    };
}

