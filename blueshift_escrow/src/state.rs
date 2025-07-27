use pinocchio::{program_error::ProgramError, pubkey::Pubkey};

#[repr(C)]
pub struct Escrow {
    pub seed: u64,      // to derive the pda account
    pub maker: Pubkey,  // the one who makes an escrow
    pub mint_a: Pubkey, // token which is deposited
    pub mint_b: Pubkey, // token requested by maker
    pub recieve: u64,   // amount of token that maker wants
    pub bump: [u8; 1],  // pda seed bump
}

impl Escrow {
    // it provide the size of the account which we are storing on-chain
    pub const LEN: usize =
        size_of::<u8>() + size_of::<Pubkey>() * 3 + size_of::<u64>() + size_of::<[u8; 1]>();

    pub fn load_mut(bytes: &mut [u8]) -> Result<&mut Self, ProgramError> {
        if bytes.len() != Escrow::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(unsafe { &mut *core::mem::transmute::<*mut u8, *mut Self>(bytes.as_mut_ptr()) })
    }

    pub fn load(bytes: &mut [u8]) -> Result<&Self, ProgramError> {
        if bytes.len() != Escrow::LEN {
            return Err(ProgramError::InvalidAccountData);
        }
        Ok(unsafe { &*core::mem::transmute::<*const u8, *const Self>(bytes.as_ptr()) })
    }

    // inline is used to copy the function directly into the calling function rather than generating
    // the multiple functions
    #[inline(always)]
    pub fn set_inner(
        &mut self,
        seed: u64,
        maker: Pubkey,
        mint_a: Pubkey,
        mint_b: Pubkey,
        recieve: u64,
        bump: [u8; 1],
    ) {
        self.seed = seed;
        self.maker = maker;
        self.mint_a = mint_a;
        self.mint_b = mint_b;
        self.recieve = recieve;
        self.bump = bump
    }
}
