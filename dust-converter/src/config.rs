elrond_wasm::imports!();

pub type AddKnownTokenType<M> = MultiValue3<TokenIdentifier<M>, ManagedAddress<M>, BigUint<M>>;

pub const MAX_FEE_PERCENTAGE: u64 = 10_000u64;

#[elrond_wasm::module]
pub trait ConfigModule {

    #[only_owner]
    #[payable("*")]
    #[endpoint(topUp)]
    fn top_up(&self) {
        let (token_id, amount) = self.call_value().single_fungible_esdt();
        require!(token_id == self.wrapped_token().get(), "Invalid token");

        self.wrapped_token_amount().update(|x| *x += amount);
    }

    #[only_owner]
    #[endpoint(setFeePercentage)]
    fn set_fee_percentage(&self, protocol_fee: u64) {
        require!(protocol_fee < MAX_FEE_PERCENTAGE, "Fee percent invalid");

        self.protocol_fee_percent().set(protocol_fee);
    }

    #[only_owner]
    #[endpoint(setSlippagePercentage)]
    fn set_slippage_percentage(&self, slippage: u64) {
        require!(slippage < MAX_FEE_PERCENTAGE, "Slippage percent invalid");

        self.slippage_percent().set(slippage);
    }

    #[only_owner]
    #[endpoint(addKnownTokens)]
    fn add_known_tokens(&self, known_tokens: MultiValueEncoded<AddKnownTokenType<Self::Api>>) {
        let mut all_tokens_vec = self.all_tokens().get();
        let known_tokens_mapper = self.known_tokens();
        for entry in known_tokens {
            let (token, sc_address, min_amount) = entry.into_tuple();
            require!(token.is_valid_esdt_identifier(), "Invalid token ID");

            if !known_tokens_mapper.contains(&token) {
                require!(
                    self.blockchain().is_smart_contract(&sc_address),
                    "Invalid SC address"
                );

                known_tokens_mapper.add(&token);
                all_tokens_vec.push(token.clone());
                self.pair_contract(&token).set(sc_address);
                self.token_threshold(&token).set(min_amount);
            }
        }
        self.all_tokens().set(all_tokens_vec);
    }

    #[only_owner]
    #[endpoint(removeKnownTokens)]
    fn remove_known_tokens(&self, tokens: MultiValueEncoded<TokenIdentifier>) {
        let mut all_tokens_vec = self.all_tokens().get();
        let known_tokens_mapper = self.known_tokens();
        for token in tokens {
            if known_tokens_mapper.contains(&token) {
                known_tokens_mapper.remove(&token);

                unsafe {
                    let index = all_tokens_vec.find(&token).unwrap_unchecked();
                    all_tokens_vec.remove(index);
                }

                self.pair_contract(&token).clear();
                self.token_threshold(&token).clear();
            }
        }
        self.all_tokens().set(&all_tokens_vec);
    }


    #[view(getAllTokens)]
    fn get_all_tokens(&self) -> MultiValueEncoded<TokenIdentifier> {
        self.all_tokens().get().into()
    }

    #[storage_mapper("pair_contract")]
    fn pair_contract(&self, token_id: &TokenIdentifier) -> SingleValueMapper<ManagedAddress>;

    #[view(getTokenThreshold)]
    #[storage_mapper("token_threshold")]
    fn token_threshold(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("knownTokens")]
    fn known_tokens(&self) -> WhitelistMapper<Self::Api, TokenIdentifier>;

    #[storage_mapper("allTokens")]
    fn all_tokens(&self) -> SingleValueMapper<ManagedVec<TokenIdentifier>>;

    #[view(getProtocolFeePercent)]
    #[storage_mapper("protocol_fee_percent")]
    fn protocol_fee_percent(&self) -> SingleValueMapper<u64>;

    #[view(getSlippagePercent)]
    #[storage_mapper("slippage_percent")]
    fn slippage_percent(&self) -> SingleValueMapper<u64>;

    #[view(getWrappedTokenId)]
    #[storage_mapper("wrappedTokenId")]
    fn wrapped_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getWrappedTokenAmount)]
    #[storage_mapper("wrapped_token_amount")]
    fn wrapped_token_amount(&self) -> SingleValueMapper<BigUint>;
}
