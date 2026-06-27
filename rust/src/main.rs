/**
 * EVM From Scratch
 * Rust template
 *
 * To work on EVM From Scratch in Rust:
 *
 * - Install Rust: https://www.rust-lang.org/tools/install
 * - Edit `rust/lib.rs`
 * - Run `cd rust && cargo run` to run the tests
 *
 * Hint: most people who were trying to learn Rust and EVM at the same
 * gave up and switched to JavaScript, Python, or Go. If you are new
 * to Rust, implement EVM in another programming language first.
 */
use evm::evm;
use primitive_types::U256;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
struct Evmtest {
    name: String,
    hint: String,
    code: Code,
    expect: Expect,
    tx: Option<serde_json::Value>,
    block: Option<serde_json::Value>,
    state: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct Code {
    asm: String,
    bin: String,
}

#[derive(Debug, Deserialize)]
struct Expect {
    stack: Option<Vec<String>>,
    success: bool,
    logs: Option<Vec<Log>>,
    #[serde(rename = "return")]
    ret: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Log {
    pub address: String,
    pub data: String,
    pub topics: Vec<String>,
}

fn main() {
    let text = std::fs::read_to_string("../evm.json").unwrap();
    let mut data: Vec<Evmtest> = serde_json::from_str(&text).unwrap();

    let total = data.len();

    for (index, test) in data.iter_mut().enumerate() {
        println!("Test {} of {}: {}", index + 1, total, test.name);

        let code: Vec<u8> = hex::decode(&test.code.bin).unwrap();
        let tx = &test.tx;
        let block = &test.block;
        // let mut state = &test.state;
        println!("tx: {:?}", tx);
        println!("block: {:?}", block);

        println!("test.code.bin: {:?}", test.code.bin);
        println!("code: {:?}", code);

        // let result = evm(&code, tx, block, state);
        let result = evm(&code, tx, block, &mut test.state);
        println!("result: {:?}", result);


        let mut expected_stack: Vec<U256> = Vec::new();
        if let Some(ref stacks) = test.expect.stack {
            for value in stacks {
                expected_stack.push(U256::from_str_radix(value, 16).unwrap());
            }
        }

        let mut matching = result.stack.len() == expected_stack.len();

        let mut expected_log: Vec<Log> = Vec::new();
        if let Some(ref stacks) = test.expect.logs {
            for value in stacks {
                expected_log.push(value.clone());
            }
        }

        let mut matching_log = result.logs.len() == expected_log.len();
        println!("result.logs: {:?}", result.logs);
        println!("expected_log: {:?}", expected_log);
        
        // Matching Stack
        if matching {
            for i in 0..result.stack.len() {
                if result.stack[i] != expected_stack[i] {
                    matching = false;
                    break;
                }
            }
        }

        // Matching Logs
        if matching_log {
            for i in 0..result.logs.len() {
                if result.logs[i].address != expected_log[i].address
                    || result.logs[i].data != expected_log[i].data
                    || result.logs[i].topics != expected_log[i].topics
                {
                    matching_log = false;
                    break;
                }
                println!("result.logs {:?}", result.logs[i]);
            }
        }
        println!("matching {:?}", matching);

        // Matching Return
        let expected_return = match &test.expect.ret {
            Some(ret) => ret,
            None => &String::new()
        };

        let matching_ret = result.ret == *expected_return;

        matching = matching && matching_log && matching_ret && result.success == test.expect.success;
        println!("matching_log {:?}", matching_log);

        if !matching {
            println!("Instructions: \n{}\n", test.code.asm);

            println!("Expected success: {:?}", test.expect.success);
            println!("Expected stack: [");
            for v in expected_stack {
                println!("  {:#X},", v);
            }
            println!("]\n");
            println!("Expected logs: [");
            for v in expected_log {
                println!("  {:?},", v);
            }
            println!("]\n");

            println!("Expected Return: {:?}", expected_return);
            println!("\n");


            println!("Actual success: {:?}", result.success);
            println!("Actual stack: [");
            for v in result.stack {
                println!("  {:#X},", v);
            }
            println!("]\n");

            println!("Actual logs: [");
            for v in result.logs {
                println!("  {:?},", v);
            }
            println!("]\n");
            println!("Actual Return: {:?}", result.ret);


            println!("\nHint: {}\n", test.hint);
            println!("Progress: {}/{}\n\n", index, total);
            panic!("Test failed");
        }
        println!("PASS");
    }
    println!("Congratulations!");
}
