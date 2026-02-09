//! Blueshift Escrow 程序入口与模块导出
//!
//! 这是一个简化的 Solana 智能合约（使用 pinocchio 框架）的顶层库文件。
//! - 定义程序 ID
//! - 导出子模块 `instructions`、`state`、`errors`
//! - 实现简单的指令分发（根据第一字节的 discriminator 路由到具体指令处理器）
//!
//! 使用说明（快速）：
//! 1. 构建：`cargo build --target wasm32-unknown-unknown`
//! 2. 测试：`cargo test`
//!
#![no_std]
use pinocchio::{
    address::address, entrypoint, error::ProgramError, nostd_panic_handler, AccountView, Address,
    ProgramResult,
};
use state::Escrow;

nostd_panic_handler!();
entrypoint!(process_instruction);

pub mod instructions;
pub use instructions::*;

pub mod state;
pub use state::*;
pub mod errors;
pub use errors::*;
// 程序 ID（示例占位地址）
pub const ID: Address = address!("22222222222222222222222222222222222222222222");

fn process_instruction(
    _program_id: &Address,
    accounts: &[AccountView],
    instruction_data: &[u8],
) -> ProgramResult {
    match instruction_data.split_first() {
        Some((Make::DISCRIMINATOR, data)) => Make::try_from((data, accounts))?.process(),
        Some((Take::DISCRIMINATOR, _)) => Take::try_from(accounts)?.process(),
        Some((Refund::DISCRIMINATOR, _)) => Refund::try_from(accounts)?.process(),
        _ => Err(ProgramError::InvalidInstructionData)
    }
}