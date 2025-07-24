use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::find_program_address,
    sysvars::{ rent::Rent, Sysvar },
    ProgramResult,
};
use pinocchio_system::instructions::{ CreateAccount, Transfer };

pub struct InitializeAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(value: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [owner, vault, _] = value else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        let [vault_key, _] = find_program_address(&[b"vault", owner.key()], &crate::ID);

        if vault.key() != &vault_key {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Self { owner, vault, system_program })
    }
}

pub struct Initialize<'a> {
    pub accounts: InitializeAccounts<'a>,
}

impl<'a> TryFrom<&'a [AccountInfo]> for Initialize {
    type Error = ProgramError;

    fn try_from(value: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = Initialize::try_from(value)?;

        Ok(Self { accounts })
    }
}

impl<'a> Initialize<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        let rent_exempt = Rent::get()?.minimum_balance(self.accounts.vault.data_len());

        let seeds = [
            Seed::from(b"vault"),
            Seed::from(self.accounts.owner.key().as_ref()),
            Seed::from(&self.accounts.bump),
        ];

        // creating the signer from the seeds
        let signer = [Signer::from(&seeds)];

        (CreateAccount {
            from: &self.accounts.owner,
            lamports: rent_exempt,
            owner: &crate::ID,
            space: self.accounts.vault.data_len(),
            to: &self.accounts.vault,
        }).invoke_signed(signers)?;
        Ok(())
    }
}
