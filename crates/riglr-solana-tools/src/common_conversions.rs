// TODO: Remove this module once SPL libraries are updated to use solana-sdk v3.x.
// See: https://github.com/riglr/riglr/issues/XX
//
//! Type conversion utilities for bridging Solana SDK v3 with SPL libraries built on v2
//!
//! # Background
//!
//! This module provides critical type conversion utilities that work around a significant
//! dependency version mismatch in the Solana ecosystem:
//!
//! - **riglr-solana-tools** uses `solana-sdk` v3.x (the latest stable version)
//! - **SPL libraries** (`spl-token`, `spl-associated-token-account`) internally bundle
//!   older versions of `solana-sdk` (typically v2.x)
//!
//! # The Problem
//!
//! SPL libraries don't use the main `solana-sdk` crate as a regular dependency. Instead,
//! they bundle their own copy of Solana types under their own module paths (e.g.,
//! `spl_token::solana_program::pubkey::Pubkey`). This creates incompatible types even
//! though they represent the same underlying data.
//!
//! # The Solution
//!
//! These conversion functions bridge the gap by:
//! 1. Converting v3 types to strings (a common format)
//! 2. Parsing the strings back into the SPL-bundled v2 types
//! 3. Providing convenience wrappers for common operations
//!
//! # Maintenance Warning
//!
//! This workaround is necessary until SPL libraries update to use `solana-sdk` v3.x
//! directly. When that happens, this module can be removed and direct type usage
//! can be restored throughout the codebase.
//!
//! # Example
//!
//! ```rust,ignore
//! use solana_sdk::pubkey::Pubkey;
//! use riglr_solana_tools::common::conversions::{to_spl_pubkey, from_spl_pubkey};
//!
//! // Convert v3 Pubkey to SPL-compatible Pubkey
//! let v3_pubkey = Pubkey::new_unique();
//! let spl_pubkey = to_spl_pubkey(&v3_pubkey);
//!
//! // Use with SPL functions
//! let ata = spl_associated_token_account::get_associated_token_address(
//!     &spl_pubkey,
//!     &spl_mint_pubkey,
//! );
//!
//! // Convert back to v3
//! let v3_ata = from_spl_pubkey(&ata);
//! ```

use core::error::Error as StdError;
use core::str::FromStr as _;
use solana_address::Address as SolanaAddress;
use solana_pubkey::Pubkey as SolanaPubkey;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};
use solana_system_interface::instruction as system_instruction;
use spl_associated_token_account::{
    instruction::create_associated_token_account_idempotent,
    solana_program::pubkey::Pubkey as SplPubkey,
};
use spl_token::{
    instruction::{initialize_mint2, mint_to, transfer},
    solana_program::pubkey::Pubkey as SplTokenPubkey,
};

/// Convert from Solana SDK v3 Pubkey to SPL-compatible Pubkey
///
/// SPL libraries (spl-token v8, spl-associated-token-account v7) are built against
/// Solana SDK v2 and have their own bundled Pubkey type. This function converts
/// between them by serializing to string and parsing back.
///
/// # Panics
///
/// This function should never panic since we're converting between validated Pubkey types
/// that represent the same underlying data. If it does panic, there's a critical bug
/// in the Solana SDK type implementations.
#[must_use]
#[inline]
#[expect(clippy::expect_used)]
pub fn to_spl_pubkey(pubkey: &Pubkey) -> SplPubkey {
    SplPubkey::from_str(&pubkey.to_string()).expect("Valid pubkey conversion")
}

/// Convert from SPL-compatible Pubkey to Solana SDK v3 Pubkey
///
/// # Panics
///
/// This function should never panic since we're converting between validated Pubkey types
/// that represent the same underlying data. If it does panic, there's a critical bug
/// in the Solana SDK type implementations.
#[must_use]
#[inline]
#[expect(clippy::expect_used)]
pub fn from_spl_pubkey(spl_pubkey: &SplPubkey) -> Pubkey {
    Pubkey::from_str(&spl_pubkey.to_string()).expect("Valid pubkey conversion")
}

/// Convert from SPL token Pubkey to Solana SDK v3 Pubkey
///
/// # Panics
///
/// This function should never panic since we're converting between validated Pubkey types
/// that represent the same underlying data. If it does panic, there's a critical bug
/// in the Solana SDK type implementations.
#[must_use]
#[inline]
#[expect(clippy::expect_used)]
pub fn from_spl_token_pubkey(spl_pubkey: &SplTokenPubkey) -> Pubkey {
    Pubkey::from_str(&spl_pubkey.to_string()).expect("Valid pubkey conversion")
}

/// Convert from Solana SDK v3 Pubkey to solana-pubkey Pubkey
///
/// # Panics
///
/// This function should never panic since we're converting between validated Pubkey types
/// that represent the same underlying data. If it does panic, there's a critical bug
/// in the Solana SDK type implementations.
#[must_use]
#[inline]
#[expect(clippy::expect_used)]
pub fn to_solana_pubkey(pubkey: &Pubkey) -> SolanaPubkey {
    SolanaPubkey::from_str(&pubkey.to_string()).expect("Valid pubkey conversion")
}

/// Convert from solana-pubkey Pubkey to Solana SDK v3 Pubkey
///
/// # Panics
///
/// This function should never panic since we're converting between validated Pubkey types
/// that represent the same underlying data. If it does panic, there's a critical bug
/// in the Solana SDK type implementations.
#[must_use]
#[inline]
#[expect(clippy::expect_used)]
pub fn from_solana_pubkey(solana_pubkey: &SolanaPubkey) -> Pubkey {
    Pubkey::from_str(&solana_pubkey.to_string()).expect("Valid pubkey conversion")
}

/// Convert from Solana SDK v3 Pubkey to solana-address Address
///
/// # Panics
///
/// This function should never panic since we're converting between validated Pubkey types
/// that represent the same underlying data. If it does panic, there's a critical bug
/// in the Solana SDK type implementations.
#[must_use]
#[inline]
#[expect(clippy::expect_used)]
pub fn to_solana_address(pubkey: &Pubkey) -> SolanaAddress {
    SolanaAddress::from_str(&pubkey.to_string()).expect("Valid pubkey conversion")
}

/// Convert from solana-address Address to Solana SDK v3 Pubkey
///
/// # Panics
///
/// This function should never panic since we're converting between validated Pubkey types
/// that represent the same underlying data. If it does panic, there's a critical bug
/// in the Solana SDK type implementations.
#[must_use]
#[inline]
#[expect(clippy::expect_used)]
pub fn from_solana_address(solana_address: &SolanaAddress) -> Pubkey {
    Pubkey::from_str(&solana_address.to_string()).expect("Valid pubkey conversion")
}

/// Get associated token address with type conversion
#[must_use]
#[inline]
pub fn get_associated_token_address_v3(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    let spl_owner = to_spl_pubkey(owner);
    let spl_mint = to_spl_pubkey(mint);
    let spl_ata = spl_associated_token_account::get_associated_token_address(&spl_owner, &spl_mint);
    from_spl_pubkey(&spl_ata)
}

/// Get associated token address with program ID and type conversion
#[must_use]
#[inline]
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
#[must_use]
#[inline]
pub fn create_associated_token_account_idempotent_v3(
    funding_address: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> Instruction {
    let spl_funding = to_spl_pubkey(funding_address);
    let spl_wallet = to_spl_pubkey(wallet_address);
    let spl_mint = to_spl_pubkey(token_mint_address);
    let spl_token_program = to_spl_pubkey(token_program_id);

    let spl_instruction = create_associated_token_account_idempotent(
        &spl_funding,
        &spl_wallet,
        &spl_mint,
        &spl_token_program,
    );

    // Convert SPL instruction to SDK v3 instruction
    Instruction {
        program_id: from_spl_pubkey(&spl_instruction.program_id),
        accounts: spl_instruction
            .accounts
            .into_iter()
            .map(|meta| AccountMeta {
                pubkey: from_spl_pubkey(&meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: spl_instruction.data,
    }
}

/// Create SPL token transfer instruction with type conversion
///
/// # Errors
///
/// Returns an error if the SPL token instruction creation fails due to invalid parameters
/// or internal SPL token library constraints.
#[inline]
pub fn spl_token_transfer_v3(
    _token_program_id: &Pubkey,
    source: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    signers: &[&Pubkey],
    amount: u64,
) -> Result<Instruction, Box<dyn StdError>> {
    let spl_source = to_spl_pubkey(source);
    let spl_dest = to_spl_pubkey(destination);
    let spl_authority = to_spl_pubkey(authority);
    let spl_signers: Vec<_> = signers.iter().map(|signer| to_spl_pubkey(signer)).collect();
    let spl_signers_refs: Vec<_> = spl_signers.iter().collect();

    let spl_instruction = transfer(
        &spl_token::id(),
        &spl_source,
        &spl_dest,
        &spl_authority,
        &spl_signers_refs,
        amount,
    )?;

    // Convert SPL instruction to SDK v3 instruction
    Ok(Instruction {
        program_id: from_spl_token_pubkey(&spl_instruction.program_id),
        accounts: spl_instruction
            .accounts
            .into_iter()
            .map(|meta| AccountMeta {
                pubkey: from_spl_token_pubkey(&meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: spl_instruction.data,
    })
}

/// Initialize mint instruction with type conversion
///
/// # Errors
///
/// Returns an error if the SPL token instruction creation fails due to invalid parameters
/// or internal SPL token library constraints.
#[inline]
pub fn initialize_mint2_v3(
    _token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) -> Result<Instruction, Box<dyn StdError>> {
    let spl_mint = to_spl_pubkey(mint_pubkey);
    let spl_mint_authority = to_spl_pubkey(mint_authority);
    let spl_freeze = freeze_authority.map(to_spl_pubkey);

    let spl_instruction = initialize_mint2(
        &spl_token::id(),
        &spl_mint,
        &spl_mint_authority,
        spl_freeze.as_ref(),
        decimals,
    )?;

    // Convert SPL instruction to SDK v3 instruction
    Ok(Instruction {
        program_id: from_spl_token_pubkey(&spl_instruction.program_id),
        accounts: spl_instruction
            .accounts
            .into_iter()
            .map(|meta| AccountMeta {
                pubkey: from_spl_token_pubkey(&meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: spl_instruction.data,
    })
}

/// Mint to instruction with type conversion
///
/// # Errors
///
/// Returns an error if the SPL token instruction creation fails due to invalid parameters
/// or internal SPL token library constraints.
#[inline]
pub fn mint_to_v3(
    _token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    account_pubkey: &Pubkey,
    owner_pubkey: &Pubkey,
    signer_pubkeys: &[&Pubkey],
    amount: u64,
) -> Result<Instruction, Box<dyn StdError>> {
    let spl_mint = to_spl_pubkey(mint_pubkey);
    let spl_account = to_spl_pubkey(account_pubkey);
    let spl_owner = to_spl_pubkey(owner_pubkey);
    let spl_signers: Vec<_> = signer_pubkeys
        .iter()
        .map(|signer| to_spl_pubkey(signer))
        .collect();
    let spl_signers_refs: Vec<_> = spl_signers.iter().collect();

    let spl_instruction = mint_to(
        &spl_token::id(),
        &spl_mint,
        &spl_account,
        &spl_owner,
        &spl_signers_refs,
        amount,
    )?;

    // Convert SPL instruction to SDK v3 instruction
    Ok(Instruction {
        program_id: from_spl_token_pubkey(&spl_instruction.program_id),
        accounts: spl_instruction
            .accounts
            .into_iter()
            .map(|meta| AccountMeta {
                pubkey: from_spl_token_pubkey(&meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: spl_instruction.data,
    })
}

/// System transfer instruction using solana-sdk v3 directly
#[must_use]
#[inline]
pub fn system_transfer_v3(from_pubkey: &Pubkey, to_pubkey: &Pubkey, lamports: u64) -> Instruction {
    let from_solana_addr = to_solana_address(from_pubkey);
    let to_solana_addr = to_solana_address(to_pubkey);
    let system_instruction =
        system_instruction::transfer(&from_solana_addr, &to_solana_addr, lamports);

    // Convert system instruction to SDK v3 instruction
    Instruction {
        program_id: from_solana_address(&system_instruction.program_id),
        accounts: system_instruction
            .accounts
            .into_iter()
            .map(|meta| AccountMeta {
                pubkey: from_solana_address(&meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: system_instruction.data,
    }
}

/// System create account instruction using solana-sdk v3 directly
#[must_use]
#[inline]
pub fn system_create_account_v3(
    from_pubkey: &Pubkey,
    to_pubkey: &Pubkey,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> Instruction {
    let from_solana_addr = to_solana_address(from_pubkey);
    let to_solana_addr = to_solana_address(to_pubkey);
    let owner_solana_addr = to_solana_address(owner);
    let system_instruction = system_instruction::create_account(
        &from_solana_addr,
        &to_solana_addr,
        lamports,
        space,
        &owner_solana_addr,
    );

    // Convert system instruction to SDK v3 instruction
    Instruction {
        program_id: from_solana_address(&system_instruction.program_id),
        accounts: system_instruction
            .accounts
            .into_iter()
            .map(|meta| AccountMeta {
                pubkey: from_solana_address(&meta.pubkey),
                is_signer: meta.is_signer,
                is_writable: meta.is_writable,
            })
            .collect(),
        data: system_instruction.data,
    }
}
