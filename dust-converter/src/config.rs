elrond_wasm::imports!();

pub type AddKnownTokenType<M> = MultiValue3<TokenIdentifier<M>, ManagedAddress<M>, BigUint<M>>;

pub const MAX_FEE_PERCENTAGE: u64 = 10_000u64;

#[elrond_wasm::module]
pub trait ConfigModule {

    #[only_owner]
    #[endpoint(setFeePercentage)]
    fn set_fee_percentage(&self, protocol_fee: u64) {
        require!(protocol_fee < MAX_FEE_PERCENTAGE, "Fee percent invalid");

        self.protocol_fee_percent().set(protocol_fee);
    }

    #[only_owner]
    #[endpoint(addKnownTokens)]
    fn add_known_tokens(&self, known_tokens: MultiValueEncoded<AddKnownTokenType<Self::Api>>) {
        let known_tokens_mapper = self.known_tokens();
        for entry in known_tokens {
            let (token, sc_address, min_amount) = entry.into_tuple();
            require!(token.is_valid_esdt_identifier(), "Invalid token ID");

            if !known_tokens_mapper.contains(&token) {
                known_tokens_mapper.add(&token);
            }

            self.known_contracts(token).set((sc_address, min_amount));
        }
    }

    #[only_owner]
    #[endpoint(removeKnownTokens)]
    fn remove_known_tokens(&self, tokens: MultiValueEncoded<TokenIdentifier>) {
        let known_tokens_mapper = self.known_tokens();
        for token in tokens {
            if known_tokens_mapper.contains(&token) {
                known_tokens_mapper.remove(&token);
                self.known_contracts(token).clear();
            }
        }
    }

    #[view(getAllKnownContracts)]
    #[storage_mapper("known_contracts")]
    fn known_contracts(&self, token_id: TokenIdentifier) -> SingleValueMapper<(ManagedAddress, BigUint)>;

    #[storage_mapper("known_tokens")]
    fn known_tokens(&self) -> WhitelistMapper<Self::Api, TokenIdentifier>;

    #[view(getProtocolFeePercent)]
    #[storage_mapper("protocol_fee_percent")]
    fn protocol_fee_percent(&self) -> SingleValueMapper<u64>;
}
