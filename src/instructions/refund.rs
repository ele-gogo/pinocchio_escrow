//! Refund 指令实现（取消 Escrow 并返还代币）
//!
//! 本文件实现 `Refund` 指令：
//! - 验证调用者为 Escrow 的创建者
//! - 将 vault 中的 Token A 返还给 maker
//! - 关闭 vault 并关闭 Escrow PDA，将租金返还给 maker
//!
use crate::helper::{AssociatedTokenAccount, MintInterface, ProgramAccount, SignerAccount};
use crate::state::Escrow;
use crate::{
    AccountCheck, AccountClose, AssociatedTokenAccountCheck, AssociatedTokenAccountInit,
    ProgramAccountInit,
};
use core::mem::size_of;
use pinocchio::{
    address::address,
    cpi::{Seed, Signer},
    error::ProgramError,
    nostd_panic_handler, AccountView, Address, ProgramResult,
};
use pinocchio_token::instructions::{CloseAccount, Transfer};
use pinocchio_token::state::TokenAccount;

// ========== 账户结构 ==========
pub struct RefundAccounts<'a> {
    pub maker: &'a AccountView,       // 托管创建者（必须是签名者）
    pub escrow: &'a AccountView,      // Escrow PDA 账户
    pub mint_a: &'a AccountView,      // Token A 的 Mint
    pub vault: &'a AccountView,       // Vault（存储 Token A 的 ATA）
    pub maker_ata_a: &'a AccountView, // Maker 的 Token A ATA（接收返还的代币）
    pub system_program: &'a AccountView,
    pub token_program: &'a AccountView,
}

pub struct RefundAccounts1 {
    pub maker: AccountView, //验证 签名
    pub escrow: AccountView, //验证pda
    pub mint_a: AccountView,//验证mint
    pub vault: AccountView,//验证 ATA 
    pub maker_ata_a: AccountView,//验证 ATA
    pub system_program: AccountView,
    pub token_program: AccountView,
}

impl<'a> TryFrom<&'a [AccountView]> for RefundAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, vault, maker_ata_a, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // 账户基础校验
        SignerAccount::check(maker)?;
        ProgramAccount::check(escrow)?;
        MintInterface::check(mint_a)?;
         AssociatedTokenAccount::check(vault, escrow, mint_a, token_program)?;
         AssociatedTokenAccount::check(maker_ata_a, maker, mint_a, token_program)?;

        // 返回账户
        Ok(Self {
            maker,
            escrow,
            mint_a,
            vault,
            maker_ata_a,
            system_program,
            token_program,
        })
    }
}

// ========== 指令数据结构（Refund 无需额外数据）==========
pub struct Refund<'a> {
    pub accounts: RefundAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountView]> for Refund<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountView]) -> Result<Self, Self::Error> {
        let accounts = RefundAccounts::try_from(accounts)?;

        // 创建创建者的代币 A ATA（如果不存在）
        // 对应 Anchor: pub maker_ata_a 的 init_if_needed 约束
        // helpers.rs 中的 init_if_needed 实现：
        // - 先尝试验证账户（check）
        // - 如果验证失败，说明账户不存在，调用 init 创建
        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_a,    // 要创建/验证的账户
            accounts.mint_a,         // mint 账户
            accounts.maker,          // payer：对应 Anchor 的 payer = maker
            accounts.maker,          // owner：对应 Anchor 的 authority = maker
            accounts.system_program, // System Program
            accounts.token_program,  // Token Program
        )?;

        // 返回完整的指令结构
        Ok(Self { accounts })
    }
}

impl<'a> Refund<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&mut self) -> ProgramResult {
        // 1. 加载 Escrow 账户数据并验证 PDA
        let data = self.accounts.escrow.try_borrow()?;
        let escrow = Escrow::load(&data)?;

        // 验证 Escrow PDA 是否有效（使用 create_program_address 检验）
        let escrow_key = Address::create_program_address(
            &[
                b"escrow",
                self.accounts.maker.address().as_ref(),
                &escrow.seed.to_le_bytes(),
                &escrow.bump,
            ],
            &crate::ID,
        )?;
        if &escrow_key != self.accounts.escrow.address() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // 验证调用者是 Escrow 的创建者（maker）
        if self.accounts.maker.address() != &escrow.maker {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // 2. 构建 Escrow PDA 的签名种子（用于带签名调用）
        let seed_binding = escrow.seed.to_le_bytes();
        let bump_binding = escrow.bump;
        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.address().as_ref()),
            Seed::from(&seed_binding),
            Seed::from(&bump_binding),
        ];
        let signer = Signer::from(&escrow_seeds);

        // 3. 从 Vault 中提取 Token A 的余额
        let vault_amount = TokenAccount::from_account_view(self.accounts.vault)?.amount();

        // 4. 将 Token A 从 Vault 转账回 Maker
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.maker_ata_a,
            authority: self.accounts.escrow, // Escrow PDA 作为 vault 的所有者
            amount: vault_amount,
        }
        .invoke_signed(&[signer.clone()])?;

        // 5. 关闭 Vault ATA 账户（将租金返还给 maker）
        CloseAccount {
            account: self.accounts.vault,
            destination: self.accounts.maker,
            authority: self.accounts.escrow,
        }
        .invoke_signed(&[signer.clone()])?;

        // 6. 关闭 Escrow PDA 账户（将租金返还给 maker）
        drop(data); // 释放借用的数据
        ProgramAccount::close(self.accounts.escrow, self.accounts.maker)?;

        Ok(())
    }
}
