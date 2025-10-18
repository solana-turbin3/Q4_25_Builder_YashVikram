use anchor_lang::{prelude::*, system_program::{transfer, Transfer}};
// since we are transferring native sol tokens, so we will import transfer and Transfer from system_program
// alternatively, if we were transferring spl tokens, then we would've used 'token' or 'token_interface'

declare_id!("6xGwnMZ2BYk4bqvttmX1WRXdiR3vdGBW67PgjTi5sqDA");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.initialize(&ctx.bumps)?;
        Ok(())
    }

    pub fn deposit(ctx:Context<Deposit>, amount:u64) -> Result<()>{
        ctx.accounts.deposit(amount)?;
        Ok(())
    }

    pub fn withdraw(ctx:Context<Withdraw>, amount:u64) -> Result<()>{

        ctx.accounts.withdraw(amount)?;
        Ok(())
    }

    pub fn close_vault(ctx:Context<CloseVault>) -> Result<()>{
        ctx.accounts.close_vault()?;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize <'info>{

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = VaultState::DISCRIMINATOR.len() + VaultState::INIT_SPACE,
        seeds = [b"state", signer.key().as_ref()],
        bump
    )]
    pub vault_state: Account<'info, VaultState>,

    // we are not initializing the vault here. the primary reason is, we don't need to. the vault here is a
    // SYSTEM ACCOUNT, and not a data account (represented by Account<'info,...>). A basic system account is 
    // capable of holding sol tokens. In the initialize instruction, when we are transferring the rent_exempt
    // to the vault, if the vault doesn't exist, IT IS CREATED AUTOMATICALLY, OWNED BY THE SYSTEM.
    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

impl <'info> Initialize <'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps)-> Result<()>{

        // step:1 initialize the vault state
        self.vault_state.state_bump = bumps.vault_state;
        self.vault_state.vault_bump = bumps.vault;

        // step:2 obtain the minimum rent exempt balance for vault
        let rent_exempt = Rent::get()?.minimum_balance(0);

        // step:3 transfer the minimum rent exempt balance to vault
        let program = self.system_program.to_account_info();
        let accounts = Transfer{
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info()
        };
        let ctx = CpiContext::new(program, accounts);

        transfer(ctx, rent_exempt)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Deposit<'info> {

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,
    
    pub system_program: Program<'info, System>
}

impl <'info> Deposit <'info> {

    pub fn deposit(&mut self, amount:u64) -> Result<()>{

        let program = self.system_program.to_account_info();
        let accounts = Transfer{
            from: self.signer.to_account_info(),
            to: self.vault.to_account_info()
        };

        let ctx = CpiContext::new(program, accounts);
        transfer(ctx, amount)?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Withdraw<'info> {

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

impl<'info> Withdraw <'info> {

    pub fn withdraw(&mut self, amount:u64)-> Result<()>{

        // Check if vault has enough balance after maintaining rent exemption
        let rent_exempt = Rent::get()?.minimum_balance(0);
        let vault_balance = **self.vault.to_account_info().lamports.borrow();

        require!(vault_balance >= amount+rent_exempt,VaultError::InsufficientFunds);

        let program = self.system_program.to_account_info();
        let accounts = Transfer {
            from:self.vault.to_account_info(),
            to:self.signer.to_account_info(),
        };
        // since we are using seeds, we need to bind the vault state key to a variable
        let binding = self.vault_state.key();
        let signer_seeds = &[
            b"vault",
            binding.as_ref(),
            &[self.vault_state.vault_bump]
        ];

        let signer_seeds = &[&signer_seeds[..]];

        let ctx = CpiContext::new_with_signer(program, accounts, signer_seeds);

        transfer(ctx, amount)?;
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct CloseVault <'info> {

    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    #[account(
        mut,
        close = signer,
        seeds = [b"state", signer.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    pub system_program: Program<'info, System>,
}

impl <'info> CloseVault <'info> {
    
    pub fn close_vault(&mut self) -> Result<()> {

        let lamports = self.vault.to_account_info().lamports();

        let program = self.system_program.to_account_info();
        let accounts = Transfer{
            from: self.vault.to_account_info(),
            to: self.signer.to_account_info(),
        };
        let binding = self.vault_state.key();
        let seeds = &[
            b"vault",
            binding.as_ref(),
            &[self.vault_state.vault_bump]
        ];
        let signer_seeds = &[&seeds[..]];

        let ctx = CpiContext::new_with_signer(program, accounts, signer_seeds);
        transfer(ctx, lamports)?;

        Ok(())
    }
}

#[account]
#[derive(InitSpace)]
pub struct VaultState{

    pub state_bump: u8,
    pub vault_bump: u8
}

#[error_code]
pub enum VaultError{
    #[msg("Insufficient funds for withdrawal")]
    InsufficientFunds,
}