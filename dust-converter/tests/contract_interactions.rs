use std::fmt::Debug;

use elrond_wasm::{
    types::{Address, MultiValueEncoded, TokenIdentifier},
    elrond_codec::multi_types::MultiValue3
};
use elrond_wasm_debug::{
    DebugApi,
    testing_framework::{BlockchainStateWrapper, ContractObjWrapper},
    managed_token_id,
    rust_biguint,
    managed_biguint,
    managed_address,
    tx_mock::{TxTokenTransfer}
};

static DUST_WASM_PATH: &str = "../output/dust-converter.wasm";

use dust_converter;
use dust_converter::{
    DustConverter,
    config::ConfigModule
};
use pausable::PausableModule;


pub struct DustConvertorSetup<DustBuilder, MockBuilder>
where
    DustBuilder: 'static + Copy + Fn() -> dust_converter::ContractObj<DebugApi>,
    MockBuilder: 'static + Copy + Fn() -> pair_mock::ContractObj<DebugApi>,
{
    pub b_wrapper: BlockchainStateWrapper,
    pub owner: Address,
    pub c_wrapper: ContractObjWrapper<dust_converter::ContractObj<DebugApi>, DustBuilder>,
    pub pair_wrapper: ContractObjWrapper<pair_mock::ContractObj<DebugApi>, MockBuilder>
}

impl<DustBuilder, MockBuilder> DustConvertorSetup<DustBuilder, MockBuilder>
where
    DustBuilder: 'static + Copy + Fn() -> dust_converter::ContractObj<DebugApi>,
    MockBuilder: 'static + Copy + Fn() -> pair_mock::ContractObj<DebugApi>,
{
    pub fn new(dust_builder: DustBuilder, wrapped_token: &[u8], pair_builder: MockBuilder) -> Self {
        let rust_zero = rust_biguint!(0);
        let initial_sc_balance = rust_biguint!(10_000_000_000_000_000_000u64);
        let mut b_wrapper = BlockchainStateWrapper::new();
        let owner = b_wrapper.create_user_account(&rust_zero);
        b_wrapper.set_esdt_balance(&owner, wrapped_token, &initial_sc_balance);

        let contract_wrapper = b_wrapper.create_sc_account(
            &rust_zero,
            Some(&owner),
            dust_builder,
            DUST_WASM_PATH,
        );

        let pair_wrapper = b_wrapper.create_sc_account(
            &rust_zero,
            Some(&owner),
            pair_builder,
            "mocked wasm"
        );
        b_wrapper.set_esdt_balance(&pair_wrapper.address_ref(), wrapped_token, &initial_sc_balance);

        b_wrapper
            .execute_tx(&owner, &contract_wrapper, &rust_zero, |sc| {
                sc.init(
                    500u64,
                    50u64,
                    managed_token_id!(wrapped_token)
                );
            })
            .assert_ok();

        b_wrapper
            .execute_esdt_transfer(&owner, &contract_wrapper, wrapped_token, 0, &initial_sc_balance, |sc| {
                sc.top_up();
            })
            .assert_ok();

        Self {
            b_wrapper,
            owner,
            c_wrapper: contract_wrapper,
            pair_wrapper
        }
    }

    pub fn add_known_tokens(&mut self, known_tokens: Vec<&[u8]>) {
        let p_wrapper = self.pair_wrapper.address_ref();
        self.b_wrapper
            .execute_tx(&self.owner, &self.c_wrapper, &rust_biguint!(0u64), |sc| {
                let mut payload_tokens = MultiValueEncoded::new();
                for t in known_tokens {
                    payload_tokens.push(MultiValue3(
                        (
                            managed_token_id!(t),
                            managed_address!(p_wrapper),
                            managed_biguint!(2u64))
                        )
                    );

                }
                sc.add_known_tokens(payload_tokens);
            })
            .assert_ok();
    }

    pub fn swap_dust_token(
        &mut self,
        payments: &[TxTokenTransfer],
        caller: &Address,
        min_out_amount: u64,
        expected_err: Option<&str>
    ) {
        let tx = self.b_wrapper
            .execute_esdt_multi_transfer(&caller, &self.c_wrapper, &payments, |sc|{
                sc.swap_dust_tokens(managed_biguint!(min_out_amount));
            });

        if let Some(msg) = expected_err {
            tx.assert_error(4, msg);
            return
        }

        tx.assert_ok()
    }

    pub fn sell_dust_token(&mut self, tokens: Vec<&[u8]>) {
        self.b_wrapper
            .execute_tx(&self.owner, &self.c_wrapper, &rust_biguint!(0u64), |sc| {
                let mut multi = MultiValueEncoded::new();
                for token in tokens {
                    multi.push(managed_token_id!(token));
                }

                sc.sell_dust_tokens(multi);
            })
            .assert_ok();
    }

    pub fn resume(&mut self) {
        self.b_wrapper
            .execute_tx(&self.owner, &self.c_wrapper, &rust_biguint!(0u64), |sc| {
                sc.resume();
            })
            .assert_ok();
    }
}

