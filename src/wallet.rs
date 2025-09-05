use crate::{
    errors::Result,
    signing::{SignedMessage, Signer as WalletSigner},
};
use alloy::{
    primitives::{Address, FixedBytes},
    signers::{Signer, local::PrivateKeySigner},
    sol_types::Eip712Domain,
};
use anyhow::Ok;

pub trait HyperLiquidSigningHash {
    fn hyperliquid_signing_hash(&self, domain: &Eip712Domain) -> FixedBytes<32>;
}

#[derive(Clone)]
pub struct LocalWallet {
    wallet_key: PrivateKeySigner,
}

impl LocalWallet {
    pub fn signer(pk: String) -> Self {
        Self {
            wallet_key: pk.parse().unwrap(),
        }
    }
    pub fn address(&self) -> Address {
        self.wallet_key.address()
    }

    pub async fn sign_hash(&self, hash: FixedBytes<32>) -> Result<alloy::signers::Signature> {
        Ok(self.wallet_key.sign_hash(&hash).await?)
    }
}

#[async_trait::async_trait]
impl WalletSigner for LocalWallet {
    async fn sign_order(&self, to_sign: FixedBytes<32>) -> Result<SignedMessage> {
        let signature = self.sign_hash(to_sign).await?;
        Ok(SignedMessage {
            r: signature.r(),
            s: signature.s(),
            v: signature.v() as u64 + 27,
        })
    }
}
