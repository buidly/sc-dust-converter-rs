#![no_std]

use config::{MAX_PERCENTAGE, MAX_FEE_PERCENTAGE};

elrond_wasm::imports!();

pub mod config;
pub mod proxy;
pub mod referral;
use pausable::State;
use permissions_module::Permissions;

#[elrond_wasm::contract]
pub trait DustConverter:
    config::ConfigModule
    + proxy::ProxyModule
    + referral::ReferralModule
    + permissions_module::PermissionsModule
    + pausable::PausableModule
{

    #[init]
    fn init(
        &self, 
        protocol_fee_percent: u64, 
        slippage_percent: u64, 
        wegld_token: TokenIdentifier,
        usdc_token: TokenIdentifier
    ) {
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
            wegld_token.is_valid_esdt_identifier(),
            "Not a valid esdt id"
        );
        require!(
            usdc_token.is_valid_esdt_identifier(),
            "Not a valid esdt id"
        );
        self.wrapped_token().set_if_empty(wegld_token);
        self.usdc_token().set_if_empty(usdc_token);
        self.collected_fee_amount().set_if_empty(BigUint::zero());
        self.state().set(State::Inactive);

        let all_permissions = Permissions::OWNER | Permissions::ADMIN | Permissions::PAUSE;
        self.set_permissions(self.blockchain().get_caller(), all_permissions);
    }

    fn compute_swap_amount(
        &self, 
        output_token: TokenIdentifier, 
        payments: &ManagedVec<EsdtTokenPayment>
    ) -> (BigUint, ManagedVec<EsdtTokenPayment>) {
        if payments.is_empty() {
            return (BigUint::zero(), ManagedVec::new());
        }

        let known_tokens_mapper = self.known_tokens();

        let mut total_amount = BigUint::zero();
        let mut refund_payments = ManagedVec::new();
        for p in payments {
            if !known_tokens_mapper.contains(&p.token_identifier) {
                refund_payments.push(p);
                continue;
            }

            let pair = self.pair_contract(&p.token_identifier).get();
            require!(pair.output_token == output_token, "Invalid payments");

            let value = self.get_amount_out(pair.address, p.token_identifier, p.amount);
            total_amount += value;
        }

        (total_amount, refund_payments)
    }

    fn add_usdc_to_first_payment(&self, payments: &mut ManagedVec<EsdtTokenPayment>, amount: BigUint) {
        if amount == BigUint::zero() {
            return;
        }

        let usdc_token = self.usdc_token().get();
        if payments.is_empty() {
            payments.push(EsdtTokenPayment::new(usdc_token, 0, amount));
            return;
        } 

        let first_payment = payments.get(0);
        if first_payment.token_identifier != usdc_token {
            payments.push(EsdtTokenPayment::new(usdc_token, 0, amount));
            return;
        }

        let result = payments.set(0, &EsdtTokenPayment::new(usdc_token.clone(), 0, first_payment.amount + amount.clone()));
        if result.is_err() {
            payments.push(EsdtTokenPayment::new(usdc_token, 0, amount));
        }
    }

    /// Receives a MultiEsdtNftTransfer and swaps the tokens to WEGLD. First, swaps all the tokens for WEGLD.
    /// After that, computes the protocol fee from the resulted amount. If a referral tag is used, the referral cut is also computed.
    /// Any user will be able to call this endpoint. Arguments:
    /// num_wegld - The first num_wegld payments will be swapped to WEGD
    /// amount_out_min - The minimum amount of WEGLD that the user wants to receive
    /// tag - The tag of the referral
    #[payable("*")]
    #[endpoint(swapDustTokens)]
    fn swap_dust_tokens(&self, num_wegld: usize, amount_out_min: BigUint, tag: Option<ManagedBuffer>, token_wanted: OptionalValue<TokenIdentifier>) {
        self.require_state_active();

        let payments = self.call_value().all_esdt_transfers();
        let num_payments = payments.len();
        require!(num_wegld <= num_payments, "Invalid num_wegld");

        let mut wegld_swaps = payments.slice(0, num_wegld).unwrap_or_else(ManagedVec::new);
        let usdc_swaps = payments.slice(num_wegld, num_payments).unwrap_or_else(ManagedVec::new);

        let (usdc_amount, usdc_refund) = self.compute_swap_amount(self.usdc_token().get(), &usdc_swaps);
        self.add_usdc_to_first_payment(&mut wegld_swaps, usdc_amount);

        let wrapped_egld = self.wrapped_token().get();
        let (total_amount, mut wegld_refund) = self.compute_swap_amount(wrapped_egld.clone(), &wegld_swaps);

        let mut fee_amount = self.get_fee_from_input(&total_amount);
        let amount_after_fees = &total_amount - &fee_amount;
        require!(amount_after_fees >= amount_out_min, "Slippage exceeded");

        if let Some(tag_name) = tag {
            self.accumulated_volume(&tag_name).update(|x| *x += total_amount);
            fee_amount = self.subtract_referral_fee_and_update_collected_fees(fee_amount, tag_name);
        }

        let caller = self.blockchain().get_caller();
        require!(amount_after_fees > 0, "Zero amount cannot be claimed");
        
        if let Some(token_wanted) = token_wanted.into_option() {
            let token_buffer = self.token_buffer(&token_wanted).get();
            let pair = self.pair_contract(&token_wanted).get();
            let value = self.get_amount_out(pair.clone().address, wrapped_egld.clone(), amount_after_fees.clone());
            require!(value < token_buffer, "Not enough token reserve");
            self.send().direct_esdt(&caller, &token_wanted, 0, &value);
        } else {
            self.send().direct_esdt(&caller, &wrapped_egld, 0, &amount_after_fees);
        }

        wegld_refund.extend(&usdc_refund);
        if !wegld_refund.is_empty() {
            self.send().direct_multi(&caller, &wegld_refund);
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
            let mut difference = balance.clone() - self.token_buffer(&token).get();
            if difference == BigUint::zero() { // balance - buffer(token) == BigUint::zero()
                continue;
            }
            let mut debt = self.token_debt(&token).get();
            if debt > 0 {
                if debt >= difference {
                    debt -= difference;

                    let value = self.get_amount_in(pair.clone().address, token.clone(), debt.clone());
                    let threshold = self.token_threshold(&token).get();
                    if value > threshold {
                        let amount_in_max = self.get_amount_in_max(&value);
                        self.swap_tokens_fixed_output(pair.address, wrapped_egld.clone(), amount_in_max, token.clone(), debt); 
                    }
                    self.token_debt(&token).clear();
                    continue;
                } else {
                    difference -= debt;
                    self.token_debt(&token).clear();
                }
            }
            let value = self.get_amount_out(pair.clone().address, token.clone(), balance.clone());
            let threshold = self.token_threshold(&token).get();
            if value > threshold {
                let amount_out_min = self.get_amount_out_min(&value);
                self.swap_tokens_fixed_input(pair.address, token, balance, wrapped_egld.clone(), amount_out_min); // balance - buffer(token)
            }
        }
    }

    #[inline]
    fn get_fee_from_input(&self, amount_in: &BigUint) -> BigUint {
        amount_in * self.protocol_fee_percent().get() / MAX_PERCENTAGE
    }

    #[inline]
    fn get_amount_out_min(&self, amount_out: &BigUint) -> BigUint {
        require!(!self.slippage_percent().is_empty(), "Slippage not set");
        let slippage = self.slippage_percent().get();
        let slippage_amount = amount_out * slippage / MAX_PERCENTAGE;

        amount_out.sub(&slippage_amount)
    }

    #[inline]
    fn get_amount_in_max(&self, amount_in: &BigUint) -> BigUint {
        require!(!self.slippage_percent().is_empty(), "Slippage not set");
        let slippage = self.slippage_percent().get();
        let slippage_amount = amount_in * slippage / MAX_PERCENTAGE;

        amount_in.add(&slippage_amount)
    }

}

