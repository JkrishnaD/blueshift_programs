use pinocchio::{
    account_info::AccountInfo, instruction::{Seed, Signer}, program_error::ProgramError, pubkey::create_program_address, ProgramResult
};
use pinocchio_token::instructions::{CloseAccount, Transfer};

use crate::{
    AccountCheck, AccountClose, AssociatedTokenAccount, AssociatedTokenAccountCheck,
    AssociatedTokenAccountInit, Escrow, MintAccount, ProgramAccount, SignerAccount,
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

        SignerAccount::check(maker)?;
        ProgramAccount::check(escrow)?;
        MintAccount::check(mint_a)?;
        AssociatedTokenAccount::check(vault, escrow, mint_a)?;
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

impl<'a> TryFrom<&'a [AccountInfo]> for Refund<'a> {
    type Error = ProgramError;

    fn try_from(value: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = RefundAccounts::try_from(value)?;

        AssociatedTokenAccount::init_if_needed(
            accounts.maker_ata_a,
            accounts.mint_a,
            accounts.maker,
            accounts.maker,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self { accounts })
    }
}

impl<'a> Refund<'a> {
    pub const DISCRIMINATOR: &'a u8 = &2;

    pub fn process(&mut self) -> ProgramResult {
        let data = self.accounts.escrow.try_borrow_data()?;
        let escrow = Escrow::load(&data)?;

        let escrow_key = create_program_address(
            &[
                b"escrow",
                self.accounts.maker.key(),
                &escrow.seed.to_le_bytes(),
                &escrow.bump,
            ],
            &crate::ID,
        )?;
        if &escrow_key != self.accounts.escrow.key() {
            return Err(ProgramError::InvalidAccountOwner);
        }

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
