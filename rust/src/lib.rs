use core::num;
use std::{ops::Div, str::FromStr};
use std::{error::Error, fmt::Display};
use primitive_types::U256;
use serde_json::from_value;

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
    SHL,
    SHR,
    PUSH0,
    PUSHX,
    POP
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
            27 => Ok(Opcodes::SHL),
            28 => Ok(Opcodes::SHR),
            95 => Ok(Opcodes::PUSH0),
            96..=127 => Ok(Opcodes::PUSHX),
            80 => Ok(Opcodes::POP),
            _ => Err(EvmError::OpCodeError(value.to_string()))
        }
    }
}

fn run(code: &[u8], mut pc: usize, mut v: Vec<U256>) -> Option<Vec<U256>> {
    let mut code = code;
    // println!("begininnng code: {:?}", code);
    if code.is_empty() {
        return Some(v)
    }

    pc += 1;
    let (opcode, _) = code.split_first().unwrap();
    println!("opcode: {}", opcode);
    let ops = Opcodes::try_from(opcode).expect("Invalid Opcode");

    match ops {
        Opcodes::STOP => return Some(v),
        Opcodes::ADD => {
            println!("ADD");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            let res =  a.overflowing_add(b);
            v.push(res.0);
        },

        Opcodes::MUL => {
            println!("MUL");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            let res =  a.overflowing_mul(b);
            v.push(res.0);
        },
        Opcodes::SUB => {
            println!("SUB");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            let res = a.overflowing_sub(b);
            v.push(res.0);
        },
        Opcodes::DIV => {
            println!("DIV");
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
        },
        Opcodes::SDIV => {
            println!("SDIV");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            println!("a : {:?}", a);
            println!("b : {:?}", b);
            let (a_neg, a_mag) = to_signed(a);
            let (b_neg, b_mag) = to_signed(b);

            // RULE 3: overflow special case
            // -2²⁵⁵ / -1 → return -2²⁵⁵ (same bits back)
            let min_value = U256::one() << 255;  // 0x80...00

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
        },

        Opcodes::MOD => {
            println!("MOD");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            if b == U256::zero() {
                v.push(U256::zero());
            } else {
                let res = a.div_mod(b);
                v.push(res.1);
            }
            
        },
        Opcodes::SMOD => {
            println!("SMOD");
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
            
        },
        Opcodes::ADDMOD => {
            println!("ADDMOD");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b, c) = get_three_mut_val(&mut v);

            let res = (a.saturating_add(b)).div_mod(c);
            println!("res: {:?}", res);


            v.push(res.1);
            
        },
        Opcodes::MULMOD => {
            println!("MULMOD");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b, c) = get_three_mut_val(&mut v);
            let res = ((a % c) * (b % c)) % c;
            v.push(res);
        },

        Opcodes::EXP => {
            println!("EXP");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            let res = a.pow(b);
            v.push(res);
        },
        Opcodes::SIGNEXTEND => {
            println!("SIGNEXTEND");

            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);

            let mut bytes = [0u8; 32];
            b.to_big_endian(&mut bytes);
            
            let hex_string = hex::encode(&bytes);
            println!("hex_string: {:?}", hex_string);

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
        },
        Opcodes::LT => {
            println!("LT");
            // Some(("0x0".to_string(), 0))
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            if a < b {
                v.push(U256::one());
            } else {
                v.push(U256::zero());
            }
        },

        Opcodes::GT => {
            println!("GT");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            if a > b {
                v.push(U256::one());
            } else {
                v.push(U256::zero());
            }
        },

        Opcodes::SLT => {
            println!("SLT");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);


            let (a_neg, a_mag) = to_signed(a);
            let (b_neg, b_mag) = to_signed(b);
            println!("a: {:?}", a);
            println!("b: {:?}", b);
            println!("a_neg: {:?}", a_neg);
            println!("b_neg: {:?}", b_neg);
            if a_neg > b_neg {
                v.push(U256::one());
            } else {
                v.push(U256::zero());
            }
        },

        Opcodes::SGT => {
            println!("SGT");
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
        },
        
        Opcodes::EQ => {
            println!("EQ");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);

            if a == b {
                v.push(U256::one());
            } else {
                v.push(U256::zero());
            }
        },
        Opcodes::ISZERO => {
            println!("ISZERO");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let a = get_one_mut_val(&mut v);

            if a == U256::zero() {
                v.push(U256::one());
            } else {
                v.push(U256::zero());
            }
        },
         Opcodes::AND => {
            println!("AND");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            v.push(a&b);
        },
        Opcodes::OR => {
            println!("OR");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            v.push(a|b);
        },
        Opcodes::XOR => {
            println!("XOR");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (a, b) = get_mut_val(&mut v);
            v.push(a^b);
        },
        Opcodes::NOT => {
            println!("NOT");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let a = get_one_mut_val(&mut v);

            v.push(!a);
        },
        Opcodes::SHL => {
            println!("SHL");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (shift, a) = get_mut_val(&mut v);

            if shift >= U256::from(256) {
                 v.push(U256::zero());
            } else if a < U256::max_value() {
                v.push(a<<shift);
            }
        },

        Opcodes::SHR => {
            println!("SHR");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;

            let (shift, a) = get_mut_val(&mut v);

            if shift >= U256::from(256) {
                 v.push(U256::zero());
            } else if a < U256::max_value() {
                v.push(a>>shift);
            }
        },
        
        Opcodes::PUSH0 => {
            println!("PUSH0");
            // Some(("0x0".to_string(), 0))
            let (bytes, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            println!("new_code: {:?}", new_code);
            println!("bytes: {:?}", bytes);
            v.push(U256::from_str(&"0x0").unwrap());
        },
        Opcodes::PUSHX => {
            println!("PUSHX IN MATCHER");
            let push: usize  = (opcode - 95) as usize;
            let (bytes, new_code) = get_n_bytes(&code, push + 1);
            println!("new_code: {:?}", new_code);
            println!("bytes: {:?}", bytes);
            println!("push: {:?}", push);
            code = new_code;
            let (_, byte) = bytes.split_first().unwrap();
            // println!("push: {:?}", push);
            let enc = hex::encode(&byte);
            // println!("enc: {:?}", enc);
            // Some((enc, push))
            v.push(U256::from_str(&enc).unwrap());
            // used_byte = push;
        },
        Opcodes::POP => {
            println!("POPPed");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let _ = v.pop();
        }
        _ => {
            todo!();
        }
    }

    run(&code, pc, v)
    // Some(v)
}
fn get_one_mut_val(v: &mut Vec<U256>) -> U256 {
    let a = v.pop().unwrap();
    a
}

fn get_mut_val(v: &mut Vec<U256>) -> (U256, U256) {
    let a = v.pop().unwrap();
    let b = v.pop().unwrap();
    (a, b)
}

fn get_three_mut_val(v: &mut Vec<U256>) -> (U256, U256, U256) {
    let a = v.pop().unwrap();
    let b = v.pop().unwrap();
    let c = v.pop().unwrap();
    (a, b, c)
}


pub fn evm(_code: impl AsRef<[u8]>) -> EvmResult {
    let stack: Vec<U256> = Vec::new();
    let mut v: Vec<U256> = vec![];

    let mut pc = 0;

    let code = _code.as_ref();
    let mut evm_result = EvmResult{stack: vec![], success: true};

    while pc < code.len() {
        let opcode = code[pc];
        let res = run(code, pc, v);

        if let Some(mut value) = res {
            value.reverse();
            evm_result.stack = value
        }

        return evm_result;
    }

    return EvmResult {
        stack: stack,
        success: true,
    };
}


fn get_n_bytes(value: &[u8], byte: usize) -> (&[u8], &[u8]) {
    (&value[..byte],  &value[byte..])
}

fn from_bits(bits: Vec<u8>) -> u8 {
    let mut v = 0;
    for (i, bit) in bits.iter().enumerate() {
        v |= bit << (7 - i);
    }
    v
}

fn to_bits(v: u8) -> [u8; 8] {
    let mut bits = [0; 8];
    for (i, bit) in bits.iter_mut().enumerate() {
        *bit = (v >> (7 - i)) & 1;
    }
    bits
}

fn to_signed(x: U256) -> (bool, U256) {
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

fn to_unsigned(is_negative: bool, magnitude: U256) -> U256 {
    if !is_negative {
        // positive, bits are just the magnitude
        magnitude
    } else {
        // negative, reverse two's complement
        (!magnitude) + U256::one()
    }
}