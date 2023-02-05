#![no_std]
elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub type SwapTokensFixedInputResultType<BigUint> = EsdtTokenPayment<BigUint>;

pub const ERR_TOKEN: &[u8] = b"ERR-a89kl3";
pub const KNOWN_TOKEN_1: &[u8] = b"USDC-0a3f5r";
pub const KNOWN_TOKEN_2: &[u8] = b"ASH-12345a";
pub const KNOWN_TOKEN_3: &[u8] = b"RIDE-12345a";
pub const KNOWN_TOKEN_4: &[u8] = b"RARE-12345a";
pub const KNOWN_TOKEN_5: &[u8] = b"LPAD-12345a";

pub const MAX_PERCENTAGE: u64 = 10_000;
pub const TOKEN_1_RATE_PERCENTAGE: u64 = 400; //   1000 TOKEN1 = 40 TOKEN_OUT
pub const TOKEN_2_RATE_PERCENTAGE: u64 = 30; //    1000 TOKEN2 = 3 TOKEN_OUT
pub const TOKEN_3_RATE_PERCENTAGE: u64 = 150; //   1000 TOKEN3 = 15 TOKEN_OUT
pub const TOKEN_4_RATE_PERCENTAGE: u64 = 2_300; // 1000 TOKEN4 = 230 TOKEN_OUT
pub const TOKEN_5_RATE_PERCENTAGE: u64 = 4_670; // 1000 TOKEN5 = 467 TOKEN_OUT
pub const AMOUNT_OUT: u64 = 1_000_000_000u64;

#[elrond_wasm::derive::contract]
pub trait PairMock {

    #[init]
    fn init(&self) {}

    #[view(getAmountOut)]
    fn get_amount_out_view(&self, token_in: TokenIdentifier, amount_in: BigUint) -> BigUint {
        require!(amount_in > 0u64, "Amount cannot be zero");
        if token_in == TokenIdentifier::from(ERR_TOKEN) {
            sc_panic!("Not enough reserve");
        }

        if token_in == TokenIdentifier::from(KNOWN_TOKEN_1) {
            return (amount_in * TOKEN_1_RATE_PERCENTAGE) / MAX_PERCENTAGE;
        }

        if token_in == TokenIdentifier::from(KNOWN_TOKEN_2) {
            return (amount_in * TOKEN_2_RATE_PERCENTAGE) / MAX_PERCENTAGE;
        }

        if token_in == TokenIdentifier::from(KNOWN_TOKEN_3) {
            return (amount_in * TOKEN_3_RATE_PERCENTAGE) / MAX_PERCENTAGE;
        }

        if token_in == TokenIdentifier::from(KNOWN_TOKEN_4) {
            return (amount_in * TOKEN_4_RATE_PERCENTAGE) / MAX_PERCENTAGE;
        }

        if token_in == TokenIdentifier::from(KNOWN_TOKEN_5) {
            return (amount_in * TOKEN_5_RATE_PERCENTAGE) / MAX_PERCENTAGE;
        }

        BigUint::from(AMOUNT_OUT)
    }

    #[payable("*")]
    #[endpoint(swapTokensFixedInput)]
    fn swap_tokens_fixed_input(
        &self,
        token_out: TokenIdentifier,
        amount_out_min: BigUint,
    ) -> EsdtTokenPayment {
        let (_token_in, _, _amount_in) = self.call_value().single_esdt().into_tuple();
        let caller = self.blockchain().get_caller();
        let payment = EsdtTokenPayment::new(token_out.clone(), 0, amount_out_min.clone());

        self.send().direct_esdt(&caller, &token_out, 0, &amount_out_min);

        payment
    }
}