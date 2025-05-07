use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

use crate::constants::{GLOBAL_AUTHORITY_SEED, PLAYER_POOL_SEED, VAULT_AUTHORITY_SEED};

#[account]
#[derive(Default)]
pub struct GlobalPool {
    pub super_admin: Pubkey,         // 32
    pub operation_authority: Pubkey, // 32
    pub finance_authority: Pubkey,   // 32
    pub update_authority: Pubkey,    // 32
    pub rtp: u64,                    // 8
    pub max_win_amount: u64,         // 8
    pub min_bet_amount: u64,         // 8
    pub min_num: u8,                // 1
    pub max_num: u8,                // 1
}

impl GlobalPool {
    pub const DATA_SIZE: usize = 32 + 32 + 32 + 32 + 8 + 8 + 8 + 1 + 1; //  154
}

#[derive(AnchorSerialize, AnchorDeserialize, Default, Clone, PartialEq)]
pub enum GameStatus {
    #[default]
    Active, // 1
    Win,        // 1
    Lose,       // 1
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        space = 8 + GlobalPool::DATA_SIZE,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
        payer = admin
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub casino_vault: AccountInfo<'info>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitPlayGameParams {
    pub target_number: u8,
    pub is_under: bool,
    pub bet_amount: u64,
    pub game_session_id: u64
}

#[derive(Accounts)]
#[instruction(
    params: InitPlayGameParams
)]
pub struct PlayGame<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        address = global_authority.operation_authority
    )]
    pub operator: Signer<'info>,

    #[account(
        init,
        space = 8 + PlayerPool::DATA_SIZE,
        seeds = [&owner.key().as_ref(), PLAYER_POOL_SEED.as_bytes(), &params.game_session_id.to_be_bytes()[..]],
        bump,
        payer = operator
    )]
    pub player_pool: Account<'info, PlayerPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub casino_vault: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [&owner.key().as_ref(), VAULT_AUTHORITY_SEED.as_bytes(), &params.game_session_id.to_be_bytes()[..]],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub game_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct SetResultParams {
    is_win: bool,
    game_session_id: u64
}

#[derive(Accounts)]
#[instruction(
    params: SetResultParams
)]
pub struct SetResult<'info> {
    #[account(
        mut,
        address = global_authority.operation_authority
    )]
    pub operator: Signer<'info>,

    /// CHECK:
    #[account(mut)]
    pub owner: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [&owner.key().as_ref(), PLAYER_POOL_SEED.as_bytes(), &params.game_session_id.to_be_bytes()[..]],
        bump
    )]
    pub player_pool: Account<'info, PlayerPool>,

    #[account(
        mut,
        seeds = [&owner.key().as_ref(), VAULT_AUTHORITY_SEED.as_bytes(), &params.game_session_id.to_be_bytes()[..]],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub game_vault: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub casino_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        address = global_authority.finance_authority
    )]
    pub financial_admin: Signer<'info>,

    /// CHECK:
    #[account(mut)]
    pub recipient: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub casino_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetGlobalPool<'info> {
    #[account(address = global_pool.update_authority)]
    pub admin: Signer<'info>, // ADMIN_C must be this signer

    #[account(mut)]
    pub global_pool: Account<'info, GlobalPool>,
}

#[derive(Accounts)]
pub struct SetAuthority<'info> {
    #[account(address = global_pool.super_admin)]
    pub admin: Signer<'info>,

    #[account(mut)]
    pub global_pool: Account<'info, GlobalPool>,
}

#[account]
#[derive(Default)]
pub struct PlayerPool {
    pub bet: u64,           // 8
    pub status: GameStatus, // 3
    pub is_under: bool,     // 1
    pub target_num: u8,     // 1
    pub player: Pubkey,     // 32
}

impl PlayerPool {
    pub const DATA_SIZE: usize = 8 + 3 + 1 + 1 + 32; // 45
}
