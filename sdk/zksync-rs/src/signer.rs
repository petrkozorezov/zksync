// Built-in imports
use eth_client::error::SignerError;
use eth_client::eth_signer::EthereumSigner;
use models::tx::TxEthSignature;
use std::fmt;
// External uses
use num::BigUint;
// Workspace uses
use models::tx::{ChangePubKey, PackedEthSignature};
use models::{AccountId, Address, Nonce, PubKeyHash, Token, Transfer, Withdraw};
use zksync_crypto::PrivateKey;

fn signing_failed_error(err: impl ToString) -> SignerError {
    SignerError::SigningFailed(err.to_string())
}

pub struct Signer {
    pub pubkey_hash: PubKeyHash,
    pub address: Address,
    pub(crate) private_key: PrivateKey,
    pub(crate) eth_signer: Option<EthereumSigner>,
    pub(crate) account_id: Option<AccountId>,
}

impl fmt::Debug for Signer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut pk_contents = Vec::new();
        self.private_key
            .write(&mut pk_contents)
            .expect("Failed writing the private key contents");
        f.debug_struct("Signer")
            .field("pubkey_hash", &self.pubkey_hash)
            .field("address", &self.address)
            .finish()
    }
}

impl Signer {
    pub fn new(
        private_key: PrivateKey,
        address: Address,
        eth_signer: Option<EthereumSigner>,
    ) -> Self {
        let pubkey_hash = PubKeyHash::from_privkey(&private_key);

        Self {
            private_key,
            pubkey_hash,
            address,
            eth_signer,
            account_id: None,
        }
    }

    pub fn pubkey_hash(&self) -> &PubKeyHash {
        &self.pubkey_hash
    }

    pub fn set_account_id(&mut self, account_id: Option<AccountId>) {
        self.account_id = account_id;
    }

    pub fn get_account_id(&self) -> Option<AccountId> {
        self.account_id
    }

    pub async fn sign_change_pubkey_tx(
        &self,
        nonce: Nonce,
        auth_onchain: bool,
    ) -> Result<ChangePubKey, SignerError> {
        let account_id = self.account_id.ok_or(SignerError::NoSigningKey)?;

        let eth_signature = if auth_onchain {
            None
        } else {
            let eth_signer = self
                .eth_signer
                .as_ref()
                .ok_or(SignerError::MissingEthPrivateKey)?; // TODO Change error code

            let sign_bytes =
                ChangePubKey::get_eth_signed_data(account_id, nonce, &self.pubkey_hash)
                    .map_err(signing_failed_error)?;

            let eth_signature = eth_signer
                .sign(&sign_bytes, true)
                .await
                .map_err(signing_failed_error)?;

            match eth_signature {
                TxEthSignature::EthereumSignature(packed_signature) => Some(packed_signature),
                _ => None,
            }
        };
        let change_pubkey = ChangePubKey {
            account_id,
            account: self.address,
            new_pk_hash: self.pubkey_hash.clone(),
            nonce,
            eth_signature,
        };

        if !auth_onchain {
            assert!(
                change_pubkey.verify_eth_signature() == Some(self.address),
                "eth signature is incorrect"
            );
        }

        Ok(change_pubkey)
    }

    pub async fn sign_transfer(
        &self,
        token: Token,
        amount: BigUint,
        fee: BigUint,
        to: Address,
        nonce: Nonce,
    ) -> Result<(Transfer, Option<PackedEthSignature>), SignerError> {
        let account_id = self.account_id.ok_or(SignerError::NoSigningKey)?;

        let transfer = Transfer::new_signed(
            account_id,
            self.address,
            to,
            token.id,
            amount,
            fee,
            nonce,
            &self.private_key,
        )
        .map_err(signing_failed_error)?;

        let eth_signature = match &self.eth_signer {
            Some(signer) => {
                let msg = transfer.get_ethereum_sign_message(&token.symbol, token.decimals);
                let signature = signer.sign(&msg.as_bytes(), true).await?; // TODO rename

                if let TxEthSignature::EthereumSignature(packed_signature) = signature {
                    Some(packed_signature)
                } else {
                    return Err(SignerError::MissingEthPrivateKey);
                }
            }
            _ => None,
        };

        Ok((transfer, eth_signature))
    }

    pub async fn sign_withdraw(
        &self,
        token: Token,
        amount: BigUint,
        fee: BigUint,
        eth_address: Address,
        nonce: Nonce,
    ) -> Result<(Withdraw, Option<PackedEthSignature>), SignerError> {
        let account_id = self.account_id.ok_or(SignerError::NoSigningKey)?;

        let withdraw = Withdraw::new_signed(
            account_id,
            self.address,
            eth_address,
            token.id,
            amount,
            fee,
            nonce,
            &self.private_key,
        )
        .map_err(signing_failed_error)?;

        let eth_signature = match &self.eth_signer {
            Some(signer) => {
                let msg = withdraw.get_ethereum_sign_message(&token.symbol, token.decimals);
                let signature = signer.sign(&msg.as_bytes(), true).await?; // TODO rename

                if let TxEthSignature::EthereumSignature(packed_signature) = signature {
                    Some(packed_signature)
                } else {
                    return Err(SignerError::MissingEthPrivateKey);
                }
            }
            _ => None,
        };

        Ok((withdraw, eth_signature))
    }
}
