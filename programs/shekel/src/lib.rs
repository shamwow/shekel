use anchor_lang::prelude::*;
use anchor_spl::token::{transfer as spl_token_transfer, TokenAccount, Token, Mint};

declare_id!("EcDwM6SLq81xpKS1ykf7UGjjyE84KJvjmAzWmLwy9tJx");

static OWNER: &str = "8FXRKgS2nDJ1axRRTvdgkQudUsBZZ5gKnp4zF1kK6vMw";

const BASIS_PTS_PRECISION: u64 = 10000;
const ONE_SHEKEL_TOKEN: u64 = 1000000;
// Once this much money has moved via shekel, the reward should be TARGET_REWARD.
const TARGET_MOVED_AMT: u64 = 1000000000;
const TARGET_REWARD: u64 = 1;
const INITIAL_REWARD: u64 = ONE_SHEKEL_TOKEN;


#[program]
pub mod shekel {
    use std::borrow::Borrow;
    use super::*;

    pub fn init(
        ctx: Context<Init>,
        merchant_tx_fee_basis_points: u64,
        purchase_protection_fee_basis_points: u64,
    ) -> Result<()> {
        // Init network config PDA.
        ctx.accounts.network_config.shekel_token_mint = ctx.accounts.shekel_token_mint.key();
        ctx.accounts.network_config.usdc_mint = ctx.accounts.usdc_mint.key();
        ctx.accounts.network_config.merchant_tx_fee_basis_points = merchant_tx_fee_basis_points;
        ctx.accounts.network_config.purchase_protection_fee_basis_points = purchase_protection_fee_basis_points;

        // Init stats PDA.
        ctx.accounts.stats.amount_moved = 0;
        ctx.accounts.stats.amount_rewarded_sender = 0;
        ctx.accounts.stats.amount_rewarded_recipient = 0;

        Ok(())
    }

    pub fn set_network_config(
        ctx: Context<SetNetworkConfig>,
        merchant_tx_fee_basis_points: u64,
        purchase_protection_fee_basis_points: u64,
    ) -> Result<()> {
        ctx.accounts.network_config.shekel_token_mint = ctx.accounts.shekel_token_mint.key();
        ctx.accounts.network_config.usdc_mint = ctx.accounts.usdc_mint.key();
        ctx.accounts.network_config.merchant_tx_fee_basis_points = merchant_tx_fee_basis_points;
        ctx.accounts.network_config.purchase_protection_fee_basis_points = purchase_protection_fee_basis_points;
        return Ok(());
    }

    pub fn transfer_pool(
        ctx: Context<TransferPool>,
        amt: u64,
    ) -> Result<()> {
        transfer_usdc(
            ctx.accounts.pool_account.borrow(),
            ctx.accounts.destination.borrow(),
            ctx.accounts.authority.to_account_info().borrow(),
            ctx.accounts.spl_token_program.borrow(),
            amt,
        )
    }

    pub fn transfer_treasury(
        ctx: Context<TransferTreasury>,
        amt: u64,
    ) -> Result<()> {
        reward_tokens(
            ctx.accounts.destination.borrow(),
            ctx.accounts.token_treasury.borrow(),
            ctx.accounts.spl_token_program.borrow(),
            ctx.accounts.authority.to_account_info().borrow(),
            amt,
        )
    }

    pub fn transact(ctx: Context<TransferUSDC>, amt: u64) -> Result<()> {
        let spl_token_program = ctx.accounts.spl_token_program.to_account_info();

        if amt == 0 {
            return err!(ErrorCode::ZeroAmt);
        }
        let fee_basis_pts = ctx.accounts.network_config.merchant_tx_fee_basis_points;
        let fee_amt = (amt * fee_basis_pts) / BASIS_PTS_PRECISION;
        if amt <= fee_amt {
            return err!(ErrorCode::FeeGreaterThanAmt);
        }

        // Send fee to pool.
        match transfer_usdc(
            ctx.accounts.source.borrow(),
            ctx.accounts.pool_account.borrow(),
            ctx.accounts.owner.borrow(),
            spl_token_program.borrow(),
            fee_amt,
        ) {
            Ok(_) => {},
            Err(e) => return Err(e),
        }

        // Move rest to the destination.
        match transfer_usdc(
            ctx.accounts.source.borrow(),
            ctx.accounts.destination.borrow(),
            ctx.accounts.owner.borrow(),
            spl_token_program.borrow(),
            amt - fee_amt,
        ) {
            Ok(_) => {},
            Err(e) => return Err(e),
        }

        ctx.accounts.stats.amount_moved += amt;

        if fee_amt == 0 {
            return Ok(());
        }

        // Since there was a fee paid, reward with tokens.
        let mut reward_amt = get_reward_rate(ctx.accounts.stats.amount_moved);

        // Make sure there are enough tokens in the treasury to reward.
        if ctx.accounts.treasury.amount == 0 {
            return Ok(());
        } else if reward_amt > ctx.accounts.treasury.amount {
            reward_amt = ctx.accounts.treasury.amount;
        }

        // Reward the sender.
        let sender_reward = reward_amt / 2;
        match reward_tokens(
            ctx.accounts.source_token_account.borrow(),
            ctx.accounts.treasury.borrow(),
            spl_token_program.borrow(),
            ctx.accounts.authority.to_account_info().borrow(),
            sender_reward,
        ) {
            Ok(_) => {},
            Err(e) => return Err(e),
        }

        // Reward the recipient.
        let recipient_reward = reward_amt - sender_reward;
        match reward_tokens(
            ctx.accounts.destination_token_account.borrow(),
            ctx.accounts.treasury.borrow(),
            spl_token_program.borrow(),
            ctx.accounts.authority.to_account_info().borrow(),
            recipient_reward,
        ) {
            Ok(_) => {},
            Err(e) => return Err(e),
        }

        ctx.accounts.stats.amount_rewarded_sender += sender_reward;
        ctx.accounts.stats.amount_rewarded_recipient += recipient_reward;

        return Ok(());
    }
}

fn transfer_usdc<'info>(
    source: &Account<'info, TokenAccount>,
    destination: &Account<'info, TokenAccount>,
    authority: &AccountInfo<'info>,
    spl_token_program: &AccountInfo<'info>,
    amt: u64,
) -> Result<()> {
    let cpi_accounts = anchor_spl::token::Transfer {
        from: source.to_account_info(),
        to: destination.to_account_info(),
        authority: authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        spl_token_program.clone(),
        cpi_accounts,
    );
    return spl_token_transfer(cpi_ctx,amt);
}

fn reward_tokens<'info>(
    destination: &Account<'info, TokenAccount>,
    treasury: &Account<'info, TokenAccount>,
    spl_token_program: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    amt: u64,
) -> Result<()> {
    let (_, bump) = Pubkey::find_program_address(&[b"authority_v4"], &crate::ID);
    let seeds: &[&[&[u8]]] = &[&[b"authority_v4", &[bump]]];
    let cpi_accounts = anchor_spl::token::Transfer {
        from: treasury.to_account_info(),
        to: destination.to_account_info(),
        authority: authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new_with_signer(
        spl_token_program.clone(),
        cpi_accounts,
        seeds,
    );
    return spl_token_transfer(cpi_ctx,amt);
}

pub fn get_reward_rate(transferred_amt: u64) -> u64 {
    if transferred_amt >= TARGET_MOVED_AMT {
        TARGET_REWARD
    } else {
        INITIAL_REWARD - ((INITIAL_REWARD - TARGET_REWARD) * transferred_amt) / TARGET_MOVED_AMT
    }
}

#[account]
#[derive(Default)]
pub struct NetworkConfig {
    merchant_tx_fee_basis_points: u64,
    purchase_protection_fee_basis_points: u64,
    usdc_mint: Pubkey,
    shekel_token_mint: Pubkey,
}

#[account]
#[derive(Default)]
pub struct Stats {
    // Has space for 20 stats.
    amount_moved: u64,
    amount_rewarded_sender: u64,
    amount_rewarded_recipient: u64,
}

#[account]
#[derive(Default)]
pub struct Authority {}

#[derive(Accounts)]
pub struct TransferUSDC<'info> {
    #[account(signer)]
    /// CHECK: This is not dangerous because we don't read or write from this account.
    owner: AccountInfo<'info>,
    #[account(mut, has_one = owner, constraint = source.mint == network_config.usdc_mint)]
    source: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = source_token_account.mint == network_config.shekel_token_mint)]
    source_token_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = destination.mint == network_config.usdc_mint)]
    destination: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = destination_token_account.mint == network_config.shekel_token_mint)]
    destination_token_account: Box<Account<'info, TokenAccount>>,
    spl_token_program: Program<'info, Token>,
    #[account(seeds = [b"config_v4".as_ref()], bump)]
    network_config: Box<Account<'info, NetworkConfig>>,
    #[account(mut, seeds = [b"pool_v4".as_ref()], bump, constraint = pool_account.mint == network_config.usdc_mint)]
    pool_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [b"stats_v4".as_ref()], bump)]
    stats: Box<Account<'info, Stats>>,
    #[account(mut, seeds = [b"treasury_v4".as_ref()], bump, constraint = treasury.mint == network_config.shekel_token_mint)]
    treasury: Box<Account<'info, TokenAccount>>,
    #[account(mut, seeds = [b"authority_v4".as_ref()], bump)]
    authority: Box<Account<'info, Authority>>
}

#[derive(Accounts)]
pub struct Init<'info> {
    #[account(init, payer = payer, seeds = [b"config_v4".as_ref()], bump)]
    network_config: Box<Account<'info, NetworkConfig>>,
    system_program: Program<'info, System>,
    #[account(mut, address = OWNER.parse::<Pubkey>().unwrap())]
    payer: Signer<'info>,
    #[account(init, payer = payer, seeds = [b"stats_v4".as_ref()], bump)]
    stats: Box<Account<'info, Stats>>,
    shekel_token_mint: Box<Account<'info, Mint>>,
    usdc_mint: Box<Account<'info, Mint>>,
    #[account(init, payer = payer, seeds = [b"authority_v4".as_ref()], bump)]
    authority: Box<Account<'info, Authority>>,
    #[account(init, payer = payer, seeds = [b"pool_v4".as_ref()], bump, token::authority = authority, token::mint = usdc_mint)]
    pool_account: Box<Account<'info, TokenAccount>>,
    #[account(init, payer = payer, seeds = [b"treasury_v4".as_ref()], bump, token::authority = authority, token::mint = shekel_token_mint)]
    token_treasury: Box<Account<'info, TokenAccount>>,
    /// CHECK: safe because we don't read or write from here. Used to initialize the token_treasury PDA.
    token_program: AccountInfo<'info>,
    // Used to initialize the token_treasury PDA.
    rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct SetNetworkConfig<'info> {
    #[account(mut, seeds = [b"config_v4".as_ref()], bump)]
    network_config: Box<Account<'info, NetworkConfig>>,
    shekel_token_mint: Box<Account<'info, Mint>>,
    usdc_mint: Box<Account<'info, Mint>>,
    #[account(address = OWNER.parse::<Pubkey>().unwrap())]
    signer: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferTreasury<'info> {
    #[account(mut, seeds = [b"treasury_v4".as_ref()], bump)]
    token_treasury: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = destination.mint == network_config.shekel_token_mint)]
    destination: Box<Account<'info, TokenAccount>>,
    #[account(seeds = [b"config_v4".as_ref()], bump)]
    network_config: Box<Account<'info, NetworkConfig>>,
    spl_token_program: Program<'info, Token>,
    #[account(address = OWNER.parse::<Pubkey>().unwrap())]
    signer: Signer<'info>,
    #[account(mut, seeds = [b"authority_v4".as_ref()], bump)]
    authority: Box<Account<'info, Authority>>
}

#[derive(Accounts)]
pub struct TransferPool<'info> {
    #[account(mut, seeds = [b"pool_v4".as_ref()], bump)]
    pool_account: Box<Account<'info, TokenAccount>>,
    #[account(mut, constraint = destination.mint == network_config.usdc_mint)]
    destination: Box<Account<'info, TokenAccount>>,
    #[account(seeds = [b"config_v4".as_ref()], bump)]
    network_config: Box<Account<'info, NetworkConfig>>,
    spl_token_program: Program<'info, Token>,
    #[account(address = OWNER.parse::<Pubkey>().unwrap())]
    signer: Signer<'info>,
    #[account(mut, seeds = [b"authority_v4".as_ref()], bump)]
    authority: Box<Account<'info, Authority>>
}

#[error_code]
pub enum ErrorCode {
    #[msg("Fee computed to be greater than amt")]
    FeeGreaterThanAmt,
    #[msg("Amt can't be 0")]
    ZeroAmt,
    #[msg("Invalid pool account")]
    InvalidPoolAccount,
}
