use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

use anchor_spl::associated_token::{self, AssociatedToken};

use crate::state::*;

#[derive(Accounts)]
#[instruction()]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = 8 + 1 + 1 + 32 + 1 +1+32 ,
        seeds = [b"master_account"],
        bump
    )]
    pub master_account: Account<'info, MasterAccount>,
    #[account(mut)]
    pub admin: Signer<'info>, // This is the payer
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct SetFees<'info> {
    //only admin struct
    #[account(mut, has_one = admin, seeds = [b"master_account"] ,bump)]
    pub master_account: Account<'info, MasterAccount>,
    #[account(mut)]
    pub admin: Signer<'info>, // This is the payer
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct SetCollector<'info> {
    //only admin struct
    #[account(mut, has_one = admin, seeds = [b"master_account"] ,bump)]
    pub master_account: Account<'info, MasterAccount>,
    #[account(mut)]
    pub admin: Signer<'info>, // This is the payer
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CollectFees<'info> {
    //only admin struct
    #[account(mut, has_one = fee_collector, seeds = [b"master_account"] ,bump)]
    pub master_account: Account<'info, MasterAccount>,
    #[account(mut)]
    pub fee_collector: Signer<'info>, // This is the payer
    #[account(mut,
        associated_token::mint = token_mint,
        associated_token::authority = fee_collector,
    )]
    pub fee_collector_token_ata: Account<'info, TokenAccount>, // User's ATA for input tokens
    #[account(mut,
        associated_token::mint = token_mint,
        associated_token::authority = master_account,
    )]
    pub master_account_token_ata: Account<'info, TokenAccount>, // User's ATA for input tokens
    pub token_mint: Account<'info, Mint>,
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct SetAdmin<'info> {
    //only admin struct
    #[account(mut, has_one = admin, seeds = [b"master_account"] ,bump)]
    pub master_account: Account<'info, MasterAccount>,
    #[account(mut)]
    pub admin: Signer<'info>, // This is the payer
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction()]
pub struct CalculateInvariant<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
}

#[derive(Accounts)]
#[instruction()]
pub struct InitializePool<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    pub input_token_mint: Account<'info, Mint>,
    pub output_token_mint: Account<'info, Mint>,
    #[account(mut, seeds = [b"master_account"] ,bump)]
    pub master_account: Account<'info, MasterAccount>,
    #[account(init_if_needed,
        payer = user,
        associated_token::mint = input_token_mint,
        associated_token::authority = master_account,
    )]
    pub master_account_input_fee_ata: Account<'info, TokenAccount>, // User's ATA for input tokens
    #[account(init_if_needed,
        payer = user,
        associated_token::mint = output_token_mint,
        associated_token::authority = master_account,
    )]
    pub master_account_output_fee_ata: Account<'info, TokenAccount>, // User's ATA for input tokens
    #[account(
        init,
        payer = user,
        space = 8 + (1 + 3 * 32 + 2 * 8 + 2 * 16 + 2 * 1 + 2 * 1 + 8 + 1  + 1 + 1 + 32 + 32 + 2 +1 ), // Adjusted space calculation
        seeds = [b"pool_account", user.key().as_ref(), input_token_mint.key().as_ref()],
        bump,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    #[account(
        init,
        payer = user,
        seeds = [b"bp_token_mint", pool_account.to_account_info().key.as_ref()], // Use appropriate seeds
        mint::decimals = 8,
        mint::authority = pool_account,
        bump,
    )]
    pub bp_token_mint: Account<'info, Mint>, // This account is being created in the transaction
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, token::Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct InitializePoolParams {
    pub account_type: u8,
    pub start_timestamp: i64, // timestamp in seconds
    pub end_timestamp: i64,
    pub start_weights: [u8; 2],
    pub end_weights: [u8; 2],
    pub is_sol: bool,
    pub is_vesting: bool,
    pub is_buy_only: bool,
}

#[derive(Accounts)]
#[instruction()]
pub struct PausePool<'info> {
    //only owner struct
    pub input_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        has_one = owner,
        seeds = [b"pool_account", owner.key().as_ref(), input_token_mint.key().as_ref()],
        bump,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    #[account(mut)]
    pub owner: Signer<'info>, // This is the payer
}

#[derive(Accounts)]
#[instruction()]
pub struct UnPausePool<'info> {
    //only owner struct
    pub input_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"pool_account", user.key().as_ref(), input_token_mint.key().as_ref()],
        bump,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    #[account(mut)]
    pub user: Signer<'info>, // This is the payer
}

#[derive(Accounts)]
pub struct InitializePoolFunds<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    /// CHECK: Just used to derive account pda
    pub input_token_mint: Account<'info, Mint>,
    pub output_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"bp_token_mint", pool_account.to_account_info().key.as_ref()],
        bump,
    )]
    pub bp_token_mint: Account<'info, Mint>,
    #[account(
        mut,
        seeds = [b"pool_account", user.key().as_ref(), input_token_mint.key().as_ref()],
        bump,
    )]
    pub pool_account: Account<'info, PoolAccount>,
    #[account(mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = user,
    )]
    pub user_input_ata: Account<'info, TokenAccount>, // User's ATA for input tokens
    #[account(mut,
        associated_token::mint = output_token_mint,
        associated_token::authority = user,
    )]
    pub user_output_ata: Account<'info, TokenAccount>, // User's ATA for output tokens
    #[account(init_if_needed,
        payer = user,
        associated_token::mint = bp_token_mint,
        associated_token::authority = user)]
    pub user_bp_ata: Account<'info, TokenAccount>, // Pool's ATA for output tokens
    #[account(init_if_needed,
        payer = user,
        associated_token::mint = input_token_mint,
        associated_token::authority = pool_account
    )]
    pub pool_input_ata: Account<'info, TokenAccount>, // Pool's ATA for input tokens
    #[account(init_if_needed,
        payer = user,
        associated_token::mint = output_token_mint,
        associated_token::authority = pool_account)]
    pub pool_output_ata: Account<'info, TokenAccount>, // Pool's ATA for output tokens
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Clone, Debug, AnchorSerialize, AnchorDeserialize)]
pub struct InitializePoolFundsParams {
    pub normalized_weights: [u8; 2],
    pub balances: [u64; 2],
}

#[derive(Accounts)]
pub struct JoinPool<'info> {
    #[account(mut)]
    pub user: Signer<'info>, // user is owner
    pub output_token_mint: Account<'info, Mint>,
    #[account(mut,
        associated_token::mint = output_token_mint,
        associated_token::authority = user)]
    pub user_output_ata: Account<'info, TokenAccount>, // User's ATA for input tokens
    #[account(mut,
        associated_token::mint = bp_token_mint,
        associated_token::authority = user)]
    pub user_bp_ata: Account<'info, TokenAccount>, // User's ATA for BP tokens
    #[account(mut)]
    pub input_token_mint: Account<'info, Mint>, // Mint of the input Token (XYZ) used to dervie the pool account
    #[account(
        mut,
        seeds = [b"pool_account", user.key().as_ref(), input_token_mint.key().as_ref()],
        bump,
    )]
    pub pool_account: Account<'info, PoolAccount>, // The pool account
    #[account(mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = pool_account)]
    pub pool_input_ata: Account<'info, TokenAccount>, // Pool's ATA for ouput tokens
    #[account(mut,
        associated_token::mint = output_token_mint,
        associated_token::authority = pool_account)]
    pub pool_output_ata: Account<'info, TokenAccount>, // Pool's ATA for ouput tokens
    #[account(mut,
        associated_token::mint = bp_token_mint,
        associated_token::authority = pool_account)]
    pub pool_bp_ata: Account<'info, TokenAccount>, // Pool's ATA for BP tokens, holding the total supply of BP tokens
    #[account(
        mut,
        seeds = [b"bp_token_mint", pool_account.to_account_info().key.as_ref()],
        bump,
    )]
    pub bp_token_mint: Account<'info, Mint>, // The BP token mint (might not be needed for transfer)
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Swap<'info> {
    /// CHECK: Just used to derive account pda
    pub owner: UncheckedAccount<'info>,
    pub output_token_mint: Account<'info, Mint>,
    #[account(mut, seeds = [b"master_account"] ,bump)]
    pub master_account: Account<'info, MasterAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut, 
        associated_token::mint = output_token_mint,
        associated_token::authority = user)]
    pub user_output_ata: Account<'info, TokenAccount>, // User's ATA for input tokens
    #[account(mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = user)]
    pub user_input_ata: Account<'info, TokenAccount>, // User's ATA for input tokens
    #[account(mut,
        associated_token::mint = bp_token_mint,
        associated_token::authority = user)]
    pub user_bp_ata: Account<'info, TokenAccount>, // User's ATA for BP tokens
    #[account(mut)]
    pub input_token_mint: Account<'info, Mint>, // Mint of the input Token (XYZ) used to dervie the pool account
    #[account(
        mut,
        seeds = [b"pool_account", owner.key().as_ref(), input_token_mint.key().as_ref()],
        bump,
    )]
    pub pool_account: Account<'info, PoolAccount>, // The pool account
    #[account(mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = pool_account)]
    pub pool_input_ata: Account<'info, TokenAccount>, // Pool's ATA for ouput tokens
    #[account(mut,
        associated_token::mint = output_token_mint,
        associated_token::authority = pool_account)]
    pub pool_output_ata: Account<'info, TokenAccount>, // Pool's ATA for ouput tokens
    #[account(mut,
        associated_token::mint = bp_token_mint,
        associated_token::authority = pool_account)]
    pub pool_bp_ata: Account<'info, TokenAccount>, // Pool's ATA for BP tokens, holding the total supply of BP tokens
    #[account(mut,
        associated_token::mint = input_token_mint,
        associated_token::authority = master_account)]
    pub fee_collector_input_ata: Account<'info, TokenAccount>, // Pool's ATA for BP tokens, holding the total supply of BP tokens
    pub bp_token_mint: Account<'info, Mint>, // The BP token mint (might not be needed for transfer)
    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Redeem<'info> {
    /// CHECK: Just used to derive account pda
    #[account(mut)]
    pub owner: UncheckedAccount<'info>,
    #[account(mut, seeds = [b"master_account"] ,bump)]
    pub master_account: Account<'info, MasterAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut,
    associated_token::mint = input_token_mint,
    associated_token::authority = user)]
    pub user_input_ata: Account<'info, TokenAccount>, // User's ATA for input tokens
    #[account(mut,
    associated_token::mint = output_token_mint,
    associated_token::authority = user)]
    pub user_output_ata: Account<'info, TokenAccount>, // User's ATA for BP tokens

    #[account(mut,
    associated_token::mint = bp_token_mint,
    associated_token::authority = user)]
    pub user_bp_ata: Account<'info, TokenAccount>, // User's ATA for BP tokens
    #[account(mut)]
    pub input_token_mint: Account<'info, Mint>, // Mint of the input Token (XYZ) used to dervie the pool account
    #[account(mut)]
    pub output_token_mint: Account<'info, Mint>, // Mint of the input Token (XYZ) used to dervie the pool account
    #[account(
        mut,
        seeds = [b"pool_account", owner.key().as_ref(), input_token_mint.key().as_ref()],
        bump,
    )]
    pub pool_account: Account<'info, PoolAccount>, // The pool account
    #[account(mut,
    associated_token::mint = input_token_mint,
    associated_token::authority = pool_account)]
    pub pool_input_ata: Account<'info, TokenAccount>, // Pool's ATA for ouput tokens
    #[account(mut,
    associated_token::mint = output_token_mint,
    associated_token::authority = pool_account)]
    pub pool_output_ata: Account<'info, TokenAccount>, // Pool's ATA for ouput tokens
    #[account(mut,
    associated_token::mint = bp_token_mint,
    associated_token::authority = pool_account)]
    pub pool_bp_ata: Account<'info, TokenAccount>, // Pool's ATA for BP tokens, holding the total supply of BP tokens
    #[account(mut)]
    pub bp_token_mint: Account<'info, Mint>, // The BP token mint (might not be needed for transfer)
    #[account(mut,
        associated_token::mint = output_token_mint,
        associated_token::authority = master_account)]
    pub fee_collector_output_ata: Account<'info, TokenAccount>,
    #[account(address = anchor_spl::associated_token::ID)]
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}


