use std::{ops::Div, str::FromStr};
use std::{error::Error, fmt::Display};
use primitive_types::U256;

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
    MOD,
    ADDMOD,
    MULMOD,
    EXP,
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
            0x06 => Ok(Opcodes::MOD),
            0x08 => Ok(Opcodes::ADDMOD),
            0x09 => Ok(Opcodes::MULMOD),
            10 => Ok(Opcodes::EXP),
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
            if b == U256::from(0) {
                v.push(U256::from(0));
            } else {
                let res = a.div(b);
                v.push(res);
            }
        },

        Opcodes::MOD => {
            println!("MOD");
            let (_, new_code) = get_n_bytes(&code, 1);
            code = new_code;
            let (a, b) = get_mut_val(&mut v);
            if b == U256::from(0) {
                v.push(U256::from(0));
            } else {
                let res = a.div_mod(b);
                v.push(res.1);
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