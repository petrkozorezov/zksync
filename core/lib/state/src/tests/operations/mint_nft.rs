use num::{BigUint, Zero};
use web3::types::H256;

use zksync_crypto::params::{
    MIN_NFT_TOKEN_ID, NFT_STORAGE_ACCOUNT_ADDRESS, NFT_STORAGE_ACCOUNT_ID, NFT_TOKEN_ID,
};
use zksync_types::{
    tokens::NFT, AccountUpdate, Address, MintNFT, Nonce, SignedZkSyncTx, TokenId, Transfer,
    ZkSyncTx, H160,
};

use crate::tests::{AccountState::*, PlasmaTestBuilder};

/// Check MintNFT operation
#[test]
fn mint_success() {
    let fee_token_id = TokenId(0);
    let fee = BigUint::from(10u32);

    let mut tb = PlasmaTestBuilder::new();

    let (creator_account_id, mut creator_account, creator_sk) = tb.add_account(Unlocked);
    tb.set_balance(creator_account_id, fee_token_id, 20u32);

    let (to_account_id, to_account, _to_sk) = tb.add_account(Locked);
    let content_hash = H256::default();
    let mint_nft = MintNFT::new_signed(
        creator_account_id,
        creator_account.address,
        content_hash,
        to_account.address,
        fee.clone(),
        fee_token_id,
        creator_account.nonce,
        &creator_sk,
    )
    .unwrap();

    let token_hash: Vec<u8> = vec![
        22, 150, 31, 94, 253, 41, 54, 65, 235, 128, 255, 236, 34, 19, 209, 244, 186, 158, 170, 230,
        95, 69, 89, 207, 183, 93, 214, 213, 45, 174, 51, 194,
    ];
    let token_address = Address::from_slice(&token_hash[12..]);
    let balance = BigUint::from(MIN_NFT_TOKEN_ID);
    let nft = NFT::new(
        TokenId(MIN_NFT_TOKEN_ID),
        0,
        creator_account_id,
        token_address,
        None,
        content_hash,
    );

    let token_data = BigUint::from_bytes_be(&token_hash[16..]);
    tb.test_tx_success(
        mint_nft.into(),
        &[
            // Create special nft storage account
            (
                NFT_STORAGE_ACCOUNT_ID,
                AccountUpdate::Create {
                    address: *NFT_STORAGE_ACCOUNT_ADDRESS,
                    nonce: Nonce(0),
                },
            ),
            // Add Minimum NFT token id to NFT storage account balance
            (
                NFT_STORAGE_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                    balance_update: (NFT_TOKEN_ID, BigUint::zero(), balance),
                },
            ),
            // Increment NFT counter
            (
                NFT_STORAGE_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                    balance_update: (
                        NFT_TOKEN_ID,
                        BigUint::from(MIN_NFT_TOKEN_ID),
                        BigUint::from(MIN_NFT_TOKEN_ID + 1),
                    ),
                },
            ),
            // Pay fee for minting nft
            (
                creator_account_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: creator_account.nonce,
                    new_nonce: creator_account.nonce,
                    balance_update: (fee_token_id, BigUint::from(20u32), BigUint::from(10u32)),
                },
            ),
            // Increment counter of nft tokens for creator
            (
                creator_account_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: creator_account.nonce,
                    new_nonce: creator_account.nonce + 1,
                    balance_update: (NFT_TOKEN_ID, BigUint::zero(), BigUint::from(1u32)),
                },
            ),
            // Mint nft
            (
                creator_account_id,
                AccountUpdate::MintNFT { token: nft.clone() },
            ),
            // Deposit nft token to recipient account
            (
                to_account_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: to_account.nonce,
                    new_nonce: to_account.nonce,
                    balance_update: (nft.id, BigUint::zero(), BigUint::from(1u32)),
                },
            ),
            // Store part of nft token hash as balance to NFT storage account id
            (
                NFT_STORAGE_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    old_nonce: to_account.nonce,
                    new_nonce: to_account.nonce,
                    balance_update: (nft.id, BigUint::zero(), token_data),
                },
            ),
        ],
    );

    // Create another nft
    creator_account.nonce.0 += 1;
    let (to_account_id, to_account, _to_sk) = tb.add_account(Locked);
    let content_hash = H256::default();
    let mint_nft = MintNFT::new_signed(
        creator_account_id,
        creator_account.address,
        content_hash,
        to_account.address,
        fee.clone(),
        fee_token_id,
        creator_account.nonce,
        &creator_sk,
    )
    .unwrap();

    let token_hash: Vec<u8> = vec![
        40, 111, 148, 133, 22, 50, 233, 206, 55, 114, 54, 47, 147, 227, 204, 20, 62, 91, 163, 18,
        248, 69, 144, 106, 242, 0, 60, 234, 75, 178, 210, 166,
    ];
    let token_address = Address::from_slice(&token_hash[12..]);
    let nft = NFT::new(
        TokenId(MIN_NFT_TOKEN_ID + 1),
        1,
        creator_account_id,
        token_address,
        None,
        content_hash,
    );

    let token_data = BigUint::from_bytes_be(&token_hash[16..]);
    tb.test_tx_success(
        mint_nft.into(),
        &[
            // Increment NFT counter
            (
                NFT_STORAGE_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                    balance_update: (
                        NFT_TOKEN_ID,
                        BigUint::from(MIN_NFT_TOKEN_ID + 1),
                        BigUint::from(MIN_NFT_TOKEN_ID + 2),
                    ),
                },
            ),
            // Pay fee for minting nft
            (
                creator_account_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: creator_account.nonce,
                    new_nonce: creator_account.nonce,
                    balance_update: (fee_token_id, fee, BigUint::zero()),
                },
            ),
            // Increment counter of nft tokens for creator
            (
                creator_account_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: creator_account.nonce,
                    new_nonce: creator_account.nonce + 1,
                    balance_update: (NFT_TOKEN_ID, BigUint::from(1u32), BigUint::from(2u32)),
                },
            ),
            // Mint nft
            (
                creator_account_id,
                AccountUpdate::MintNFT { token: nft.clone() },
            ),
            // Deposit nft token to recipient account
            (
                to_account_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: to_account.nonce,
                    new_nonce: to_account.nonce,
                    balance_update: (nft.id, BigUint::zero(), BigUint::from(1u32)),
                },
            ),
            // Store part of nft token hash as balance to NFT storage account id
            (
                NFT_STORAGE_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    old_nonce: to_account.nonce,
                    new_nonce: to_account.nonce,
                    balance_update: (nft.id, BigUint::zero(), token_data),
                },
            ),
        ],
    )
}

#[test]
fn mint_token_to_new_account() {
    let fee_token_id = TokenId(0);
    let fee = BigUint::from(10u32);
    let zero_amount = BigUint::from(0u32);

    let balance_from = BigUint::from(20u32);

    let balance_to = BigUint::from(0u64);

    let mut tb = PlasmaTestBuilder::new();

    let (creator_account_id, creator_account, sk) = tb.add_account(Unlocked);
    tb.set_balance(creator_account_id, fee_token_id, balance_from.clone());

    let new_address = H160::random();

    let transfer_1 = Transfer::new_signed(
        creator_account_id,
        creator_account.address,
        new_address,
        fee_token_id,
        zero_amount,
        fee.clone(),
        creator_account.nonce,
        Default::default(),
        &sk,
    )
    .unwrap();

    let signed_zk_sync_tx1 = SignedZkSyncTx {
        tx: ZkSyncTx::Transfer(Box::new(transfer_1)),
        eth_sign_data: None,
    };

    let new_id = tb.state.get_free_account_id();

    let content_hash = H256::default();
    let mint_nft = MintNFT::new_signed(
        creator_account_id,
        creator_account.address,
        content_hash,
        new_address,
        fee.clone(),
        fee_token_id,
        creator_account.nonce,
        &sk,
    )
    .unwrap();

    let token_hash: Vec<u8> = vec![
        22, 150, 31, 94, 253, 41, 54, 65, 235, 128, 255, 236, 34, 19, 209, 244, 186, 158, 170, 230,
        95, 69, 89, 207, 183, 93, 214, 213, 45, 174, 51, 194,
    ];
    let token_address = Address::from_slice(&token_hash[12..]);
    let balance = BigUint::from(MIN_NFT_TOKEN_ID);
    let nft = NFT::new(
        TokenId(MIN_NFT_TOKEN_ID),
        0,
        creator_account_id,
        token_address,
        None,
        content_hash,
    );

    let token_data = BigUint::from_bytes_be(&token_hash[16..]);

    let signed_zk_sync_mint = SignedZkSyncTx {
        tx: ZkSyncTx::MintNFT(Box::new(mint_nft)),
        eth_sign_data: None,
    };

    tb.test_txs_batch_success(
        &[signed_zk_sync_tx1, signed_zk_sync_mint],
        &[
            // Create new account
            (
                new_id,
                AccountUpdate::Create {
                    address: new_address,
                    nonce: Nonce(0),
                },
            ),
            // Pay for for creating account
            (
                creator_account_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: creator_account.nonce,
                    new_nonce: creator_account.nonce + 1,
                    balance_update: (fee_token_id, balance_from, fee),
                },
            ),
            // Transfer zero token to new account (TransferToNew operation)
            (
                new_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                    balance_update: (fee_token_id, balance_to.clone(), balance_to),
                },
            ),
            // Create special nft storage account
            (
                NFT_STORAGE_ACCOUNT_ID,
                AccountUpdate::Create {
                    address: *NFT_STORAGE_ACCOUNT_ADDRESS,
                    nonce: Nonce(0),
                },
            ),
            // Add Minimum NFT token id to NFT storage account balance
            (
                NFT_STORAGE_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                    balance_update: (NFT_TOKEN_ID, BigUint::zero(), balance),
                },
            ),
            // Increment NFT counter
            (
                NFT_STORAGE_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                    balance_update: (
                        NFT_TOKEN_ID,
                        BigUint::from(MIN_NFT_TOKEN_ID),
                        BigUint::from(MIN_NFT_TOKEN_ID + 1),
                    ),
                },
            ),
            // Pay fee for minting nft
            (
                creator_account_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: creator_account.nonce + 1,
                    new_nonce: creator_account.nonce + 1,
                    balance_update: (fee_token_id, BigUint::from(10u32), BigUint::from(0u32)),
                },
            ),
            // Increment counter of nft tokens for creator
            (
                creator_account_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: creator_account.nonce + 1,
                    new_nonce: creator_account.nonce + 2,
                    balance_update: (NFT_TOKEN_ID, BigUint::zero(), BigUint::from(1u32)),
                },
            ),
            // Mint nft
            (
                creator_account_id,
                AccountUpdate::MintNFT { token: nft.clone() },
            ),
            // Deposit nft token to recipient account
            (
                new_id,
                AccountUpdate::UpdateBalance {
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                    balance_update: (nft.id, BigUint::zero(), BigUint::from(1u32)),
                },
            ),
            // Store part of nft token hash as balance to NFT storage account id
            (
                NFT_STORAGE_ACCOUNT_ID,
                AccountUpdate::UpdateBalance {
                    old_nonce: Nonce(0),
                    new_nonce: Nonce(0),
                    balance_update: (nft.id, BigUint::zero(), token_data),
                },
            ),
        ],
    );
}

/// Check MINT NFT failure if recipient address does not exist
/// does not correspond to accound_id
#[test]
fn mint_already_created_nft() {
    let fee_token_id = TokenId(0);
    let fee = BigUint::from(10u32);

    let mut tb = PlasmaTestBuilder::new();

    let (creator_account_id, creator_account, creator_sk) = tb.add_account(Unlocked);
    tb.set_balance(creator_account_id, fee_token_id, 20u32);

    let (to_account_id, mut to_account, _to_sk) = tb.add_account(Locked);

    let nft_token_id = TokenId(MIN_NFT_TOKEN_ID);
    to_account.set_balance(nft_token_id, BigUint::from(1u32));
    tb.state.insert_account(to_account_id, to_account.clone());
    let content_hash = H256::default();
    let mint_nft = MintNFT::new_signed(
        creator_account_id,
        creator_account.address,
        content_hash,
        to_account.address,
        fee,
        fee_token_id,
        creator_account.nonce,
        &creator_sk,
    )
    .unwrap();

    tb.test_tx_fail(
        mint_nft.into(),
        format!("Token {} is already in account", nft_token_id).as_str(),
    )
}