use crate::{errors::Result, hl::SignedMessage};
use alloy::{
    primitives::FixedBytes,
    signers::{Signer, local::PrivateKeySigner},
    sol_types::Eip712Domain,
};
use anyhow::Ok;
use qrcode::{QrCode, render::unicode};

pub trait HyperLiquidSigningHash {
    fn hyperliquid_signing_hash(&self, domain: &Eip712Domain) -> FixedBytes<32>;
}

pub enum Signers {
    Local(LocalWallet),
}
pub struct LocalWallet {
    wallet_key: PrivateKeySigner,
}

impl LocalWallet {
    pub fn signer(pk: String) -> Self {
        Self {
            wallet_key: pk.parse().unwrap(),
        }
    }
    pub fn print_wallet(&self) {
        let code = QrCode::new(format!(
            "https://blockscan.com/address/{}",
            self.wallet_key.address()
        ))
        .unwrap();
        let image = code
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Light)
            .light_color(unicode::Dense1x2::Dark)
            .build();
        println!("{image}");
    }

    pub async fn sign_hash(&self, hash: FixedBytes<32>) -> Result<alloy::signers::Signature> {
        Ok(self.wallet_key.sign_hash(&hash).await?)
    }
}

impl crate::HlAgentWallet for Signers {
    async fn sign_order(
        &self,
        domain: Eip712Domain,
        to_sign: FixedBytes<32>,
    ) -> Result<SignedMessage> {
        // let hash = to_sign.hyperliquid_signing_hash(&domain);
        let signature = match self {
            Signers::Local(wallet) => wallet.sign_hash(to_sign),
        }
        .await?;
        Ok(SignedMessage {
            r: signature.r(),
            s: signature.s(),
            v: signature.v() as u64 + 27,
        })
    }
}
