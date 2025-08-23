use alloy::{
    dyn_abi::Eip712Domain,
    hex::hex,
    primitives::{FixedBytes, U256, address, keccak256},
    sol,
    sol_types::{SolStruct, eip712_domain},
};

use serde::{Deserialize, Serialize};

use crate::{
    HyperLiquidSigningHash, SendAssetRequest,
    errors::{Errors, Result},
    hl::{SignedMessage, TransferRequest},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExchangeResponse {
    pub status: String,
    pub response: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ExchangeRequest {
    pub action: serde_json::Value,
    pub signature: SignedMessage,
    pub nonce: u64,
}

sol! {
    #[derive(Serialize)]
    struct UsdClassTransfer {
        string hyperliquidChain;
        string amount;
        bool toPerp;
        uint64 nonce;
    }

    #[derive(Serialize)]
    struct SendAsset {
        string hyperliquidChain;
        string destination;
        string sourceDex;
        string destinationDex;
        string token;
        string amount;
        string fromSubAccount;
        uint64 nonce;
    }

    #[derive(Serialize,Debug)]
    struct Agent {
        string source;
        bytes32 connectionId;
    }
}

impl HyperLiquidSigningHash for Agent {
    fn hyperliquid_signing_hash(&self, domain: &Eip712Domain) -> FixedBytes<32> {
        self.eip712_signing_hash(domain)
    }
}

#[derive(Clone)]
pub struct TransferClass<S>
where
    S: SolStruct,
{
    pub(crate) inner: S,
    type_string: String,
}

impl<S> HyperLiquidSigningHash for TransferClass<S>
where
    S: SolStruct,
{
    fn hyperliquid_signing_hash(&self, domain: &Eip712Domain) -> FixedBytes<32> {
        let type_hash = keccak256(self.type_string.as_bytes());

        let encoded_data = self.inner.eip712_encode_data();

        let mut struct_hash_input = Vec::new();
        struct_hash_input.extend_from_slice(type_hash.as_slice());
        struct_hash_input.extend_from_slice(&encoded_data);
        let struct_hash: FixedBytes<32> = keccak256(&struct_hash_input);

        let mut signing_input = [0u8; 2 + 32 + 32];
        signing_input[0] = 0x19;
        signing_input[1] = 0x01;
        signing_input[2..34].copy_from_slice(domain.hash_struct().as_slice());
        signing_input[34..66].copy_from_slice(struct_hash.as_slice());

        keccak256(signing_input)
    }
}

pub fn generate_transfer_params(
    req: &TransferRequest,
) -> Result<(TransferClass<UsdClassTransfer>, Eip712Domain)> {
    let hex_str = req.sig_chain_id.strip_prefix("0x").unwrap_or(&req.chain);
    let chain_raw = hex::decode(hex_str)?;
    let chain_id: u64 = U256::from_be_slice(chain_raw.as_slice()).try_into()?;

    Ok((
        TransferClass {
            type_string: "HyperliquidTransaction:UsdClassTransfer(string hyperliquidChain,string amount,bool toPerp,uint64 nonce)".to_owned(),
            inner: UsdClassTransfer {
                hyperliquidChain: req.chain.clone(),
                amount: req.amount.clone(),
                toPerp: req.to_perp,
                nonce: req.nonce,
            },
        },
        eip712_domain! {
            name : "HyperliquidSignTransaction",
            version : "1",
            chain_id : chain_id,
            verifying_contract : address!("0x0000000000000000000000000000000000000000"),
        },
    ))
}

pub fn generate_send_asset_params(
    req: &SendAssetRequest,
) -> Result<(TransferClass<SendAsset>, Eip712Domain)> {
    let hex_str = req.sig_chain_id.strip_prefix("0x").unwrap_or(&req.chain);
    let chain_raw = hex::decode(hex_str)?;
    let chain_id: u64 = U256::from_be_slice(chain_raw.as_slice()).try_into()?;

    Ok((
        TransferClass {
            type_string: "HyperliquidTransaction:SendAsset(string hyperliquidChain,string destination,string sourceDex,string destinationDex,string token,string amount,string fromSubAccount,uint64 nonce)".to_owned(),
            inner: SendAsset {
                hyperliquidChain: req.chain.clone(),
                destination:req.destination.clone(),
                sourceDex: req.source_dex.clone(),
                destinationDex: req.dst_dex.clone(),
                token: req.token.clone(),
                amount: req.amount.clone(),
                fromSubAccount: req.from_sub_account.clone(),
                nonce: req.nonce,
            }
        },
        eip712_domain! {
            name : "HyperliquidSignTransaction",
            version : "1",
            chain_id : chain_id,
            verifying_contract : address!("0x0000000000000000000000000000000000000000"),
        },
    ))
}

pub fn generate_action_params(
    action: &crate::Actions,
    is_mainnet: bool,
    nonce: u64,
) -> Result<(Agent, Eip712Domain)> {
    let mut bytes =
        rmp_serde::to_vec_named(action).map_err(|e| Errors::AgentSignature(e.to_string()))?;
    bytes.extend(nonce.to_be_bytes());
    bytes.push(0);
    let out: FixedBytes<32> = keccak256(bytes.clone());
    let source = if is_mainnet { "a" } else { "b" }.to_string();
    let data = Agent {
        source,
        connectionId: out,
    };
    Ok((
        data,
        eip712_domain! {
            name : "Exchange",
            version : "1",
            chain_id : 1337,
            verifying_contract : address!("0x0000000000000000000000000000000000000000"),
        },
    ))
}
