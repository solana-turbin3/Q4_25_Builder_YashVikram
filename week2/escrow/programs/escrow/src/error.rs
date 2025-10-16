use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Taker does not have the required token")]
    AmountError,
    #[msg("Taker does not have sufficient required tokens")]
    TokenAmountError,
    #[msg("Invalid amount")]
    InvalidAmount
}
