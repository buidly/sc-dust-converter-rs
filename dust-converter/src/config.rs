elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub type AddKnownTokenType<M> = MultiValue3<TokenIdentifier<M>, ManagedAddress<M>, BigUint<M>>;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, PartialEq, Debug)]
pub struct PairContractData<M: ManagedTypeApi> {
    pub address: ManagedAddress<M>,
    pub output_token: TokenIdentifier<M>,
}

pub const MAX_PERCENTAGE: u64 = 10_000u64;
pub const MAX_FEE_PERCENTAGE: u64 = 9_000u64;

#[elrond_wasm::module]
pub trait ConfigModule:
    permissions_module::PermissionsModule
    + pausable::PausableModule
{

    #[payable("*")]
    #[endpoint(topUp)]
    fn top_up(&self) {
        self.require_caller_has_owner_or_admin_permissions();
        let (token_id, _) = self.call_value().single_fungible_esdt();
        require!(token_id == self.wrapped_token().get(), "Invalid token");
    }

    #[endpoint(extractFees)]
    fn extract_fees(&self) {
        self.require_caller_has_owner_permissions();

        let owner = self.blockchain().get_caller();
        let wrapped_token = self.wrapped_token().get();
        let fee_amount = self.collected_fee_amount().get();
        self.send().direct_esdt(&owner, &wrapped_token, 0, &fee_amount);

        self.collected_fee_amount().set(BigUint::zero());
    }

    #[endpoint(setFeePercentage)]
    fn set_fee_percentage(&self, protocol_fee: u64) {
        self.require_caller_has_owner_permissions();
        require!(protocol_fee < MAX_FEE_PERCENTAGE, "Fee percent invalid");

        self.protocol_fee_percent().set(protocol_fee);
    }

    #[endpoint(setSlippagePercentage)]
    fn set_slippage_percentage(&self, slippage: u64) {
        self.require_caller_has_owner_or_admin_permissions();
        require!(slippage < MAX_FEE_PERCENTAGE, "Slippage percent invalid");

        self.slippage_percent().set(slippage);
    }

    #[endpoint(addKnownTokens)]
    fn add_known_tokens(&self, output_token: TokenIdentifier, known_tokens: MultiValueEncoded<AddKnownTokenType<Self::Api>>) {
        self.require_caller_has_owner_or_admin_permissions();

        let mut all_tokens_vec = self.all_tokens(&output_token).get();
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
                self.pair_contract(&token).set(PairContractData {
                    address: sc_address,
                    output_token: output_token.clone(),
                });
                self.token_threshold(&token).set(min_amount);
            }
        }
        self.all_tokens(&output_token).set(all_tokens_vec);
    }

    #[endpoint(removeKnownTokens)]
    fn remove_known_tokens(&self, output_token: TokenIdentifier, tokens: MultiValueEncoded<TokenIdentifier>) {
        self.require_caller_has_owner_or_admin_permissions();

        let mut all_tokens_vec = self.all_tokens(&output_token).get();
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
        self.all_tokens(&output_token).set(&all_tokens_vec);
    }

    #[view(getAllTokens)]
    fn get_all_tokens(&self, output_token: TokenIdentifier) -> MultiValueEncoded<TokenIdentifier> {
        self.all_tokens(&output_token).get().into()
    }

    #[storage_mapper("pair_contract")]
    fn pair_contract(&self, token_id: &TokenIdentifier) -> SingleValueMapper<PairContractData<Self::Api>>;

    #[view(getTokenThreshold)]
    #[storage_mapper("token_threshold")]
    fn token_threshold(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("known_tokens")]
    fn known_tokens(&self) -> WhitelistMapper<Self::Api, TokenIdentifier>;

    #[storage_mapper("all_tokens")]
    fn all_tokens(&self, output_token: &TokenIdentifier) -> SingleValueMapper<ManagedVec<TokenIdentifier>>;

    #[storage_mapper("token_buffer")]
    fn token_buffer(&self, token: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[storage_mapper("token_debt")]
    fn token_debt(&self, token: &TokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getProtocolFeePercent)]
    #[storage_mapper("protocol_fee_percent")]
    fn protocol_fee_percent(&self) -> SingleValueMapper<u64>;

    #[view(getSlippagePercent)]
    #[storage_mapper("slippage_percent")]
    fn slippage_percent(&self) -> SingleValueMapper<u64>;

    #[view(getWrappedTokenId)]
    #[storage_mapper("wrapped_token_id")]
    fn wrapped_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getUsdcTokenId)]
    #[storage_mapper("usdc_token_id")]
    fn usdc_token(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("collected_fee_amount")]
    fn collected_fee_amount(&self) -> SingleValueMapper<BigUint>;
}
