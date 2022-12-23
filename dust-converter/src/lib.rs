#![no_std]

use config::{MAX_PERCENTAGE, MAX_FEE_PERCENTAGE};

elrond_wasm::imports!();

pub mod config;
pub mod proxy;
use pausable::State;
use permissions_module::Permissions;

#[elrond_wasm::contract]
pub trait DustConverter:
    config::ConfigModule
    + proxy::ProxyModule
    + permissions_module::PermissionsModule
    + pausable::PausableModule
{

    #[init]
    fn init(&self, protocol_fee_percent: u64, slippage_percent: u64, wrapped_token: TokenIdentifier) {
        require!(
            protocol_fee_percent < MAX_FEE_PERCENTAGE,
            "Invalid protocol fee value"
        );
        self.protocol_fee_percent().set(protocol_fee_percent);

        require!(
            slippage_percent < MAX_PERCENTAGE,
            "Invalid slippage percent value"
        );
        self.slippage_percent().set(slippage_percent);

        require!(
            wrapped_token.is_valid_esdt_identifier(),
            "Not a valid esdt id"
        );
        self.wrapped_token().set_if_empty(wrapped_token);
        self.collected_fee_amount().set_if_empty(BigUint::zero());
        self.state().set(State::Inactive);

        let all_permissions = Permissions::OWNER | Permissions::ADMIN | Permissions::PAUSE;
        self.set_permissions(self.blockchain().get_caller(), all_permissions);
    }

    #[payable("*")]
    #[endpoint(swapDustTokens)]
    fn swap_dust_tokens(&self, amount_out_min: BigUint, tag: OptionalValue<ManagedBuffer>) {
        self.require_state_active();

        let payments = self.call_value().all_esdt_transfers();
        let known_tokens_mapper = self.known_tokens();
        let wrapped_egld = self.wrapped_token().get();

        let mut total_amount = BigUint::zero();
        let mut refund_payments = ManagedVec::new();
        for p in &payments {
            if !known_tokens_mapper.contains(&p.token_identifier) {
                refund_payments.push(p);
                continue;
            }

            let pair = self.pair_contract(&p.token_identifier).get();
            let value = self.get_amount_out(pair, p.token_identifier, p.amount);

            total_amount += &value;
        }

        let mut fee_amount = self.get_fee_from_input(&total_amount);
        total_amount -= &fee_amount;
        require!(total_amount >= amount_out_min, "Slippage exceeded");

        if let Some(tag_name) = tag.into_option() {
            fee_amount = self.subtract_referral_fee_and_update_collected_fees(fee_amount, tag_name);
        }

        let caller = self.blockchain().get_caller();
        if total_amount > 0 {
            self.send().direct_esdt(&caller, &wrapped_egld, 0, &total_amount);
        }
        if !refund_payments.is_empty() {
            self.send().direct_multi(&caller, &refund_payments);
        }

        self.collected_fee_amount().update(|x| *x += fee_amount);
    }

    #[endpoint(sellDustTokens)]
    fn sell_dust_tokens(&self, tokens_to_sell: MultiValueEncoded<TokenIdentifier>) {
        let wrapped_egld = self.wrapped_token().get();
        let known_tokens_mapper = self.known_tokens();
        for token in tokens_to_sell.into_iter() {
            if !known_tokens_mapper.contains(&token) {
                continue;
            }

            let pair = self.pair_contract(&token).get();
            let balance = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token.clone()), 0);
            if balance == BigUint::zero() {
                continue;
            }

            let value = self.get_amount_out(pair.clone(), token.clone(), balance.clone());
            let threshold = self.token_threshold(&token).get();
            if value > threshold {
                let amount_out_min = self.get_amount_out_min(&value);
                self.swap_tokens_fixed_input(pair, token, balance, wrapped_egld.clone(), amount_out_min);
            }
        }
    }

    #[endpoint(registerReferralTag)]
    fn register_referral_tag(&self, tag: ManagedBuffer) {
        let caller = self.blockchain().get_caller();
        require!(self.referral_tag_percent(&tag).is_empty(), "Tag already registered");

        self.referral_tag_percent(&tag).set(config::DEFAULT_REFERRAL_PERCENTAGE);
        self.user_tag_mapping(&caller).set(tag);
    }

    #[endpoint(removeReferralTag)]
    fn remove_referral_tag(&self, user_address: ManagedAddress) {
        self.require_caller_has_owner_or_admin_permissions();

        let wrapped_egld = self.wrapped_token().get();
        let tag = self.user_tag_mapping(&user_address).get();
        let collected_amount = self.collected_tag_fees(&tag).get();
        if collected_amount > 0 {
            self.send().direct_esdt(&user_address, &wrapped_egld, 0, &collected_amount);
        }

        self.referral_tag_percent(&tag).clear();
        self.collected_tag_fees(&tag).clear();
        self.user_tag_mapping(&user_address).clear();
    }

    fn subtract_referral_fee_and_update_collected_fees(&self, fee_amount: BigUint, tag: ManagedBuffer) -> BigUint {
        let tag_percentage = self.referral_tag_percent(&tag).get();
        let referral_amount = &fee_amount * tag_percentage / MAX_PERCENTAGE;
        self.collected_tag_fees(&tag).update(|x| *x += &referral_amount);

        fee_amount - referral_amount
    }

    #[inline]
    fn get_fee_from_input(&self, amount_in: &BigUint) -> BigUint {
        amount_in * self.protocol_fee_percent().get() / MAX_PERCENTAGE
    }

    #[inline]
    fn get_amount_out_min(&self, amount_in: &BigUint) -> BigUint {
        require!(!self.slippage_percent().is_empty(), "Slippage not set");
        let slippage = self.slippage_percent().get();
        let slippage_amount = amount_in * slippage / MAX_PERCENTAGE;

        amount_in.sub(&slippage_amount)
    }

}

