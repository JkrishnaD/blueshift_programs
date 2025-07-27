use pinocchio::{
    account_info::AccountInfo, program_error::ProgramError, pubkey::find_program_address,
};

// defining the trait for checking the accounts
pub trait AccountCheck {
    fn check(account: &AccountInfo) -> Result<(), ProgramError>;
}

pub struct SignerAccount;

impl AccountCheck for SignerAccount {
    // function to verify the signer
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_signer() {
            return Err(ProgramError::InvalidAccountOwner);
        };
        Ok(())
    }
}

pub struct SystemAccount;

impl AccountCheck for SystemAccount {
    // function to verify the account is system or not
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if account.is_owned_by(&pinocchio_system::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        };
        Ok(())
    }
}

// TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb is the token program id
pub const TOKEN_2022_PROGRAM_ID: [u8; 32] = [
    0x06, 0xdd, 0xf6, 0xe1, 0xee, 0x75, 0x8f, 0xde, 0x18, 0x42, 0x5d, 0xbc, 0xe4, 0x6c, 0xcd, 0xda,
    0xb6, 0x1a, 0xfc, 0x4d, 0x83, 0xb9, 0x0d, 0x27, 0xfe, 0xbd, 0xf9, 0x28, 0xd8, 0xa1, 0x8b, 0xfc,
];

// discriminators for the accounts for the difference between the legacy and the token22 programs
const TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET: usize = 165;
pub const TOKEN_2022_MINT_DISCRIMINATOR: u8 = 0x01;
pub const TOKEN_2022_TOKEN_ACCOUNT_DISCRIMINATOR: u8 = 0x02;

pub struct MintInterface;

// this check is to verify the account is the mint account or not
impl AccountCheck for MintInterface {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&TOKEN_2022_PROGRAM_ID) {
            if !account.is_owned_by(&pinocchio_token::ID) {
                return Err(ProgramError::InvalidAccountOwner.into());
            } else {
                if account.data_len().ne(&pinocchio_token::state::Mint::LEN) {
                    return Err(ProgramError::InvalidAccountData.into());
                }
            }
        } else {
            let data = account.try_borrow_data()?;

            if data.len().ne(&pinocchio_token::state::Mint::LEN) {
                if data[TOKEN_2022_ACCOUNT_DISCRIMINATOR_OFFSET].ne(&TOKEN_2022_MINT_DISCRIMINATOR)
                {
                    return Err(ProgramError::InvalidAccountData.into());
                }
            }
        }
        Ok(())
    }
}

pub struct TokenAccount;

// this is to check the account is it a token account or not
impl AccountCheck for TokenAccount {
    fn check(account: &AccountInfo) -> Result<(), ProgramError> {
        if !account.is_owned_by(&pinocchio_token::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        };

        if !account
            .data_len()
            .ne(&pinocchio_token::state::TokenAccount::LEN)
        {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(())
    }
}

pub struct AssociatedTokenAccount;

pub trait AssociatedTokenCheck {
    fn check(
        account: &AccountInfo,
        authority: &AccountInfo,
        mint: &AccountInfo,
    ) -> Result<(), ProgramError>;
}
// this is to check whether is the token account is associated with a user or not
impl AssociatedTokenCheck for AssociatedTokenAccount {
    fn check(
        account: &AccountInfo,
        authority: &AccountInfo,
        mint: &AccountInfo,
    ) -> Result<(), ProgramError> {
        TokenAccount::check(account);

        if find_program_address(
            &[authority.key(), &pinocchio_token::ID, mint.key()],
            &pinocchio_associated_token_account::ID,
        )
        .0
        .ne(account.key())
        {
            return Err(ProgramError::InvalidAccountOwner);
        }
        Ok(())
    }
}
