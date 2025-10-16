use anchor_lang::{prelude::*};
use anchor_spl::{associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, close_account, transfer_checked, CloseAccount, TransferChecked}};

use crate::Escrow;
use crate::EscrowError;

/*
    accounts required:
        -taker
        -mint_a
        -mint_b
        -maker_ata_b
        -taker_ata_a
        -taker_ata_b
        -escrow
        -escrow_vault
        -other three programs
*/

#[derive(Accounts)]
#[instruction(seed:u64)]
pub struct TakeOffer<'info>{

    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

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
        associated_token::authority = maker,
        associated_token::token_program = token_program,
        associated_token::mint = mint_b
    )]
    pub maker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
        associated_token::mint = mint_a
    )]
    pub taker_ata_a: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::authority = taker,
        associated_token::token_program = token_program,
        associated_token::mint = mint_b
    )]
    pub taker_ata_b: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        close = maker,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        seeds = [b"escrow", maker.key().as_ref(), seed.to_le_bytes().as_ref()],
        bump = escrow.bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        associated_token::authority = escrow,
        associated_token::mint = mint_a,
        associated_token::token_program = token_program
    )]
    pub escrow_vault: InterfaceAccount<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>
}

impl <'info> TakeOffer<'info> {

    pub fn take_offer(&mut self, _seed: u64) -> Result<()>{

        // taker_ata_b should have mint_b tokens, and greater than or equal to the amount asked by receiver 
        require!(self.taker_ata_b.amount > 0, EscrowError::AmountError);
        require!(self.taker_ata_b.amount >= self.escrow.receive,EscrowError::TokenAmountError);

        // step:1 transfer tokens from taker_ata_b to maker_ata_b
        let program = self.token_program.to_account_info();

        let accounts = TransferChecked{
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info()
        };

        let ctx = CpiContext::new(program, accounts);
        transfer_checked(ctx, self.escrow.receive, self.mint_b.decimals)?;

        // step:2 transfer tokens escrow_vault to taker_ata_a
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
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(program, accounts, &signer_seeds);

        transfer_checked(ctx, self.escrow_vault.amount, self.mint_a.decimals)?;

        // step:3 close the vault
        let program = self.token_program.to_account_info();
        let accounts = CloseAccount{
            account: self.escrow_vault.to_account_info(),
            destination: self.maker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let ctx = CpiContext::new_with_signer(program, accounts, &signer_seeds);
        close_account(ctx)?;

        Ok(())
    }
}