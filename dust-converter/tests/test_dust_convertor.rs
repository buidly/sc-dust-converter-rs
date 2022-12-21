mod contract_interactions;
use contract_interactions::*;
use dust_converter::{self, config::MAX_PERCENTAGE};
use elrond_wasm_debug::{rust_biguint, tx_mock::TxTokenTransfer};
use pair_mock::{self, ERR_TOKEN, AMOUNT_OUT};

static WRAPPED_TOKEN: &[u8] = b"WEGLD-0a3f5r";
static KNOWN_TOKEN_1: &[u8] = b"STK-2022a0";
static KNOWN_TOKEN_2: &[u8] = b"ASH-12345a";
static UNKOWN_TOKEN_3: &[u8] = b"UKN-1sy8n4";

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
    setup.resume();

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

    let amount_out = AMOUNT_OUT * 2u64;
    let fee = amount_out * 500u64 / MAX_PERCENTAGE;
    let total = amount_out - fee;

    let caller_address = setup.owner.clone();
    setup.swap_dust_token(&payments, &caller_address, total, None);

    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_1, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_2, &rust_biguint!(0u64));



    setup.b_wrapper.check_esdt_balance(&setup.owner, WRAPPED_TOKEN, &rust_biguint!(total));

}

#[test]
fn test_swap_dust_token_pair_fail() {
    let initial_err_amount = 1_000_000u64;
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(vec![ERR_TOKEN]);
    setup.resume();

    setup.b_wrapper.set_esdt_balance(&setup.owner, ERR_TOKEN, &rust_biguint!(initial_err_amount));

    let payments = [
        TxTokenTransfer {
            token_identifier: ERR_TOKEN.to_vec(),
            nonce: 0,
            value: rust_biguint!(initial_err_amount)
        },
    ];
    let caller_address = setup.owner.clone();
    setup.swap_dust_token(&payments, &caller_address, initial_err_amount, Some("Not enough reserve"));

    setup.b_wrapper.check_esdt_balance(&setup.owner, ERR_TOKEN, &rust_biguint!(initial_err_amount));
}

#[test]
fn test_sell_dust_tokens() {
    let token_1_amount = 3000000u64;
    let token_2_amount = 4000000u64;
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2]);
    setup.resume();

    let user = setup.b_wrapper.create_user_account(&rust_biguint!(0u64));

    setup.b_wrapper.set_esdt_balance(&user, KNOWN_TOKEN_1, &rust_biguint!(token_1_amount));
    setup.b_wrapper.set_esdt_balance(&user, KNOWN_TOKEN_2, &rust_biguint!(token_2_amount));

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

    let amount_out = AMOUNT_OUT * 2u64;
    let fee = amount_out * 500u64 / MAX_PERCENTAGE;
    let total = amount_out - fee;

    setup.swap_dust_token(&payments, &user, total, None);
    setup.sell_dust_token();

    setup.b_wrapper.check_esdt_balance(&user, KNOWN_TOKEN_1, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&user, KNOWN_TOKEN_2, &rust_biguint!(0u64));
}


#[test]
fn test_refund_unknown_tokens() {
    let known_token_amount = 3_000_000u64;
    let unkown_token_amount = 2_500_000u64;

    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2]);
    setup.resume();

    let user = setup.b_wrapper.create_user_account(&rust_biguint!(0u64));

    setup.b_wrapper.set_esdt_balance(&user, KNOWN_TOKEN_1, &rust_biguint!(known_token_amount));
    setup.b_wrapper.set_esdt_balance(&user, UNKOWN_TOKEN_3, &rust_biguint!(unkown_token_amount));

    let payments = [
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_1.to_vec(),
            nonce: 0,
            value: rust_biguint!(known_token_amount)
        },
        TxTokenTransfer {
            token_identifier: UNKOWN_TOKEN_3.to_vec(),
            nonce: 0,
            value: rust_biguint!(unkown_token_amount)
        }
    ];

    let amount_out = AMOUNT_OUT * 1u64;
    let fee = amount_out * 500u64 / MAX_PERCENTAGE;
    let total = amount_out - fee;

    setup.swap_dust_token(&payments, &user, total, None);

    setup.b_wrapper.check_esdt_balance(&user, KNOWN_TOKEN_1, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&user, UNKOWN_TOKEN_3, &rust_biguint!(unkown_token_amount));
}
