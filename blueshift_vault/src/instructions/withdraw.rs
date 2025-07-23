use pinocchio::{
    account_info::AccountInfo,
    instruction::{ Seed, Signer },
    program_error::ProgramError,
    pubkey::find_program_address,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

// accounts involved in the withdraw instructions
pub struct WithdrawAccounts<'a> {
    pub owner: &'a AccountInfo,
    pub vault: &'a AccountInfo,
    pub bump: [u8; 1],
}

// this is for the accounts validation check
impl<'a> TryFrom<&'a [AccountInfo]> for WithdrawAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // checking for the correct number of the accounts 
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::InvalidAccountData);
        };

        // checking if the owner is the signer of the transaction
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // checking if the vault is the system account or not
        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // as vault should have something to withdraw so it shouldn't be empty
        if vault.lamports().eq(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        // as vault is pda we are deriving the key from seeds
        let (vault_key, bump) = find_program_address(&[b"vault", owner.key()], &crate::ID);

        // comparing the vault keys
        if vault.key() != &vault_key {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // returning all the accounts 
        Ok(Self { owner, vault, bump: [bump] })
    }
}

// as withdraw functions doesn't have any inputs there are no instructions
pub struct Withdraw<'a> {
    pub accounts: WithdrawAccounts<'a>, // assigning the withdraw accounts 
}

impl<'a> TryFrom<&'a [AccountInfo]> for Withdraw<'a> {
    type Error = ProgramError;

    // this is for the accounts validation check
    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let accounts = WithdrawAccounts::try_from(accounts)?;

        Ok(Self { accounts })
    }
}

impl<'a> Withdraw<'a> {
    pub const DISCRIMINATOR: &'a u8 = &1;

    pub fn process(&mut self) -> ProgramResult {
        // defining the seeds for the signer
        let seeds = [
            Seed::from(b"vault"),
            Seed::from(self.accounts.owner.key().as_ref()),
            Seed::from(&self.accounts.bump),
        ];

        // creating the signer from the seeds
        let signer = [Signer::from(&seeds)];

        // transferring the lamports from the vault to the owner
        (Transfer {
            from: self.accounts.vault,
            lamports: self.accounts.vault.lamports(),
            to: self.accounts.owner,
        }).invoke_signed(&signer)?;
        Ok(())
    }
}
