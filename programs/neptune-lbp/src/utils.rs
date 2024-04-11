use anchor_lang::prelude::*;
use anchor_spl::token::{self, TransferChecked};
use anchor_spl::token_2022::{self, Transfer, ID as T22ID};
use solana_program::{program::invoke, program::invoke_signed, system_instruction};

fn transfer_token_2022<'info>(
    token_program: AccountInfo<'info>,  // Token-2022 program account
    source_account: AccountInfo<'info>, // Source token account
    mint_account: AccountInfo<'info>,   // Token mint account
    destination_account: AccountInfo<'info>, // Destination token account
    authority_account: AccountInfo<'info>, // Authority account (source account's owner/delegate)
    signer_seeds: Option<&[&[&[u8]]]>,  // Seeds for the signer, if required
    amount: u64,                        // Amount of tokens to transfer
    decimals: u8,                       // Token decimals
) -> Result<()> {
    // Setting up the accounts for the TransferChecked instruction#
    let transfer_checked_accounts = TransferChecked {
        from: source_account,
        mint: mint_account,
        to: destination_account,
        authority: authority_account,
    };

    let transfer_checked_ctx = if let Some(seeds) = signer_seeds {
        let seeds_slice: &[&[&[u8]]] = seeds;

        CpiContext::new_with_signer(
            token_program,
            transfer_checked_accounts,
            seeds_slice, // Use provided signer seeds
        )
    } else {
        CpiContext::new(token_program, transfer_checked_accounts)
    };

    // Perform the transfer
    token::transfer_checked(transfer_checked_ctx, amount, decimals)?;

    Ok(())
}

fn transfer_token_std<'info>(
    token_program: AccountInfo<'info>,       // Token program account
    source_account: AccountInfo<'info>,      // Source account
    mint_account: AccountInfo<'info>,        // Token mint account
    destination_account: AccountInfo<'info>, // Destination account
    authority_account: AccountInfo<'info>,   // Authority account (source account's owner/delegate)
    signer_seeds: Option<&[&[&[u8]]]>,       // Seeds for the signer, if required
    amount: u64,                             // The amount of tokens to transfer
    decimals: u8, // Number of base 10 digits to the right of the decimal place
) -> Result<()> {
    let transfer_checked_accounts = TransferChecked {
        from: source_account,
        mint: mint_account,
        to: destination_account,
        authority: authority_account,
    };

    let transfer_checked_ctx = if let Some(seeds) = signer_seeds {
        let seeds_slice: &[&[&[u8]]] = seeds;
        CpiContext::new_with_signer(
            token_program,
            transfer_checked_accounts,
            seeds_slice, // Use provided signer seeds
        )
    } else {
        CpiContext::new(token_program, transfer_checked_accounts)
    };

    // Perform the transfer
    token::transfer_checked(transfer_checked_ctx, amount, decimals)?;

    Ok(())
}

fn transfer_sol<'info>(
    source_account: &AccountInfo<'info>,
    destination_account: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    amount: u64,
    signer_seeds: Option<&[&[&[u8]]]>,
) -> Result<()> {
    // Create the transfer instruction
    let transfer_instruction =
        system_instruction::transfer(source_account.key, destination_account.key, amount);
    msg!("checks {:?}", source_account);
    // Conditionally invoke the transfer instruction with or without signing capabilities
    match signer_seeds {
        Some(seeds) => {
            // Invoke the transfer instruction with signing capabilities if seeds are provided
            invoke_signed(
                &transfer_instruction,
                &[
                    source_account.clone(),
                    destination_account.clone(),
                    system_program.clone(),
                ],
                seeds, // Use the provided signer seeds
            )?;
        }
        None => {
            // Directly invoke the transfer instruction without signing capabilities if no seeds are provided
            invoke(
                &transfer_instruction,
                &[
                    source_account.clone(),
                    destination_account.clone(),
                    system_program.clone(),
                ],
            )?;
        }
    }

    Ok(())
}

pub fn transfer_router<'info>(
    is_sol: bool,
    token_program: Option<AccountInfo<'info>>,
    source_account: AccountInfo<'info>,
    mint_account: Option<AccountInfo<'info>>,
    destination_account: AccountInfo<'info>,
    authority_account: Option<AccountInfo<'info>>,
    signer_seeds: Option<&[&[&[u8]]]>,
    amount: u64,
    decimals: Option<u8>,
    system_program: Option<AccountInfo<'info>>,
) -> Result<()> {
    if is_sol {
        transfer_sol(
            &source_account,
            &destination_account,
            &system_program.expect("System program account info required for SOL transfer"),
            amount,
            signer_seeds,
        )?;
    } else {
        // Determine if it's Token or Token-2022 based on token_program
        let is_token_2022 = token_program
            .as_ref()
            .map_or(false, |p| p.key != &token_2022::ID);

        if is_token_2022 {
            // Perform Token-2022 transfer
            transfer_token_2022(
                token_program.expect("Token-2022 program account info required"),
                source_account,
                mint_account.expect("Mint account info required for Token-2022 transfer"),
                destination_account,
                authority_account.expect("Authority account info required for Token-2022 transfer"),
                signer_seeds,
                amount,
                decimals.expect("Decimals required for Token-2022 transfer"),
            )?;
        } else {
            // Perform standard SPL Token transfer
            transfer_token_std(
                token_program.expect("Token program account info required for SPL transfer"),
                source_account,
                mint_account.expect("Mint account info required for SPL transfer"),
                destination_account,
                authority_account.expect("Authority account info required for SPL transfer"),
                signer_seeds,
                amount,
                decimals.expect("Decimals required for SPL transfer"),
            )?;
        }
    }
    Ok(())
}
