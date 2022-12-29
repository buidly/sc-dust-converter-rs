use crate::config::{MAX_FEE_PERCENTAGE, MAX_PERCENTAGE, self};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

pub const DEFAULT_REFERRAL_PERCENTAGE: u64 = 500u64;

#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone, Debug)]
pub struct TierDetails<M: ManagedTypeApi>  {
    pub name: ManagedBuffer<M>,
    pub min_volume: BigUint<M>,
    pub fee_percent: u64,
}

#[elrond_wasm::module]
pub trait ReferralModule:
    permissions_module::PermissionsModule
    + pausable::PausableModule
    + config::ConfigModule
{

    #[endpoint(registerReferralTag)]
    fn register_referral_tag(&self, tag: ManagedBuffer) {
        self.require_state_active();

        let caller = self.blockchain().get_caller();
        require!(self.referral_tag_percent(&tag).is_empty(), "Tag already registered");
        require!(self.user_tag_mapping(&caller).is_empty(), "User already owns a tag");
        require!(!self.tier_details().is_empty(), "Tiers not set");
        
        self.user_tag_mapping(&caller).set(tag.clone());

        for tier in self.tier_details().iter() {
            if tier.min_volume == BigUint::zero() {
                self.referral_tag_percent(&tag).set(tier.fee_percent);
                return;
            }
        }
        sc_panic!("No tier with min volume 0");
    }

    #[endpoint(claimReferralFees)]
    fn claim_referral_fees(&self) {
        self.require_state_active();
        
        let caller = self.blockchain().get_caller();
        require!(!self.user_tag_mapping(&caller).is_empty(), "Not a tag owner");
        let user_tag = self.user_tag_mapping(&caller).get();

        let amount = self.collected_tag_fees(&user_tag).get();
        require!(amount > 0, "No fees to claim");

        self.send().direct_esdt(&caller, &self.wrapped_token().get(), 0, &amount);
        self.collected_tag_fees(&user_tag).clear();
    }

    #[endpoint(updateUserTier)]
    fn update_user_tier(&self) -> ManagedBuffer {
        self.require_state_active();

        let caller = self.blockchain().get_caller();
        require!(!self.user_tag_mapping(&caller).is_empty(), "Not a tag owner");

        let user_tag = self.user_tag_mapping(&caller).get();
        let user_volume = self.accumulated_volume(&user_tag).get();
        let mut tier_name = ManagedBuffer::new();
        let mut fee_percent = self.referral_tag_percent(&user_tag).get();

        for tier in self.tier_details().iter() {
            if user_volume >= tier.min_volume && tier.fee_percent > fee_percent {
                fee_percent = tier.fee_percent;
                tier_name = tier.name;
            }
        }
        
        require!(fee_percent != self.referral_tag_percent(&user_tag).get(), "No tier upgrade found");
        self.referral_tag_percent(&user_tag).set(fee_percent);
        
        tier_name
    }

    #[endpoint(addTierDetails)]
    fn add_tier_details(&self, name: ManagedBuffer, min_volume: BigUint, fee_percent: u64) {
        self.require_caller_has_owner_permissions();
        require!(fee_percent < MAX_FEE_PERCENTAGE, "Invalid fee percentage");
        let is_new = self.tier_details().insert(TierDetails {
            name,
            min_volume,
            fee_percent,
        });

        require!(is_new, "Tier already exists");
    }


    #[endpoint(removeTierDetails)]
    fn remove_tier_details(&self, name: ManagedBuffer) {
        self.require_caller_has_owner_permissions();
        let mut tier_details = self.tier_details();
        
        for tier in tier_details.iter() {
            if tier.name == name {
                tier_details.swap_remove(&tier);
                return;
            }
        }

        sc_panic!("Tier not found");
    }

    #[endpoint(setReferralFeePercentage)]
    fn set_referral_fee_percentage(&self, tag: ManagedBuffer, new_percentage: u64) {
        self.require_caller_has_owner_or_admin_permissions();
        require!(new_percentage < MAX_FEE_PERCENTAGE, "Invalid new percentage given");
        require!(!self.referral_tag_percent(&tag).is_empty(), "Tag not found");
        self.referral_tag_percent(&tag).set(new_percentage);
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

        self.accumulated_volume(&tag).clear();
        self.referral_tag_percent(&tag).clear();
        self.collected_tag_fees(&tag).clear();
        self.user_tag_mapping(&user_address).clear();
    }

    #[view(getCollectedFeeAmount)]
    fn get_collected_fee_amount(
        &self,
        address: ManagedAddress
    ) -> BigUint {
        let tag_mapping = self.user_tag_mapping(&address);

        self.collected_tag_fees(&tag_mapping.get()).get()
    }


    fn subtract_referral_fee_and_update_collected_fees(&self, fee_amount: BigUint, tag: ManagedBuffer) -> BigUint {
        let tag_percentage = self.referral_tag_percent(&tag).get();
        if tag_percentage == 0 {
            return fee_amount;
        }
        
        let referral_amount = &fee_amount * tag_percentage / MAX_PERCENTAGE;
        self.collected_tag_fees(&tag).update(|x| *x += &referral_amount);

        fee_amount - referral_amount
    }

    #[view(getReferralFeePercentage)]
    #[storage_mapper("referral_tags_percent")]
    fn referral_tag_percent(&self, tag: &ManagedBuffer) -> SingleValueMapper<u64>;

    #[view(getTierDetails)]
    #[storage_mapper("tier_details")]
    fn tier_details(&self) -> UnorderedSetMapper<TierDetails<Self::Api>>;

    #[view(getTagAccumulatedVolume)]
    #[storage_mapper("accumulated_volume")]
    fn accumulated_volume(&self, tag: &ManagedBuffer) -> SingleValueMapper<BigUint>;
}