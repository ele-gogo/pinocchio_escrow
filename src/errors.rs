//! 错误定义（中文注释）
//!
//! 本文件列举了程序中可能返回的自定义错误类型（映射为 ProgramError::Custom）。
//! 每个错误带有简短中文说明，便于本地化调试与日志阅读。
//!
use pinocchio::error::ProgramError;
use core::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EscrowError {
    /// 账户余额低于租金豁免阈值
    /// Solana 要求账户必须持有足够的 lamports 以免被删除
    /// 如果余额不足，账户可能被垃圾回收
    NotRentExempt = 0,

    /// 账户未签名
    /// 某些操作需要特定的账户签名（如创建者、接受者）
    /// 如果应该签名的账户没有签名，返回此错误
    NotSigner = 1,

    /// 非法的账户所有者
    /// Solana 账户由特定程序拥有（通过 owner 字段指定）
    /// 如果账户的所有者不是预期的程序，返回此错误
    /// 例如：Token Account 的 owner 应该是 Token Program
    InvalidOwner = 2,

    /// 非法的账户数据
    /// 账户数据的长度或格式不符合预期
    /// 例如：期望的账户数据长度与实际不匹配
    InvalidAccountData = 3,

    /// 非法的地址
    /// 提供的地址不符合预期要求
    /// 例如：PDA 派生失败、地址不匹配等
    InvalidAddress = 4,
}

impl From<EscrowError> for ProgramError {
    fn from(error: EscrowError) -> Self {
        ProgramError::Custom(error as u32)
    }
}

impl fmt::Display for EscrowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EscrowError::NotRentExempt => write!(f, "Lamport balance below rent-exempt threshold"),
            EscrowError::NotSigner => write!(f, "没有签名"),
            EscrowError::InvalidOwner => write!(f, "非法的所有者"),
            EscrowError::InvalidAccountData => write!(f, "非法的账户数据"),
            EscrowError::InvalidAddress => write!(f, "非法的地址"),
        }
    }
}