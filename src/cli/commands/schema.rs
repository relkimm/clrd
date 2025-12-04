//! Schema command - Output JSON schema for LLM integration

use crate::mapper::templates::DEAD_CODE_JSON_SCHEMA;
use anyhow::Result;

pub async fn run() -> Result<i32> {
    println!("{}", DEAD_CODE_JSON_SCHEMA);
    Ok(0)
}
