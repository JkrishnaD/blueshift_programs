use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    ProgramResult,
};
use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::{
    AccountCheck, AccountClose, AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenAccountInit, Escrow, MintAccount, ProgramAccount, SignerAccount
};

pub struct TakeAccounts<'a> {
    pub maker: &'a AccountInfo,
    pub taker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub mint_a: &'a AccountInfo,
    pub mint_b: &'a AccountInfo,
    pub taker_ata_a: &'a AccountInfo,
    pub taker_ata_b: &'a AccountInfo,
    pub maker_ata_b: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for TakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [maker, taker, escrow, vault, mint_a, mint_b, maker_ata_b, taker_ata_a, taker_ata_b, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        SignerAccount::check(taker)?;
        ProgramAccount::check(escrow)?;
        MintAccount::check(mint_a)?;
        MintAccount::check(mint_b)?;
        AssociatedTokenAccount::check(taker_ata_b, taker, mint_b)?;
        AssociatedTokenAccount::check(vault, escrow, mint_a)?;

        Ok(Self {
            maker,
            taker,
            escrow,
            vault,
            mint_a,
            mint_b,
            maker_ata_b,
            taker_ata_a,
            taker_ata_b,
            system_program,
            token_program,
        })
    }
}

// in this there will be no instructions because all the instructions that are need are
// already provided in the maker, taker just need to perform transaction and need to get
// his required tokens

pub struct Take<'a> {
    pub accounts: TakeAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Take<'a> {
    type Error = ProgramError;

    fn try_from(data: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = TakeAccounts::try_from(data)?;

        AssociatedTokenAccount::init_if_needed(
            accounts.taker_ata_a,
            accounts.mint_a,
            accounts.taker,
            accounts.taker,
            accounts.system_program,
            accounts.token_program,
        )?;

        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_b,
            accounts.mint_b,
            accounts.maker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self { accounts })
    }
}

impl<'a> Take<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

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

        // transfering the amount from taker to the maker
        Transfer {
            from: self.accounts.taker_ata_b,
            to: self.accounts.maker_ata_b,
            amount: escrow.receive,
            authority: self.accounts.taker,
        }
        .invoke()?;

        // transfering the amount from vault to taker
        Transfer {
            from: self.accounts.vault,
            to: self.accounts.taker_ata_a,
            amount: escrow.receive,
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
