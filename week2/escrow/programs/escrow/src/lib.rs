pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use error::*;
pub use state::*;

declare_id!("CazUFZkHBppLVEJic66t4S5wybfawxgqtS4p2PdMLA3q");

#[program]
pub mod escrow {
    use super::*;

    pub fn make_offer(ctx: Context<MakeOffer>, seed:u64, deposit_amount:u64, receive:u64)-> Result<()>{

        require!(deposit_amount > 0, EscrowError::InvalidAmount);
        require!(receive > 0, EscrowError::InvalidAmount);

        ctx.accounts.offer(seed, deposit_amount, receive, &ctx.bumps)?;
        Ok(())
    }

    pub fn revoke_offer(ctx:Context<RevokeOffer>, seed:u64) -> Result<()>{
        ctx.accounts.revoke_offer(seed)?;
        Ok(())
    }

    pub fn take_offer(ctx:Context<TakeOffer>, seed:u64) -> Result<()> {
        ctx.accounts.take_offer(seed)?;
        Ok(())
    }
}
