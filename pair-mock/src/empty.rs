#![no_std]
elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub type SwapTokensFixedInputResultType<BigUint> = EsdtTokenPayment<BigUint>;

pub const ERR_TOKEN: &[u8] = b"ERR-a89kl3";
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