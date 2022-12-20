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
        _token_out: TokenIdentifier,
        _amount_out_min: BigUint,
    ) -> SwapTokensFixedInputResultType<Self::Api> {
        todo!()
    }
}