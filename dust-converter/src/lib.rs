#![no_std]

use crate::config::MAX_FEE_PERCENTAGE;

elrond_wasm::imports!();

pub mod config;
pub mod proxy;

#[elrond_wasm::contract]
pub trait EmptyContract:
    config::ConfigModule +
    proxy::ProxyModule 
{
    
    #[init]
    fn init(&self, protocol_fee_percent: u64, slippage_percent: u64, wrapped_token: TokenIdentifier) {
        require!(
            protocol_fee_percent < MAX_FEE_PERCENTAGE,
            "Invalid protocol fee value"
        );
        self.protocol_fee_percent().set(protocol_fee_percent);

        require!(
            slippage_percent < MAX_FEE_PERCENTAGE,
            "Invalid slippage percent value"
        );
        self.slippage_percent().set(slippage_percent);

        require!(
            wrapped_token.is_valid_esdt_identifier(),
            "Not a valid esdt id"
        );
        self.wrapped_token().set_if_empty(wrapped_token);
    }

    #[payable("*")]
    #[endpoint(swapDustTokens)]
    fn swap_dust_token(&self) {
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
            let mut value = self.get_amount_out(pair, p.token_identifier, p.amount);

            let fee_amount = self.get_fee_from_input(&value);
            value -= &fee_amount;
            total_amount += &value;
        }

        let caller = self.blockchain().get_caller();
        self.send().direct_esdt(&caller, &wrapped_egld, 0, &total_amount);
        if !refund_payments.is_empty() {
            self.send().direct_multi(&caller, &refund_payments);
        }
        self.wrapped_token_amount().update(|x| *x -= total_amount);
    }

    #[endpoint(sellDustTokens)]
    fn sell_dust_tokens(&self) {
        let wrapped_egld = self.wrapped_token().get();
        let all_tokens = self.all_tokens().get();
        for token in &all_tokens {
            let pair = self.pair_contract(&token).get();
            let balance = self.blockchain().get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(token.clone()), 0);
            if balance == BigUint::zero() {
                continue;
            }

            let value = self.get_amount_out(pair.clone(), token.clone(), balance.clone());

            let threshold = self.token_threshold(&token).get();
            if &value > &threshold {
                let amount_out_min = self.get_amount_out_min(&value);
                self.swap_tokens_fixed_input(pair, token, balance, wrapped_egld.clone(), amount_out_min);
            }
        }
    }

    #[inline]
    fn get_fee_from_input(&self, amount_in: &BigUint) -> BigUint {
        amount_in * self.protocol_fee_percent().get() / MAX_FEE_PERCENTAGE
    }

    #[inline]
    fn get_amount_out_min(&self, amount_in: &BigUint) -> BigUint {
        require!(!self.slippage_percent().is_empty(), "Slippage not set");
        let slippage = self.slippage_percent().get();
        let slippage_amount = amount_in * &BigUint::from(slippage) / MAX_FEE_PERCENTAGE;

        let amount_out_min = amount_in - &slippage_amount;

        amount_out_min
    }

}

