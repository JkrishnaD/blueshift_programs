use pinocchio::{
    account_info::{AccountInfo, Ref},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::instructions::initialize;

#[repr(C)]
pub struct Config {
    state: u8,
    seed: [u8; 8],
    authority: Pubkey,
    mint_x: Pubkey,
    mint_y: Pubkey,
    fee: [u8; 2],
    bump: [u8; 1],
}

#[repr(u8)]
pub enum AmmState {
    Uninitialized = 0u8,
    Initialized = 1u8,
    Disabled = 2u8,
    WithdrawOnly = 3u8,
}

impl Config {
    pub const LEN: usize = size_of::<Self>();

    // inline always attribute rather than adding the function call to the cll stack
    // it adds the function code to the call stack which eliminate the overhead function call
    #[inline(always)]
    pub fn load(account_info: &AccountInfo) -> Result<Ref<Self>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Ref::map(account_info.try_borrow_data()?, |data| unsafe {
            Self::from_bytes_unchecked(data)
        }))
    }

    #[inline(always)]
    pub unsafe fn load_unchecked(account_info: &AccountInfo) -> Result<&Self, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(Self::from_bytes_unchecked(
            account_info.borrow_data_unchecked(),
        ))
    }

    #[inline(always)]
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        &*(bytes.as_ptr() as *const Config)
    }

    #[inline(always)]
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes.as_mut_ptr() as *mut Config)
    }

    // getter methods to access the fields in the
    pub fn state(&self) -> u8 {
        self.state
    }

    pub fn seed(&self) -> u64 {
        u64::from_le_bytes(self.seed)
    }
    // state: u8,
    // seed: [u8; 8],
    // authority: Pubkey,
    // mint_x: Pubkey,
    // mint_y: Pubkey,
    // fee: [u8; 2],
    // bump: [u8; 1],

    pub fn authority(&self) -> &Pubkey {
        &self.authority
    }

    pub fn mint_x(&self) -> &Pubkey {
        &self.mint_x
    }

    pub fn mint_y(&self) -> &Pubkey {
        &self.mint_x
    }

    pub fn fee(&self) -> u16 {
        u16::from_le_bytes(self.fee)
    }

    pub fn bump(&self) -> [u8; 1] {
        self.bump
    }
}
