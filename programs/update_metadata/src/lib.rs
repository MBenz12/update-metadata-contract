use anchor_lang::{
    prelude::*,
    solana_program::{
        clock,
        program::invoke
    }
};
use anchor_spl::{
    token::{Mint, Token, TokenAccount, transfer, Transfer},
    associated_token::{create, Create, AssociatedToken}
};
use mpl_token_metadata::{
    instruction::update_metadata_accounts_v2, 
    state::{DataV2, Metadata, TokenMetadataAccount}
};

declare_id!("A2StQ8kXhQfa4EsEZxh6zwNftQ1wXxPj17JNJzpCeUMQ");

pub const VAULT_POOL_SEED_PREFIX: &str = "vault_pool";

pub fn get_now_timestamp() -> u64 {
    clock::Clock::get()
        .unwrap()
        .unix_timestamp
        .try_into()
        .unwrap()
}

#[program]
pub mod update_metadata {
    use super::*;

    pub fn initialize_vault(ctx: Context<InitializeVault>, vault_bump: u8) -> Result<()> {
        if ctx.accounts.vault_pool.owner == &System::id() {
            let cpi_context = CpiContext::new(
                ctx.accounts.associated_token.to_account_info(),
                Create {
                    payer: ctx.accounts.payer.to_account_info(),
                    associated_token: ctx.accounts.vault_pool_ata.to_account_info(),
                    authority: ctx.accounts.vault_pool.to_account_info(),
                    mint: ctx.accounts.flwr_mint.to_account_info(),
                    rent: ctx.accounts.rent.to_account_info(),
                    token_program: ctx.accounts.token_program.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info(),
                },
            );
            create(cpi_context)?;
        }

        let vault = &mut ctx.accounts.vault;
        vault.bump = vault_bump;
        vault.mint_accounts = vec![];
        vault.updated_times = vec![];
        Ok(())
    }

    pub fn update(ctx: Context<Update>, is_update: bool, spec: bool, new_uri: String) -> Result<()> {
        if is_update == true {
            let amount: u64 = match spec {
                true => 33_333_000_000_000,
                false => 38_333_000_000_000
            };

            let cpi_context = CpiContext::new(
                ctx.accounts.token_program.to_account_info().clone(),
                Transfer {
                    from: ctx.accounts.claimer_ata.to_account_info().clone(),
                    to: ctx.accounts.vault_pool_ata.to_account_info().clone(),
                    authority: ctx.accounts.claimer.to_account_info().clone(),
                }
            ); 
            transfer(cpi_context, amount)?;
        }

        let metadata: Metadata = Metadata::from_account_info(&ctx.accounts.metadata.to_account_info())?;

        let data = metadata.data;
        let datav2 = DataV2 {
            name: data.name,
            symbol: data.symbol,
            uri: new_uri,
            seller_fee_basis_points: data.seller_fee_basis_points,
            creators: data.creators,
            collection: metadata.collection,
            uses: metadata.uses,
        };

        let accounts = vec![
            ctx.accounts.token_metadata_program.to_account_info().clone(),
            ctx.accounts.metadata.to_account_info().clone(),
            ctx.accounts.update_authority.to_account_info().clone(),
            ctx.accounts.claimer.to_account_info().clone(),
            ctx.accounts.token_program.to_account_info().clone(),
            ctx.accounts.system_program.to_account_info().clone(),
            ctx.accounts.rent.to_account_info().clone()
        ];
        
        invoke(
            &update_metadata_accounts_v2(
                ctx.accounts.token_metadata_program.key(),
                ctx.accounts.metadata.key(),
                ctx.accounts.update_authority.key(),
                None,
                Some(datav2),
                None,
                None,
            ),
            &accounts
        )?;

        let vault = &mut ctx.accounts.vault;
        let index = vault.mint_accounts.iter().position(|x| x.key() == ctx.accounts.nft_mint.key());
        if is_update == true {
            if let Some(index) = index {
                vault.updated_times[index as usize] = get_now_timestamp();
            } else {
                vault.mint_accounts.push(ctx.accounts.nft_mint.key());
                vault.updated_times.push(get_now_timestamp());
            }
        } else {
            if let Some(index) = index {
                vault.updated_times.remove(index);
                vault.mint_accounts.remove(index);
            }
        }
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(vault_bump: u8)]
pub struct InitializeVault<'info> {
    #[account(mut)]
    payer: Signer<'info>,

    #[account(zero)]
    vault: Account<'info, Vault>,
    
    #[account(seeds = [VAULT_POOL_SEED_PREFIX.as_bytes(), vault.key().as_ref()], bump = vault_bump)]
    vault_pool: SystemAccount<'info>,
    
    /// CHECK:
    #[account(mut)]
    flwr_mint: AccountInfo<'info>,
    
    /// CHECK:
    #[account(mut)]
    vault_pool_ata: AccountInfo<'info>,
    
    rent: Sysvar<'info, Rent>,
    
    #[account(address = anchor_spl::associated_token::ID)]
    associated_token: Program<'info, AssociatedToken>,

    #[account(address = spl_token::id())]
    token_program: Program<'info, Token>,

    system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    claimer: Signer<'info>,

    #[account(mut)]
    nft_mint: Account<'info, Mint>,

    /// CHECK:
    #[account(mut)]
    metadata: AccountInfo<'info>,

    #[account(mut)]
    update_authority: Signer<'info>,
    
    #[account(mut)]
    vault: Account<'info, Vault>,
    
    /// CHECK:
    #[account(seeds = [VAULT_POOL_SEED_PREFIX.as_bytes(), vault.key().as_ref()], bump = vault.bump)]
    vault_pool: AccountInfo<'info>,
    
    #[account(mut)]
    flwr_mint: Account<'info, Mint>,

    #[account(mut)]
    claimer_ata: Account<'info, TokenAccount>,
    
    #[account(mut)]
    vault_pool_ata: Account<'info, TokenAccount>,

    #[account(address = spl_token::id())]
    token_program: Program<'info, Token>,

    /// CHECK:
    token_metadata_program: UncheckedAccount<'info>,

    rent: Sysvar<'info, Rent>,

    system_program: Program<'info, System>,
}

#[account]
pub struct Vault {                          // 8
    pub bump: u8,                           // 1
    pub mint_accounts: Vec<Pubkey>,         // 32 * 333 + 8
    pub updated_times: Vec<u64>,            // 8  * 333 + 8
}
