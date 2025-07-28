use pinocchio::{
    account_info::AccountInfo, instruction::Seed, program_error::ProgramError,
    pubkey::find_program_address, ProgramResult,
};
use pinocchio_token::instructions::Transfer;

use crate::{
    AccountCheck, AssociatedTokenAccount, AssociatedTokenAccountCheck, AssociatedTokenAccountInit, Escrow, MintAccount, ProgramAccount, ProgramAccountInit, SignerAccount
};

// accounts which are required for the Make function
pub struct MakeAccounts<'a> {
    pub maker: &'a AccountInfo,
    pub escrow: &'a AccountInfo,
    pub mint_a: &'a AccountInfo,
    pub mint_b: &'a AccountInfo,
    pub maker_ata_a: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub token_program: &'a AccountInfo,
}

// tryform implementation which is used to verify the accounts if they are valid or not
impl<'a> TryFrom<&'a [AccountInfo]> for MakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [maker, escrow, mint_a, mint_b, maker_ata_a, vault, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        SignerAccount::check(maker)?;
        MintAccount::check(mint_a)?;
        MintAccount::check(mint_b)?;
        AssociatedTokenAccount::check(maker_ata_a, maker, mint_a)?;
        Ok(Self {
            maker,
            escrow,
            mint_a,
            mint_b,
            maker_ata_a,
            vault,
            system_program,
            token_program,
        })
    }
}

// the instructions for the Make function
pub struct MakeInstructionData {
    pub seed: u64,
    pub recieve: u64,
    pub amount: u64,
}

impl<'a> TryFrom<&'a [u8]> for MakeInstructionData {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() != size_of::<u64>() * 3 {
            return Err(ProgramError::InvalidAccountData);
        };

        let seed = u64::from_le_bytes(data[0..8].try_into().unwrap());
        let recieve = u64::from_le_bytes(data[8..16].try_into().unwrap());
        let amount = u64::from_le_bytes(data[16..24].try_into().unwrap());

        if amount == 0 {
            return Err(ProgramError::InvalidInstructionData);
        };

        Ok(Self {
            seed,
            recieve,
            amount,
        })
    }
}

pub struct Make<'a> {
    pub accounts: MakeAccounts<'a>,
    pub instructions: MakeInstructionData,
    pub bump: u8,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Make<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = MakeAccounts::try_from(accounts)?;
        let instructions = MakeInstructionData::try_from(data)?;

        let (_, bump) = find_program_address(
            &[
                b"escrow",
                accounts.maker.key(),
                &instructions.seed.to_le_bytes(),
            ],
            &crate::ID,
        );

        let seed_bindings = instructions.seed.to_le_bytes();
        let bump_bindings = [bump];
        let escrow_seeds = [
            Seed::from(b"escrow"),
            Seed::from(accounts.maker.key().as_ref()),
            Seed::from(&seed_bindings),
            Seed::from(&bump_bindings),
        ];

        ProgramAccount::init::<Escrow>(
            accounts.maker,
            accounts.escrow,
            &escrow_seeds,
            Escrow::LEN,
        )?;

        AssociatedTokenAccount::init(
            accounts.vault,
            accounts.mint_a,
            accounts.maker,
            accounts.vault,
            accounts.system_program,
            accounts.token_program,
        )?;

        Ok(Self {
            accounts,
            instructions,
            bump,
        })
    }
}

impl<'a> Make<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        let mut data = self.accounts.escrow.try_borrow_mut_data()?;
        let escrow = Escrow::load_mut(data.as_mut())?;

        escrow.set_inner(
            self.instructions.seed,
            *self.accounts.maker.key(),
            *self.accounts.mint_a.key(),
            *self.accounts.mint_b.key(),
            self.instructions.recieve,
            [self.bump],
        );

        Transfer {
            from: self.accounts.maker_ata_a,
            amount: self.instructions.amount,
            authority: self.accounts.maker,
            to: self.accounts.vault,
        }
        .invoke()?;
        Ok(())
    }
}
