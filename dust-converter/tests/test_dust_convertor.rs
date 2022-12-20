mod contract_interactions;
use contract_interactions::*;
use dust_converter::{self, config::MAX_FEE_PERCENTAGE};
use elrond_wasm_debug::{rust_biguint, tx_mock::TxTokenTransfer};
use pair_mock::{self, ERR_TOKEN, AMOUNT_OUT};

static WRAPPED_TOKEN: &[u8] = b"WEGLD-0a3f5r";
static KNOWN_TOKEN_1: &[u8] = b"STK-2022a0";
static KNOWN_TOKEN_2: &[u8] = b"ASH-12345a";

#[test]
fn deploy() {
    DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, pair_mock::contract_obj);
}

#[test]
fn test_swap_dust_token_success() {
    let token_1_amount = 3000000u64;
    let token_2_amount = 4000000u64;
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2]);

    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_1, &rust_biguint!(token_1_amount));
    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_2, &rust_biguint!(token_2_amount));

    let payments = [
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_1.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_1_amount)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_2.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_2_amount)
        }
    ];
    setup.swap_dust_token(&payments, None);

    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_1, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_2, &rust_biguint!(0u64));

    let amount_out = AMOUNT_OUT * 2u64;
    let fee = amount_out * 500u64 / MAX_FEE_PERCENTAGE;
    let total = amount_out - fee;

    setup.b_wrapper.check_esdt_balance(&setup.owner, WRAPPED_TOKEN, &rust_biguint!(total));

}

#[test]
fn test_swap_dust_token_pair_fail() {
    let initial_err_amount = 1_000_000u64;
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(vec![ERR_TOKEN]);

    setup.b_wrapper.set_esdt_balance(&setup.owner, ERR_TOKEN, &rust_biguint!(initial_err_amount));

    let payments = [
        TxTokenTransfer {
            token_identifier: ERR_TOKEN.to_vec(),
            nonce: 0,
            value: rust_biguint!(3u64)
        },
    ];
    setup.swap_dust_token(&payments, Some("Not enough reserve"));

    setup.b_wrapper.check_esdt_balance(&setup.owner, ERR_TOKEN, &rust_biguint!(initial_err_amount));
}

