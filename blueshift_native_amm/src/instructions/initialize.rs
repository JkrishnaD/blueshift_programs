use pinocchio::{
    account_info::AccountInfo, instruction::Seed, program_error::ProgramError,
    pubkey::find_program_address, ProgramResult,
};

use crate::{
    instructions::{
        AccountCheck, MintInterface, ProgramAccount, ProgramAccountInit, SignerAccount,
    },
    state::Config,
};

pub struct InitializeConfigAccounts<'a> {
    pub authority: &'a AccountInfo,
    pub config: &'a AccountInfo,

    pub mint_x: &'a AccountInfo,
    pub mint_y: &'a AccountInfo,

    pub vault_x: &'a AccountInfo,
    pub vault_y: &'a AccountInfo,

    pub lp_mint: &'a AccountInfo,

    pub token_program: &'a AccountInfo,
    pub system_program: &'a AccountInfo,
    pub associated_token_program: &'a AccountInfo,
}

impl<'a> TryFrom<&'a [AccountInfo]> for InitializeConfigAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [authority, config, mint_x, mint_y, vault_x, vault_y, lp_mint, token_program, system_program, associated_token_program] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        SignerAccount::check(authority)?;
        MintInterface::check(mint_x)?;
        MintInterface::check(mint_y)?;

        if mint_x.key() == mint_y.key() {
            return Err(ProgramError::InvalidAccountData);
        }

        Ok(Self {
            authority,
            config,
            mint_x,
            mint_y,
            vault_x,
            vault_y,
            lp_mint,
            token_program,
            system_program,
            associated_token_program,
        })
    }
}

pub struct InitializeConfigInstruction {
    pub fee: u16,
}

impl<'a> TryFrom<&'a [u8]> for InitializeConfigInstruction {
    type Error = ProgramError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() < 2 {
            return Err(ProgramError::InvalidAccountData);
        };

        let fee = u16::from_le_bytes([data[0], data[1]]);

        if fee > 1000 {
            return Err(ProgramError::InvalidAccountData);
        };
        Ok(Self { fee })
    }
}

pub struct InitializeConfig<'a> {
    pub accounts: InitializeConfigAccounts<'a>,
    pub instruction: InitializeConfigInstruction,
}

impl<'a> TryFrom<(&'a [AccountInfo], &'a [u8])> for InitializeConfig<'a> {
    type Error = ProgramError;

    fn try_from(value: (&'a [AccountInfo], &'a [u8])) -> Result<Self, Self::Error> {
        let accounts = InitializeConfigAccounts::try_from(value.0)?;
        let instruction = InitializeConfigInstruction::try_from(value.1)?;

        Ok(Self {
            accounts,
            instruction,
        })
    }
}

impl<'a> InitializeConfig<'a> {
    pub const DISCRIMINATOR: &'a u8 = &0;

    pub fn process(&self) -> ProgramResult {
        // signers check
        SignerAccount::check(self.accounts.authority)?;

        if self.accounts.vault_x.key() != self.accounts.token_program.key()
            || self.accounts.vault_y.key() != self.accounts.token_program.key()
        {
            return Err(ProgramError::InvalidAccountData);
        }

        let seeds_slice = &[b"config", self.accounts.authority.key().as_ref()];

        let (config_pda, bump) = find_program_address(seeds_slice, &crate::ID);

        let lp_seed = [
            b"lp_mint",
            self.accounts.mint_x.key().as_ref(),
            self.accounts.mint_y.key().as_ref(),
        ];
        let (_lp_pda, lp_bump) = find_program_address(&lp_seed, &crate::ID);

        if &config_pda != self.accounts.config.key() {
            return Err(ProgramError::InvalidAccountData);
        };

        let config_seeds = [
            Seed::from(b"config"),
            Seed::from(self.accounts.mint_x.key()),
            Seed::from(self.accounts.mint_y.key()),
        ];

        // create the config account
        ProgramAccount::init::<Config>(
            self.accounts.authority,
            self.accounts.config,
            &config_seeds,
            Config::LEN,
        )?;

        // get the config account mutable data
        let mut config_data = Config::load_mut(self.accounts.config)?;

        // set the config account data
        config_data.set_inner(
            *self.accounts.authority.key(),
            *self.accounts.mint_x.key(),
            *self.accounts.mint_y.key(),
            *self.accounts.vault_x.key(),
            *self.accounts.vault_y.key(),
            *self.accounts.lp_mint.key(),
            self.instruction.fee,
            bump,
            lp_bump,
        )?;

        Ok(())
    }
}
