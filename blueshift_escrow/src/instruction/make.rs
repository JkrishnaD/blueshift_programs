use pinocchio::{account_info::AccountInfo, instruction::Signer, program_error::ProgramError};

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

impl<'a> TryFrom<&'a [AccountInfo]> for MakeAccounts<'a> {
    type Error = ProgramError;

    fn try_from(accounts: &'a [AccountInfo]) -> Result<Self, Self::Error> {
        let [marker, escrow, mint_a, mint_b, maker_ata_a, vault, system_program, token_program, _] =
            accounts
        else {
            return Err(ProgramError::InvalidAccountData);
        };

        SignerAccount::check(maker)?;
        MintInterface::check(mint_a)?;
        MintInterface::check(mint_b)?;
    }
}
