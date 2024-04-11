use anchor_lang::prelude::*;
use solana_maths::{Decimal, MathError, U128}; //Precision lib for scaled amount

#[account]
pub struct MasterAccount {
    pub account_type: u8,
    pub version: u8,
    pub admin: Pubkey,
    pub protocol_swap_fee_percentage: u8,
    pub protocol_flat_rate_percentage: u8,
    pub fee_collector: Pubkey,
}

#[account]
pub struct PoolAccount {
    pub account_type: u8,
    pub input_token_mint: Pubkey,
    pub output_token_mint: Pubkey,
    pub bp_token_mint: Pubkey,
    pub start_timestamp: i64,
    pub end_timestamp: i64,
    pub scaling_factors: [u128; 2],
    pub start_weights: [u8; 2],
    pub end_weights: [u8; 2],
    pub invariant: u64,
    pub swap_enabled: bool,
    pub is_initialized: bool,
    pub is_vesting: bool,
    pub is_buy_only: bool,
    pub swap_fee_percentage: u8,
    pub flat_rate_percentage: u8,
    pub fee_collector: Pubkey,
    pub owner: Pubkey,
}
