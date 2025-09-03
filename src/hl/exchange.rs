use alloy::{
    dyn_abi::Eip712Domain,
    hex::hex,
    primitives::{FixedBytes, U256, address, keccak256},
    sol as alloy_sol,
    sol_types::{SolStruct, eip712_domain},
};

use hl_sol::sol;

use serde::{Deserialize, Serialize};

use crate::{
    HyperLiquidSigningHash,
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
    #[multisig]
    #[derive(Serialize)]
    struct UsdClassTransfer {
        hyperliquidChain: string,
        amount: string,
        toPerp: bool,
        nonce: uint64
    }
}

sol! {
    #[multisig]
    #[derive(Serialize)]
    struct SendAsset {
        hyperliquidChain: string,
        destination: string,
        sourceDex: string,
        destinationDex: string,
        token: string,
        amount: string,
        fromSubAccount: string,
        nonce: uint64
    }
}

sol! {
    #[multisig]
    #[derive(Serialize, Debug)]
    struct ConvertToMultiSigUser {
        hyperliquidChain: string,
        signers: string,
        nonce: uint64
    }
}

alloy_sol! {
    #[derive(Serialize, Debug)]
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

pub fn hyperliquid_signing_hash<S: SolStruct>(
    type_str: String,
    data: S,
    domain: &Eip712Domain,
) -> FixedBytes<32> {
    let type_hash = keccak256(type_str.as_bytes());

    let encoded_data = data.eip712_encode_data();

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

pub fn hyperliquid_signing_hash_with_default_domain<S: SolStruct>(
    type_str: String,
    data: S,
    sig_chain: u64,
) -> FixedBytes<32> {
    let domain = eip712_domain! {
        name : "HyperliquidSignTransaction",
        version : "1",
        chain_id : sig_chain,
        verifying_contract : address!("0x0000000000000000000000000000000000000000"),
    };
    hyperliquid_signing_hash(type_str, data, &domain)
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
