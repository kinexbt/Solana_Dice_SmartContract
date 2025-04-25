use anchor_lang::{prelude::*, AnchorDeserialize};

use solana_program::pubkey::Pubkey;

pub mod account;
pub mod constants;
pub mod error;
pub mod utils;

use account::*;
use constants::*;
use error::*;
use utils::*;

declare_id!("2yGiLmgFhZvHvLYMshTwcAsZ9Ja2wNxxQiSWKhmPmqZe");

#[program]
pub mod dice {
    use super::*;
    use solana_program::native_token::LAMPORTS_PER_SOL;
    pub fn initialize(
        ctx: Context<Initialize>,
        operate_admin: Pubkey,
        financial_admin: Pubkey,
        update_admin: Pubkey,
    ) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;

        // sol_transfer_user(
        //     ctx.accounts.admin.to_account_info().clone(),
        //     ctx.accounts.casino_vault.to_account_info().clone(),
        //     ctx.accounts.system_program.to_account_info().clone(),
        //     ctx.accounts.rent.minimum_balance(0),
        // )?;

        global_authority.super_admin = ctx.accounts.admin.key();
        global_authority.operation_authority = operate_admin.key();
        global_authority.finance_authority = financial_admin.key();
        global_authority.update_authority = update_admin.key();
        global_authority.rtp = RTP;
        global_authority.max_win_amount = MAX_WIN_AMOUNT;
        global_authority.min_bet_amount = MIN_BET_AMOUNT;
        global_authority.min_num = MIN_NUMBER;
        global_authority.max_num = MAX_NUMBER;

        Ok(())
    }
    
    /**
        @disc: Main function to flip coin.
        @param:
            head_or_tail: indicate whether the player bet on head or tail       0: Tail, 1: Head
            bet_amount:    The SOL amount to deposit
    */
    pub fn play_game(ctx: Context<PlayGame>, target_number: u8, is_under: bool, bet_amount: u64) -> Result<()> {
        let player_pool = &mut ctx.accounts.player_pool;
        let player = &ctx.accounts.owner;
        let global_authority = &ctx.accounts.global_authority;

        require!(
            global_authority.min_bet_amount < bet_amount,
            GameError::InvalidBetAmount
        );

        let bet_amount_f64 = bet_amount as f64;
        let rtp_f64 = global_authority.rtp as f64;
        let rtp_ratio = rtp_f64 / 100.0;

        let new_balance;
        if is_under == true {
            let win_chance = target_number as f64 / 100.0;
            let multiplier = (1.0 / win_chance) * rtp_ratio;
            new_balance = bet_amount_f64 * multiplier;
        } else {
            let win_chance = (99 - target_number) as f64 / 100.0;
            let multiplier = (1.0 / win_chance) * rtp_ratio;
            new_balance = bet_amount_f64 * multiplier;
        }

        let net_gain = new_balance - bet_amount_f64;
        let net_gain_u64 = net_gain as u64;
        let max_win_amount_u64 = global_authority.max_win_amount;

        require!(
            net_gain_u64 < max_win_amount_u64,
            GameError::InvalidBetAmountMaxWinAmountViolation
        );

        // require!(
        //     (bet_amount * 2 * (global_authority.rtp / 100) - bet_amount)
        //         < global_authority.max_win_amount,
        //     GameError::InvalidBetAmountMaxWinAmountViolation
        // );

        require!(
            ctx.accounts.owner.to_account_info().lamports() > bet_amount,
            GameError::InsufficientUserBalance
        );

        require!(
            ctx.accounts.casino_vault.to_account_info().lamports() > bet_amount,
            GameError::InsufficientCasinoVault
        );

        // Transfer rent fee for PDA of player pool
        sol_transfer_user(
            ctx.accounts.owner.to_account_info().clone(),
            player_pool.to_account_info().clone(),
            ctx.accounts.system_program.to_account_info().clone(),
            ctx.accounts.rent.minimum_balance(0),
        )?;

        // Transfer bet_amount Sol to this PDA from User Wallet
        sol_transfer_user(
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.game_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            bet_amount,
        )?;

        player_pool.status = GameStatus::Active;
        player_pool.bet = bet_amount;
        player_pool.target_num = target_number;
        player_pool.is_under = is_under;
        player_pool.player = player.key();

        if is_under == true {
            msg!(
                "User's choice is under {}",
                target_number
            );
        } else {
            msg!(
                "User's choice is over {}",
                target_number
            );
        }

        Ok(())
    }

    /**
    The setting result function to determine whether player Win or Lose
    */
    pub fn set_result(ctx: Context<SetResult>, is_win: bool) -> Result<()> {
        let player_pool = &mut ctx.accounts.player_pool;
        let game_bump = ctx.bumps.game_vault;
        let casino_bump = ctx.bumps.casino_vault;
        let global_authority = &ctx.accounts.global_authority;
        let game_vault = &mut ctx.accounts.game_vault;
        let casino_vault = &mut ctx.accounts.casino_vault;
        let vault_balance = game_vault.lamports();

        let bet_amount_f64 = player_pool.bet as f64;
        let rtp_f64 = global_authority.rtp as f64;
        let rtp_ratio = rtp_f64 / 100.0;

        let win_balance;
        if player_pool.is_under == true {
            let win_chance = player_pool.target_num as f64 / 100.0;
            let multiplier = (1.0 / win_chance) * rtp_ratio;
            win_balance = bet_amount_f64 * multiplier;
        } else {
            let win_chance = (99 - player_pool.target_num) as f64 / 100.0;
            let multiplier = (1.0 / win_chance) * rtp_ratio;
            win_balance = bet_amount_f64 * multiplier;
        }

        if is_win == true {
            // Transfer bet_amount Sol to this PDA from casino bank
            sol_transfer_with_signer(
                casino_vault.to_account_info(),
                game_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                &[&[VAULT_AUTHORITY_SEED.as_bytes(), &[casino_bump]]],
                win_balance as u64 - vault_balance,
            )?;

            player_pool.status = GameStatus::Win
        } else {
            sol_transfer_with_signer(
                game_vault.to_account_info(),
                casino_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                &[&[
                    ctx.accounts.owner.key().as_ref(),
                    VAULT_AUTHORITY_SEED.as_bytes(),
                    &[game_bump],
                ]],
                vault_balance,
            )?;

            player_pool.status = GameStatus::Lose;

            // Here, add closePda function
            // **game_vault.to_account_info().try_borrow_mut_lamports()? = 0;
            // **player_pool.to_account_info().try_borrow_mut_lamports()? = 0;
        }

        Ok(())
    }

    /**
        @disc: Admin can withdraw SOL from the PDA
        @param:
            amount: The sol amount to withdraw from this PDA
    */
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let global_authority = &ctx.accounts.global_authority;
        let financial_authority = &ctx.accounts.financial_admin;
        let recipient = &ctx.accounts.recipient;
        let casino_bump = ctx.bumps.casino_vault;
        let casino_vault = &ctx.accounts.casino_vault;

        require!(
            financial_authority.key() == global_authority.finance_authority,
            GameError::UnauthorizedFinanceAdmin
        );

        require!(
            casino_vault.lamports() > amount,
            GameError::InsufficientCasinoVault
        );

        sol_transfer_with_signer(
            ctx.accounts.casino_vault.to_account_info(),
            recipient.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[&[VAULT_AUTHORITY_SEED.as_bytes(), &[casino_bump]]],
            amount,
        )?;

        let balance = ctx.accounts.casino_vault.to_account_info().lamports();

        msg!("Remaining balance: {:?}", balance);

        Ok(())
    }

    pub fn set_rtp(ctx: Context<SetGlobalPool>, new_rtp: u64) -> Result<()> {
        require!(new_rtp < 100, GameError::InvalidRtp);

        ctx.accounts.global_pool.rtp = new_rtp;

        Ok(())
    }

    pub fn set_max_win_amount(ctx: Context<SetGlobalPool>, new_max_win_amount: u64) -> Result<()> {
        ctx.accounts.global_pool.max_win_amount = new_max_win_amount;
        Ok(())
    }

    pub fn set_min_bet_amount(ctx: Context<SetGlobalPool>, new_min_bet_amount: u64) -> Result<()> {
        ctx.accounts.global_pool.min_bet_amount = new_min_bet_amount;
        Ok(())
    }

    pub fn set_operation_authority(
        ctx: Context<SetAuthority>,
        new_operation_authority: Pubkey,
    ) -> Result<()> {
        ctx.accounts.global_pool.operation_authority = new_operation_authority;
        Ok(())
    }

    pub fn set_finance_authority(
        ctx: Context<SetAuthority>,
        new_finance_authority: Pubkey,
    ) -> Result<()> {
        ctx.accounts.global_pool.finance_authority = new_finance_authority;
        Ok(())
    }

    pub fn set_update_authority(
        ctx: Context<SetAuthority>,
        new_update_authority: Pubkey,
    ) -> Result<()> {
        ctx.accounts.global_pool.update_authority = new_update_authority;
        Ok(())
    }
}
