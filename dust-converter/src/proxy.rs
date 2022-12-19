elrond_wasm::imports!();

mod pair_proxy {
    elrond_wasm::imports!();

    #[elrond_wasm::proxy]
    pub trait PairProxy {
        
        #[view(getAmountOut)]
        fn get_amount_out_view(
            &self, 
            token_in: TokenIdentifier, 
            amount_in: BigUint
        ) -> BigUint;

        #[endpoint(swapTokensFixedInput)]
        fn swap_tokens_fixed_input(
            &self,
            token_out: TokenIdentifier,
            amount_out_min: BigUint
        ) -> EsdtTokenPayment;
    }
}
#[elrond_wasm::module]
pub trait ProxyModule {

    fn get_amount_out(
        &self,
        pair_address: ManagedAddress,
        token_in: TokenIdentifier,
        amount_in: BigUint
    ) -> BigUint {
        self.pair_proxy(pair_address)
            .get_amount_out_view(token_in, amount_in)
            .execute_on_dest_context()
    }

    fn swap_tokens_fixed_input(
        &self,
        pair_address: ManagedAddress,
        token_in: TokenIdentifier,
        amount_in: BigUint,
        token_out: TokenIdentifier,
        amount_out_min: BigUint
    ) -> EsdtTokenPayment {
        let payment = EsdtTokenPayment::new(token_in, 0, amount_in);

        self.pair_proxy(pair_address)
            .swap_tokens_fixed_input(token_out, amount_out_min)
            .with_esdt_transfer(payment)
            .execute_on_dest_context()
    }

    #[proxy]
    fn pair_proxy(&self, to: ManagedAddress) -> pair_proxy::Proxy<Self::Api>;
}