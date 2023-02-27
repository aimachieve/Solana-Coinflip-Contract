use anchor_lang::prelude::*;
use {
    std::hash::{Hash, Hasher},
    std::collections::hash_map::DefaultHasher,
};

use anchor_spl::token::{
    self,
    Mint, 
    Token, 
    TokenAccount,
    Transfer
};
declare_id!("6Be5Dyf1P4fSjZ7VzA9wiGt44Dpa79nY1Xqt95vsYH8o");

#[program]
pub mod coin_flip {
    use super::*;
    pub fn create_treasury(_ctx: Context<CreateTreasury>) -> Result<()> {
        let trade_treasury = &mut _ctx.accounts.trade_treasury;
        if is_zero_account(&trade_treasury.to_account_info()) {
            trade_treasury.super_owner = _ctx.accounts.authority.key();
        }
        require(_ctx.accounts.authority.key() == trade_treasury.super_owner, "not allowed")?;
        trade_treasury.trade_mint = _ctx.accounts.trade_mint.key();
        trade_treasury.trade_vault = _ctx.accounts.trade_vault.key();
        trade_treasury.balance = 0;
        trade_treasury.decimals = _ctx.accounts.trade_mint.decimals as u32;
        Ok(())
    }

    pub fn claim_treasury(_ctx: Context<ClaimTreasury>) -> Result<()> {
        let trade_treasury = &mut _ctx.accounts.trade_treasury;
        let trade_vault = &mut _ctx.accounts.trade_vault;
        let trade_mint = &mut _ctx.accounts.trade_mint.key();
        require(_ctx.accounts.authority.key() == trade_treasury.super_owner, "not allowed")?;

        let cpi_program = _ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: trade_vault.to_account_info(),
            to: _ctx.accounts.user_vault.to_account_info(),
            authority: trade_treasury.to_account_info(),
        };

        let seeds = &[TREASURY_TAG, trade_mint.as_ref()];
        let (_treasury_key, bump) = Pubkey::find_program_address(seeds, &crate::ID);

        let signer_seeds = &[
            TREASURY_TAG,
            &trade_mint.as_ref(),
            &[bump],
        ];
        let signer = &[&signer_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);


        token::transfer(cpi_ctx, trade_vault.amount)?;
        trade_treasury.claimed_amount += trade_treasury.balance;

        if trade_vault.amount > trade_treasury.balance {
            trade_treasury.balance = 0;
        } else {
            trade_treasury.balance -= trade_vault.amount;
        }

        Ok(())
    }

    pub fn process_game(_ctx: Context<ProcessGame>, amount: u64, choice: bool, is_spin: bool) -> Result<()> {
        
        let user_treasury = &mut _ctx.accounts.user_treasury;
        user_treasury.initialize_if_needed(_ctx.accounts.authority.key(), _ctx.accounts.trade_treasury.key(), _ctx.accounts.trade_treasury.trade_mint, _ctx.accounts.trade_treasury.decimals);

        require(!is_spin || user_treasury.spin_win_cnt + user_treasury.spin_lose_cnt <= 10, "cannot spin more than 10")?;

        let trade_treasury = &mut _ctx.accounts.trade_treasury;
        let cpi_program = _ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: _ctx.accounts.user_vault.to_account_info(),
            to: _ctx.accounts.trade_vault.to_account_info(),
            authority: _ctx.accounts.authority.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::transfer(cpi_ctx, amount)?;
        trade_treasury.balance += amount;

        let result_num = get_random(_ctx.accounts.clock.unix_timestamp as u64, _ctx.accounts.authority.key()) % 2;
        let result = result_num == 0;

        if result == choice {
            if is_spin {
                user_treasury.balance += 500000000;
                user_treasury.spin_win_cnt += 1;
            } else {
                user_treasury.balance += amount * 2 * 98 / 100;
                user_treasury.general_win_cnt += 1;
            }
        } else {
            if is_spin {
                user_treasury.spin_lose_cnt += 1;
            } else {
                user_treasury.general_lose_cnt += 1;
            }
        }
        Ok(())
    }

    pub fn claim(_ctx: Context<ProcessGame>) -> Result<()> {
        let user_treasury = &mut _ctx.accounts.user_treasury;
        let trade_treasury = &mut _ctx.accounts.trade_treasury;
        let trade_mint = &mut _ctx.accounts.trade_mint.key();
        user_treasury.initialize_if_needed(_ctx.accounts.authority.key(), trade_treasury.key(), trade_treasury.trade_mint, trade_treasury.decimals);

        require(user_treasury.balance > 0, "No funds")?;

        let cpi_program = _ctx.accounts.token_program.to_account_info();
        let cpi_accounts = Transfer {
            from: _ctx.accounts.trade_vault.to_account_info(),
            to: _ctx.accounts.user_vault.to_account_info(),
            authority: trade_treasury.to_account_info(),
        };

        let seeds = &[TREASURY_TAG, trade_mint.as_ref()];
        let (_treasury_key, bump) = Pubkey::find_program_address(seeds, &crate::ID);

        let signer_seeds = &[
            TREASURY_TAG,
            &trade_mint.as_ref(),
            &[bump],
        ];
        let signer = &[&signer_seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::transfer(cpi_ctx, user_treasury.balance)?;
        trade_treasury.balance -= user_treasury.balance;
        user_treasury.balance = 0;

        Ok(())
    }

}
#[derive(Accounts)]
#[instruction()]
pub struct CreateTreasury<'info> {
    #[account(
        init,
        seeds = [TREASURY_TAG, &trade_mint.key().as_ref()],
        bump,
        payer = authority,
        space = std::mem::size_of::<TradeTreasury>() + 8,
    )]
    pub trade_treasury: Box<Account<'info, TradeTreasury>>,

    pub trade_mint: Box<Account<'info, Mint>>,
    #[account(
        init,
        token::mint = trade_mint,
        token::authority = trade_treasury,
        seeds = [VAULT_TAG, &trade_mint.key().as_ref()],
        bump,
        payer = authority)]
    pub trade_vault:Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction()]
pub struct ClaimTreasury<'info> {
    #[account(
        mut,
        seeds = [TREASURY_TAG, &trade_mint.key().as_ref()],
        bump,
    )]
    pub trade_treasury: Box<Account<'info, TradeTreasury>>,

    pub trade_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [VAULT_TAG, &trade_mint.key().as_ref()],
        bump,)]
    pub trade_vault:Box<Account<'info, TokenAccount>>,
    pub user_vault:Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[derive(Accounts)]
#[instruction()]
pub struct ProcessGame<'info> {
    #[account(
        mut,
        seeds = [TREASURY_TAG, &trade_mint.key().as_ref()],
        bump,
    )]
    pub trade_treasury: Box<Account<'info, TradeTreasury>>,

    pub trade_mint: Box<Account<'info, Mint>>,
    #[account(
        mut,
        seeds = [VAULT_TAG, &trade_mint.key().as_ref()],
        bump,)]
    pub trade_vault:Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed,
        payer= authority,
        seeds = [USER_TREASURY_TAG, authority.key().as_ref(), &trade_mint.key().as_ref()],
        bump,
        space = 8 + std::mem::size_of::<UserTreasury>())]
    pub user_treasury:Box<Account<'info, UserTreasury>>,
    #[account(mut)]
    pub user_vault:Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

#[account]
#[derive(Default)]
pub struct TradeTreasury {
    pub super_owner: Pubkey,
    pub trade_mint: Pubkey,
    pub trade_vault: Pubkey,
    pub balance: u64,
    pub claimed_amount: u64,
    pub decimals: u32,
    pub reserved: [u128; 5],
}

#[account]
#[derive(Default)]
pub struct UserTreasury {
    pub owner: Pubkey,
    pub trade_treasury: Pubkey,
    pub trade_mint: Pubkey,
    pub balance: u64,
    pub general_win_cnt: u64,
    pub general_lose_cnt: u64,
    pub spin_win_cnt: u64,
    pub spin_lose_cnt: u64,
    pub decimals: u32,
    pub reserved: [u128; 3],
}
impl UserTreasury {
    pub fn initialize_if_needed<'info>(&mut self, owner: Pubkey, trade_treasury: Pubkey, trade_mint: Pubkey, decimals: u32) {
        if self.trade_treasury != trade_treasury {
            self.owner = owner;
            self.trade_treasury = trade_treasury;
            self.trade_mint = trade_mint;
            self.balance = 0;
            self.decimals = decimals;
        }
    }
}

#[constant]
pub const TREASURY_TAG:&[u8] = b"coin-flip-treasury";
#[constant]
pub const VAULT_TAG:&[u8] = b"coin-flip-vault";
#[constant]
pub const USER_TREASURY_TAG:&[u8] = b"coin-flip-user-treasury";

#[error_code]
pub enum CoinFlipError {
    #[msg("Not Allowed")]
    NotAllowed,
}

pub fn get_random(cur_timestamp:u64, ticket_pubkey:Pubkey)->u64{
    let mut hasher1 = DefaultHasher::new(); 
    ticket_pubkey.hash(&mut hasher1);
    let determine_num = hasher1.finish()+cur_timestamp;

    return determine_num;
}

pub fn require(flag: bool, err_msg: &str) -> Result<()> {
    if !flag {
        msg!(err_msg);
        return Err(error!(CoinFlipError::NotAllowed));
    }
    Ok(())
}

pub fn is_zero_account(account_info:&AccountInfo)->bool{
    let account_data: &[u8] = &account_info.data.borrow();
    let len = account_data.len();
    let mut is_zero = true;
    for i in 0..len-1 {
        if account_data[i] != 0 {
            is_zero = false;
        }
    }
    is_zero
}