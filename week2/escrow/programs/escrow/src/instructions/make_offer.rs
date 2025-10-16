use anchor_lang::prelude::*;
use anchor_spl::{self, associated_token::AssociatedToken, token::{transfer_checked, TransferChecked}, token_interface::{Mint, TokenAccount, TokenInterface}};

use crate::{Escrow, ANCHOR_DISCRIMINATOR};

/*
    accounts required:
    - maker
    - mint_a
    - mint_b
    - maker_ata_a
    - escrow
    - escrow_vault
    - other three programs
*/

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct MakeOffer<'info> {

    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_a: InterfaceAccount<'info, Mint>,

    #[account(
        mint::token_program = token_program
    )]
    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::token_program = token_program,
        associated_token::authority = maker,
    )]
    pub maker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        init,
        payer = maker,
        space = ANCHOR_DISCRIMINATOR + Escrow::INIT_SPACE,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        init,
        payer = maker,
        associated_token::mint = mint_a,
        associated_token::authority = escrow,
        associated_token::token_program = token_program
    )]
    pub escrow_vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl <'info> MakeOffer<'info> {

   pub fn offer(&mut self, seed: u64, deposit_amount: u64, receive: u64, bump: &MakeOfferBumps)-> Result<()>{

        // step:1 initialize the escrow
        self.escrow.set_inner(Escrow { 
            seed, 
            maker: self.maker.key(), 
            mint_a: self.mint_a.key(), 
            mint_b: self.mint_b.key(), 
            receive, 
            bump: bump.escrow
        });

        // step:2 transfer mint_a to escrow_vault
        let transfer_accounts = TransferChecked {
            from: self.maker_ata_a.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.escrow_vault.to_account_info(),
            authority: self.maker.to_account_info()
        };

        let ctx = CpiContext::new(
            self.token_program.to_account_info(), 
            transfer_accounts
        );

        transfer_checked(ctx, deposit_amount, self.mint_a.decimals)?;

        Ok(())

   }
}
