use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, close_account, transfer_checked, CloseAccount, TransferChecked}};

use crate::Escrow;

/*
    accounts required:
        - maker
        - escrow
        - escrow_vault
        - maker_ata_a
        - mint_a
        - two programs
*/

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct RevokeOffer<'info> {

    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = maker,
        associated_token::token_program = token_program
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = mint_a,
        has_one = maker,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub escrow_vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl <'info> RevokeOffer <'info> {

    pub fn revoke_offer(&mut self, _seed:u64) -> Result<()>{

        // Step:1 transfer the tokens back to maker
        let signer_seeds: [&[&[u8]]; 1] = [&[
            b"escrow",
            self.maker.to_account_info().key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        let program = self.token_program.to_account_info();

        let accounts = TransferChecked{
            from: self.escrow_vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.maker_ata_a.to_account_info(),
            authority:self.escrow.to_account_info()
        };

        let ctx = CpiContext::new_with_signer(program, accounts, &signer_seeds);
        transfer_checked(ctx, self.escrow_vault.amount, self.mint_a.decimals)?;

        // Step:2 close the account
        let accounts = CloseAccount{
            account: self.escrow_vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        let ctx = CpiContext::new_with_signer(self.token_program.to_account_info(), accounts, &signer_seeds);
        close_account(ctx)?;

        Ok(())
    }
}