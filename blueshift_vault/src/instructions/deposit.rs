use pinocchio::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::find_program_address,
    ProgramResult,
};
use pinocchio_system::instructions::Transfer;

// custom accounts for deposit
pub struct DepositAccount<'a> {
    pub owner: &'a AccountInfo, // owner who signs
    pub vault: &'a AccountInfo,
}

// doing all the necessary checks for the accounts involved in the deposit instruction
impl<'a> TryFrom<&'a [AccountInfo]> for DepositAccount<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        // checking that the account slice has exactly 3 accounts
        let [owner, vault, _] = accounts else {
            return Err(ProgramError::NotEnoughAccountKeys);
        };

        // verifying the owner is a signer or not
        if !owner.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // verifying the vault is owned by the pinocchio system program
        if !vault.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // making sure that vault has no lamports before deposit
        if vault.lamports().ne(&0) {
            return Err(ProgramError::InvalidAccountData);
        }

        // deriving the vault key using the pda
        let (vault_key, _) = find_program_address(&[b"vault", owner.key()], &crate::ID);

        // checking that vault's key matches the derived vault key
        if vault.key().ne(&vault_key) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        // returning both accounts as a DepositAccount
        Ok(Self { owner, vault })
    }
}

// instruction data which are need to perform by the deposit function
pub struct DepositInstruction {
    pub amount: u64,
}

// deposit instruction validation checks
impl<'a> TryFrom<&'a [u8]> for DepositInstruction {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        // checking if the size of the data matches the size of u64 as 8 bytes
        if data.len() != size_of::<u64>() {
            return Err(ProgramError::InvalidInstructionData);
        }

        // converting the data slice into a u64 value
        // try_into is used to convert a slice of bytes into an array of 8 bytes [u8; 8]
        let amount = u64::from_le_bytes(data.try_into().unwrap());

        // checking if the amount is greater than zero
        if amount.eq(&0) {
            return Err(ProgramError::InvalidInstructionData);
        }

        // returning the DepositInstruction with the amount
        Ok(Self { amount })
    }
}

pub struct Deposit<'a> {
    pub accounts: DepositAccount<'a>,
    pub instructions_data: DepositInstruction,
}

impl<'a> TryFrom<(&'a [u8], &'a [AccountInfo])> for Deposit<'a> {
    type Error = ProgramError;

    fn try_from((data, accounts): (&'a [u8], &'a [AccountInfo])) -> Result<Self, Self::Error> {
        let accounts = DepositAccount::try_from(accounts)?;
        let instructions_data = DepositInstruction::try_from(data)?;
        Ok(Self {
            accounts,
            instructions_data,
        })
    }
}

impl<'a> Deposit<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&mut self) -> ProgramResult {
        (Transfer {
            from: &self.accounts.owner,
            lamports: self.instructions_data.amount,
            to: &self.accounts.vault,
        }).invoke()?;
        Ok(())
    }
}
