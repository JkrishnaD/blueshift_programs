use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::{
    AccountCheck, AccountClose, AssociatedTokenAccount, AssociatedTokenCheck, Escrow,
    ProgramAccount, SignerAccount,
};

pub struct RefundAccounts<'a> {
    pub maker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub mint_a: &'a AccountInfo,
    pub maker_ata_a: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for RefundAccounts<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [maker, escrow, vault, mint_a, maker_ata_a, system_program, token_program, _] = data
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        SignerAccount::check(maker);
        ProgramAccount::check(escrow);
        AssociatedTokenAccount::check(vault, escrow, maker_ata_a)?;
        AssociatedTokenAccount::check(maker_ata_a, maker, mint_a)?;

        Ok(Self {
            maker,
            escrow,
            vault,
            mint_a,
            maker_ata_a,
            system_program,
            token_program,
        })
    }
}

pub struct Refund<'a> {
    pub accounts: RefundAccounts<'a>,
}

impl<'a> Refund<'a> {
    pub fn process(&mut self) -> ProgramResult {
        let data = self.accounts.escrow.try_borrow_data()?;
        let escrow = Escrow::load(&data)?;

        let seed_bindings = escrow.seed.to_le_bytes();
        let bump_bindings = escrow.bump;
        let seeds = [
            Seed::from(b"escrow"),
            Seed::from(self.accounts.maker.key().as_ref()),
            Seed::from(&seed_bindings),
            Seed::from(&bump_bindings),
        ];

        let signer_seeds = Signer::from(&seeds);

        Transfer {
            from: self.accounts.vault,
            to: self.accounts.maker_ata_a,
            amount: escrow.recieve,
            authority: self.accounts.escrow,
        }
        .invoke_signed(&[signer_seeds.clone()])?;

        CloseAccount {
            account: self.accounts.vault,
            authority: self.accounts.escrow,
            destination: self.accounts.maker,
        }
        .invoke_signed(&[signer_seeds.clone()])?;

        drop(data);
        ProgramAccount::close(self.accounts.escrow, self.accounts.maker)?;
        Ok(())
    }
}
