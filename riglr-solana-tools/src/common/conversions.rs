//! Type conversion utilities for bridging Solana SDK v3 with SPL libraries built on v2

use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Convert from Solana SDK v3 Pubkey to SPL-compatible Pubkey
///
/// SPL libraries (spl-token v8, spl-associated-token-account v7) are built against
/// Solana SDK v2 and have their own bundled Pubkey type. This function converts
/// between them by serializing to string and parsing back.
pub fn to_spl_pubkey(
    pubkey: &Pubkey,
) -> spl_associated_token_account::solana_program::pubkey::Pubkey {
    spl_associated_token_account::solana_program::pubkey::Pubkey::from_str(&pubkey.to_string())
        .expect("Valid pubkey conversion")
}

/// Convert from SPL-compatible Pubkey to Solana SDK v3 Pubkey
pub fn from_spl_pubkey(
    spl_pubkey: &spl_associated_token_account::solana_program::pubkey::Pubkey,
) -> Pubkey {
    Pubkey::from_str(&spl_pubkey.to_string()).expect("Valid pubkey conversion")
}

/// Convert from SPL token Pubkey to Solana SDK v3 Pubkey
pub fn from_spl_token_pubkey(spl_pubkey: &spl_token::solana_program::pubkey::Pubkey) -> Pubkey {
    Pubkey::from_str(&spl_pubkey.to_string()).expect("Valid pubkey conversion")
}

/// Get associated token address with type conversion
pub fn get_associated_token_address_v3(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let spl_owner = to_spl_pubkey(owner);
    let spl_mint = to_spl_pubkey(mint);
    let spl_ata = spl_associated_token_account::get_associated_token_address(&spl_owner, &spl_mint);
    from_spl_pubkey(&spl_ata)
}

/// Get associated token address with program ID and type conversion
pub fn get_associated_token_address_with_program_id_v3(
    owner: &Pubkey,
    mint: &Pubkey,
    token_program_id: &Pubkey,
) -> Pubkey {
    let spl_owner = to_spl_pubkey(owner);
    let spl_mint = to_spl_pubkey(mint);
    let spl_token_program = to_spl_pubkey(token_program_id);
    let spl_ata = spl_associated_token_account::get_associated_token_address_with_program_id(
        &spl_owner,
        &spl_mint,
        &spl_token_program,
    );
    from_spl_pubkey(&spl_ata)
}

/// Create associated token account idempotent instruction with type conversion
pub fn create_associated_token_account_idempotent_v3(
    funding_address: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> solana_sdk::instruction::Instruction {
    let spl_funding = to_spl_pubkey(funding_address);
    let spl_wallet = to_spl_pubkey(wallet_address);
    let spl_mint = to_spl_pubkey(token_mint_address);
    let spl_token_program = to_spl_pubkey(token_program_id);

    let spl_instruction =
        spl_associated_token_account::instruction::create_associated_token_account_idempotent(
            &spl_funding,
            &spl_wallet,
            &spl_mint,
            &spl_token_program,
        );

    // Convert SPL instruction to SDK v3 instruction
    solana_sdk::instruction::Instruction {
        program_id: from_spl_pubkey(&spl_instruction.program_id),
        accounts: spl_instruction
            .accounts
            .into_iter()
            .map(|meta| solana_sdk::instruction::AccountMeta {
                pubkey: from_spl_pubkey(&meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: spl_instruction.data,
    }
}

/// Create SPL token transfer instruction with type conversion
pub fn spl_token_transfer_v3(
    _token_program_id: &Pubkey,
    source: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    signers: &[&Pubkey],
    amount: u64,
) -> Result<solana_sdk::instruction::Instruction, Box<dyn std::error::Error>> {
    let spl_source = to_spl_pubkey(source);
    let spl_dest = to_spl_pubkey(destination);
    let spl_authority = to_spl_pubkey(authority);
    let spl_signers: Vec<_> = signers.iter().map(|s| to_spl_pubkey(s)).collect();
    let spl_signers_refs: Vec<_> = spl_signers.iter().collect();

    let spl_instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        &spl_source,
        &spl_dest,
        &spl_authority,
        &spl_signers_refs,
        amount,
    )?;

    // Convert SPL instruction to SDK v3 instruction
    Ok(solana_sdk::instruction::Instruction {
        program_id: from_spl_token_pubkey(&spl_instruction.program_id),
        accounts: spl_instruction
            .accounts
            .into_iter()
            .map(|meta| solana_sdk::instruction::AccountMeta {
                pubkey: from_spl_token_pubkey(&meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: spl_instruction.data,
    })
}

/// Initialize mint instruction with type conversion
pub fn initialize_mint2_v3(
    _token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) -> Result<solana_sdk::instruction::Instruction, Box<dyn std::error::Error>> {
    let spl_mint = to_spl_pubkey(mint_pubkey);
    let spl_mint_authority = to_spl_pubkey(mint_authority);
    let spl_freeze = freeze_authority.map(to_spl_pubkey);

    let spl_instruction = spl_token::instruction::initialize_mint2(
        &spl_token::id(),
        &spl_mint,
        &spl_mint_authority,
        spl_freeze.as_ref(),
        decimals,
    )?;

    // Convert SPL instruction to SDK v3 instruction
    Ok(solana_sdk::instruction::Instruction {
        program_id: from_spl_token_pubkey(&spl_instruction.program_id),
        accounts: spl_instruction
            .accounts
            .into_iter()
            .map(|meta| solana_sdk::instruction::AccountMeta {
                pubkey: from_spl_token_pubkey(&meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: spl_instruction.data,
    })
}

/// Mint to instruction with type conversion
pub fn mint_to_v3(
    _token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    account_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
) -> Result<solana_sdk::instruction::Instruction, Box<dyn std::error::Error>> {
    let spl_mint = to_spl_pubkey(mint_pubkey);
    let spl_account = to_spl_pubkey(account_pubkey);
    let spl_owner = to_spl_pubkey(owner_pubkey);
    let spl_signers: Vec<_> = signer_pubkeys.iter().map(|s| to_spl_pubkey(s)).collect();
    let spl_signers_refs: Vec<_> = spl_signers.iter().collect();

    let spl_instruction = spl_token::instruction::mint_to(
        &spl_token::id(),
        &spl_mint,
        &spl_account,
        &spl_owner,
        &spl_signers_refs,
        amount,
    )?;

    // Convert SPL instruction to SDK v3 instruction
    Ok(solana_sdk::instruction::Instruction {
        program_id: from_spl_token_pubkey(&spl_instruction.program_id),
        accounts: spl_instruction
            .accounts
            .into_iter()
            .map(|meta| solana_sdk::instruction::AccountMeta {
                pubkey: from_spl_token_pubkey(&meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: spl_instruction.data,
    })
}
