use hl_sol::sol;

sol! {
    #[multisig]
    #[derive(Serialize)]
    struct TestJest {
        hyperliquidChain: string,
        amount: string,
        toPerp: bool,
        nonce: uint64
    }
}

fn main() {
    print!("{}", TEST_JEST_TYPE)
}
