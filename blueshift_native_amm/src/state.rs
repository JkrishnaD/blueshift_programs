use pinocchio::{
    account_info::{AccountInfo, Ref, RefMut},
    program_error::ProgramError,
    pubkey::Pubkey,
};

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
    pub fn load_mut(account_info: &AccountInfo) -> Result<RefMut<Self>, ProgramError> {
        if account_info.data_len() != Self::LEN {
            return Err(ProgramError::InvalidAccountData);
        }

        if account_info.owner().ne(&crate::ID) {
            return Err(ProgramError::InvalidAccountOwner);
        }

        Ok(RefMut::map(
            account_info.try_borrow_mut_data()?,
            |data| unsafe { Self::from_bytes_unchecked_mut(data) },
        ))
    }

    #[inline(always)]
    pub unsafe fn from_bytes_unchecked_mut(bytes: &mut [u8]) -> &mut Self {
        &mut *(bytes.as_mut_ptr() as *mut Config)
    }

    // getter methods to access the fields in the
    #[inline(always)]
    pub fn state(&self) -> u8 {
        self.state
    }

    #[inline(always)]
    pub fn seed(&self) -> u64 {
        u64::from_le_bytes(self.seed)
    }

    #[inline(always)]
    pub fn authority(&self) -> &Pubkey {
        &self.authority
    }

    #[inline(always)]
    pub fn mint_x(&self) -> &Pubkey {
        &self.mint_x
    }

    #[inline(always)]
    pub fn mint_y(&self) -> &Pubkey {
        &self.mint_y
    }

    #[inline(always)]
    pub fn fee(&self) -> u16 {
        u16::from_le_bytes(self.fee)
    }

    #[inline(always)]
    pub fn bump(&self) -> [u8; 1] {
        self.bump
    }

    #[inline(always)]
    pub fn set_state(&mut self, state: u8) -> Result<(), ProgramError> {
        if state.ge(&(AmmState::WithdrawOnly as u8)) {
            return Err(ProgramError::InvalidAccountData);
        }
        self.state = state as u8;
        Ok(())
    }

    #[inline(always)]
    pub fn set_seed(&mut self, seed: u64) -> Result<(), ProgramError> {
        if seed == 0 {
            return Err(ProgramError::InvalidAccountData);
        }
        self.seed = seed.to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn set_fee(&mut self, fee: u16) -> Result<(), ProgramError> {
        if fee.ge(&10_000) {
            return Err(ProgramError::InvalidAccountData);
        }
        self.fee = fee.to_le_bytes();
        Ok(())
    }

    #[inline(always)]
    pub fn set_authority(&mut self, authority: Pubkey) -> Result<(), ProgramError> {
        if authority == Pubkey::default() {
            return Err(ProgramError::InvalidAccountData);
        }
        self.authority = authority;
        Ok(())
    }

    #[inline(always)]
    pub fn set_mint_x(&mut self, mint_x: Pubkey) -> Result<(), ProgramError> {
        if mint_x == Pubkey::default() {
            return Err(ProgramError::InvalidAccountData);
        }
        self.mint_x = mint_x;
        Ok(())
    }

    #[inline(always)]
    pub fn set_mint_y(&mut self, mint_y: Pubkey) -> Result<(), ProgramError> {
        if mint_y == Pubkey::default() {
            return Err(ProgramError::InvalidAccountData);
        }
        self.mint_y = mint_y;
        Ok(())
    }

    #[inline(always)]
    pub fn set_bump(&mut self, bump: u8) -> Result<(), ProgramError> {
        if bump == 0 {
            return Err(ProgramError::InvalidAccountData);
        }
        self.bump = bump.to_le_bytes();
        Ok(())
    }

    pub fn set_inner(
        &mut self,
        seed: u64,
        authority: Pubkey,
        mint_x: Pubkey,
        mint_y: Pubkey,
        fee: u16,
        config_bump: [u8; 1],
    ) -> Result<(), ProgramError> {
        self.set_state(AmmState::Initialized as u8)?;
        self.set_seed(seed)?;
        self.set_authority(authority)?;
        self.set_mint_x(mint_x)?;
        self.set_mint_y(mint_y)?;
        self.set_fee(fee)?;
        self.set_bump(config_bump[0])?;
        Ok(())
    }

    pub fn has_authority(&self) -> Option<Pubkey> {
        if self.authority != Pubkey::default() {
            Some(self.authority)
        } else {
            None
        }
    }
}
