use serde::Deserialize;
use primitive_types::U256;

#[derive(Deserialize, Debug, Clone)]
pub struct Log {
    pub address: String,
    pub data: String,
    pub topics: Vec<String>,
}

#[derive(Debug)]
pub struct EvmResult {
    pub stack: Vec<U256>,
    pub logs: Vec<Log>,
    pub ret: String,
    pub success: bool,
}

#[derive(Debug)]
pub enum CheckResult {
    Success(ExpectData),       // succeeded, with data to verify
    SuccessNoData,             // succeeded, nothing to check
    Failure(ExpectData),       // failed (expected or not), with data
    FailureNoData,             // failed, nothing to check
}

#[derive(Debug)]
pub enum EvmError {
    OpCodeError(String),
}


#[derive(Debug)]
pub struct ExpectData {
    pub stack: Vec<U256>,
    pub logs: Vec<Log>,
    pub ret: String
}