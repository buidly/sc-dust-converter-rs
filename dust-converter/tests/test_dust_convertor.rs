mod contract_interactions;
use contract_interactions::*;
use dust_converter::{self, config::{MAX_PERCENTAGE}};
use elrond_wasm_debug::{rust_biguint, tx_mock::TxTokenTransfer};
use pair_mock::{self, ERR_TOKEN, AMOUNT_OUT, KNOWN_TOKEN_1, KNOWN_TOKEN_2, KNOWN_TOKEN_3, KNOWN_TOKEN_4, KNOWN_TOKEN_5};

static WRAPPED_TOKEN: &[u8] = b"WEGLD-0a3f5r";
static USDC_TOKEN: &[u8] = b"USDC-0a3f5r";
pub const UNKOWN_TOKEN_3: &[u8] = b"UKN-1sy8n4";

pub const TOKEN_1_RATE_PERCENTAGE: u64 = 400; //   1000 TOKEN1 = 40 TOKEN_OUT
pub const TOKEN_2_RATE_PERCENTAGE: u64 = 30; //    1000 TOKEN2 = 3 TOKEN_OUT
pub const TOKEN_3_RATE_PERCENTAGE: u64 = 150; //   1000 TOKEN3 = 15 TOKEN_OUT
pub const TOKEN_4_RATE_PERCENTAGE: u64 = 2_300; // 1000 TOKEN4 = 230 TOKEN_OUT
pub const TOKEN_5_RATE_PERCENTAGE: u64 = 4_670; // 1000 TOKEN5 = 467 TOKEN_OUT

fn compute_token_out_amount(token_in_amount: u64, token_id: &[u8]) -> u64 {
    match token_id {
        KNOWN_TOKEN_1 => token_in_amount * TOKEN_1_RATE_PERCENTAGE / MAX_PERCENTAGE,
        KNOWN_TOKEN_2 => token_in_amount * TOKEN_2_RATE_PERCENTAGE / MAX_PERCENTAGE,
        KNOWN_TOKEN_3 => token_in_amount * TOKEN_3_RATE_PERCENTAGE / MAX_PERCENTAGE,
        KNOWN_TOKEN_4 => token_in_amount * TOKEN_4_RATE_PERCENTAGE / MAX_PERCENTAGE,
        KNOWN_TOKEN_5 => token_in_amount * TOKEN_5_RATE_PERCENTAGE / MAX_PERCENTAGE,
        _ => AMOUNT_OUT,
    }
}

#[test]
fn deploy() {
    DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
}

#[test]
fn test_swap_dust_token_wegld_success() {
    let token_1_amount = 3000000u64;
    let token_2_amount = 4000000u64;
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(WRAPPED_TOKEN, vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2]);
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

    let amount_out = compute_token_out_amount(token_1_amount, KNOWN_TOKEN_1) + compute_token_out_amount(token_2_amount, KNOWN_TOKEN_2);
    let fee = amount_out * 500u64 / MAX_PERCENTAGE;
    let total = amount_out - fee;

    let caller_address = setup.owner.clone();
    setup.swap_dust_token(&payments, &caller_address, payments.len(), total, None, None);

    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_1, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_2, &rust_biguint!(0u64));



    setup.b_wrapper.check_esdt_balance(&setup.owner, WRAPPED_TOKEN, &rust_biguint!(total));
}

#[test]
fn test_swap_dust_tokens_usdc_success () {
    let token_2_amount = 3000000u64;
    let token_3_amount = 4000000u64;
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(WRAPPED_TOKEN, vec![KNOWN_TOKEN_1]);
    setup.add_known_tokens(USDC_TOKEN, vec![KNOWN_TOKEN_2, KNOWN_TOKEN_3]);
    setup.resume();

    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_2, &rust_biguint!(token_2_amount));
    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_3, &rust_biguint!(token_3_amount));

    let payments = [
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_2.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_2_amount)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_3.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_3_amount)
        }
    ];

    let usdc_amount = compute_token_out_amount(token_2_amount, KNOWN_TOKEN_2) + compute_token_out_amount(token_3_amount, KNOWN_TOKEN_3);
    let wegld_amount = compute_token_out_amount(usdc_amount, USDC_TOKEN);
    let fee = wegld_amount * 500u64 / MAX_PERCENTAGE;
    let total = wegld_amount - fee;

    let caller_address = setup.owner.clone();
    setup.swap_dust_token(&payments, &caller_address, 0, total, None, None);

    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_2, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_3, &rust_biguint!(0u64));

    setup.b_wrapper.check_esdt_balance(&setup.owner, WRAPPED_TOKEN, &rust_biguint!(total));
}

#[test]
fn test_swap_dust_tokens_usdc_wegld_success () {
    let token_1_amount = 3000000u64;
    let token_2_amount = 3000000u64;
    let token_3_amount = 4000000u64;
    let token_4_amount = 5000000u64;
    let token_5_amount = 5000000u64;
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(WRAPPED_TOKEN, vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2, KNOWN_TOKEN_3]);
    setup.add_known_tokens(USDC_TOKEN, vec![KNOWN_TOKEN_4, KNOWN_TOKEN_5]);
    setup.resume();

    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_1, &rust_biguint!(token_1_amount));
    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_2, &rust_biguint!(token_2_amount));
    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_3, &rust_biguint!(token_3_amount));
    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_4, &rust_biguint!(token_4_amount));
    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_5, &rust_biguint!(token_5_amount));

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
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_3.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_3_amount)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_4.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_4_amount)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_5.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_5_amount)
        }
    ];

    let usdc_tokens_amount = compute_token_out_amount(token_4_amount, KNOWN_TOKEN_4) + compute_token_out_amount(token_5_amount, KNOWN_TOKEN_5);
    let amount_out = compute_token_out_amount(token_1_amount + usdc_tokens_amount, KNOWN_TOKEN_1) 
        + compute_token_out_amount(token_2_amount, KNOWN_TOKEN_2) + compute_token_out_amount(token_3_amount, KNOWN_TOKEN_3);
    let fee = amount_out * 500u64 / MAX_PERCENTAGE;
    let total = amount_out - fee;

    let caller_address = setup.owner.clone();
    setup.swap_dust_token(&payments, &caller_address, 3, total, None, None);

    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_1, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_2, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_3, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_4, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_5, &rust_biguint!(0u64));

    setup.b_wrapper.check_esdt_balance(&setup.owner, WRAPPED_TOKEN, &rust_biguint!(total));

}

#[test]
fn test_swap_dust_tokens_usdc_wegld_refund_tokens () {
    let unknown_token_amount = 3000000u64;
    let token_2_amount = 3000000u64;
    let token_3_amount = 4000000u64;
    let token_4_amount = 5000000u64;
    let token_5_amount = 5000000u64;
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(WRAPPED_TOKEN, vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2, KNOWN_TOKEN_3]);
    setup.add_known_tokens(USDC_TOKEN, vec![KNOWN_TOKEN_4, KNOWN_TOKEN_5]);
    setup.resume();

    setup.b_wrapper.set_esdt_balance(&setup.owner, UNKOWN_TOKEN_3, &rust_biguint!(unknown_token_amount));
    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_2, &rust_biguint!(token_2_amount));
    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_3, &rust_biguint!(token_3_amount));
    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_4, &rust_biguint!(token_4_amount));
    setup.b_wrapper.set_esdt_balance(&setup.owner, KNOWN_TOKEN_5, &rust_biguint!(token_5_amount));

    let payments = [
        TxTokenTransfer {
            token_identifier: UNKOWN_TOKEN_3.to_vec(),
            nonce: 0,
            value: rust_biguint!(unknown_token_amount)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_2.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_2_amount)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_3.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_3_amount)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_4.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_4_amount)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_5.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_5_amount)
        }
    ];

    let usdc_tokens_amount = compute_token_out_amount(token_4_amount, KNOWN_TOKEN_4) + compute_token_out_amount(token_5_amount, KNOWN_TOKEN_5);
    let amount_out = compute_token_out_amount(usdc_tokens_amount, USDC_TOKEN) 
        + compute_token_out_amount(token_2_amount, KNOWN_TOKEN_2) + compute_token_out_amount(token_3_amount, KNOWN_TOKEN_3);
    let fee = amount_out * 500u64 / MAX_PERCENTAGE;
    let total = amount_out - fee;

    let caller_address = setup.owner.clone();
    setup.swap_dust_token(&payments, &caller_address, 3, total, None, None);

    setup.b_wrapper.check_esdt_balance(&setup.owner, UNKOWN_TOKEN_3, &rust_biguint!(unknown_token_amount));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_2, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_3, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_4, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&setup.owner, KNOWN_TOKEN_5, &rust_biguint!(0u64));

    setup.b_wrapper.check_esdt_balance(&setup.owner, WRAPPED_TOKEN, &rust_biguint!(total));

}

#[test]
fn test_swap_dust_token_pair_fail() {
    let initial_err_amount = 1_000_000u64;
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(WRAPPED_TOKEN, vec![ERR_TOKEN]);
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
    setup.swap_dust_token(&payments, &caller_address, payments.len(), initial_err_amount, Some("Not enough reserve"), None);

    setup.b_wrapper.check_esdt_balance(&setup.owner, ERR_TOKEN, &rust_biguint!(initial_err_amount));
}

#[test]
fn test_sell_dust_tokens() {
    let token_1_amount = 3000000u64;
    let token_2_amount = 4000000u64;
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(WRAPPED_TOKEN, vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2]);
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

    let amount_out = compute_token_out_amount(token_1_amount, KNOWN_TOKEN_1) + compute_token_out_amount(token_2_amount, KNOWN_TOKEN_2);
    let fee = amount_out * 500u64 / MAX_PERCENTAGE;
    let total = amount_out - fee;

    setup.swap_dust_token(&payments, &user, payments.len(), total, None, None);
    setup.sell_dust_token(vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2]);

    setup.b_wrapper.check_esdt_balance(&user, KNOWN_TOKEN_1, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&user, KNOWN_TOKEN_2, &rust_biguint!(0u64));
}


#[test]
fn test_refund_unknown_tokens() {
    let known_token_amount = 3_000_000u64;
    let unkown_token_amount = 2_500_000u64;

    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(WRAPPED_TOKEN, vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2]);
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

    let amount_out = compute_token_out_amount(known_token_amount, KNOWN_TOKEN_1);
    let fee = amount_out * 500u64 / MAX_PERCENTAGE;
    let total = amount_out - fee;

    setup.swap_dust_token(&payments, &user, payments.len(), total, None, None);

    setup.b_wrapper.check_esdt_balance(&user, KNOWN_TOKEN_1, &rust_biguint!(0u64));
    setup.b_wrapper.check_esdt_balance(&user, UNKOWN_TOKEN_3, &rust_biguint!(unkown_token_amount));
}

#[test]
fn test_register_referral_tag() {
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.resume();

    let user = setup.b_wrapper.create_user_account(&rust_biguint!(0u64));
    let tag = "TEST5".to_string();
    setup.register_referral_tag(&user, tag.as_bytes());
    setup.check_registered_tags(tag.as_bytes(), &user);
    setup.check_referral_fee_percentage(TIER_1_FEE_PERCENT, tag.as_bytes());
}

#[test]
fn test_update_referral_tag_percent() {
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.resume();

    let user = setup.b_wrapper.create_user_account(&rust_biguint!(0u64));
    let tag = "TEST5".to_string();
    setup.register_referral_tag(&user, tag.as_bytes());

    setup.check_referral_fee_percentage(500u64, tag.as_bytes());
    setup.set_referral_fee_percentage(1000u64, tag.as_bytes());
    setup.check_referral_fee_percentage(1000u64, tag.as_bytes());
}

#[test]
fn test_swap_token_with_referral_tag() {
    let known_token_amount_1 = 3_000_000u64;
    let known_token_amount_2 = 2_500_000u64;
    let tag = b"TEST5";

    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(WRAPPED_TOKEN, vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2]);
    setup.resume();
    let user_1 = setup.b_wrapper.create_user_account(&rust_biguint!(0u64));
    setup.register_referral_tag(&user_1, tag);

    let user_2 = setup.b_wrapper.create_user_account(&rust_biguint!(0u64));
    setup.b_wrapper.set_esdt_balance(&user_2, KNOWN_TOKEN_1, &rust_biguint!(known_token_amount_1));
    setup.b_wrapper.set_esdt_balance(&user_2, KNOWN_TOKEN_2, &rust_biguint!(known_token_amount_2));
    let payments = [
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_1.to_vec(),
            nonce: 0,
            value: rust_biguint!(known_token_amount_1)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_2.to_vec(),
            nonce: 0,
            value: rust_biguint!(known_token_amount_2)
        }
    ];

    let amount_out = compute_token_out_amount(known_token_amount_1, KNOWN_TOKEN_1) + compute_token_out_amount(known_token_amount_2, KNOWN_TOKEN_2);
    let fee = amount_out * TIER_1_FEE_PERCENT / MAX_PERCENTAGE;
    let referral_fee = fee * TIER_1_FEE_PERCENT / MAX_PERCENTAGE;
    let total = amount_out - fee;
    setup.swap_dust_token(&payments, &user_2, payments.len(), total, None, Some(tag));
    setup.check_referral_fee_amount(tag, referral_fee);

    setup.remove_referral_tag(&user_1, None);
    setup.b_wrapper.check_esdt_balance(&user_1, WRAPPED_TOKEN, &rust_biguint!(referral_fee));
}

#[test]
fn test_accumulate_volume_and_update_tier() {
    let known_token_amount_1 = 3_000_000u64;
    let known_token_amount_2 = 2_500_000u64;
    let tag = b"TEST5";

    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(WRAPPED_TOKEN, vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2]);
    setup.resume();
    let user_1 = setup.b_wrapper.create_user_account(&rust_biguint!(0u64));
    setup.register_referral_tag(&user_1, tag);

    let user_2 = setup.b_wrapper.create_user_account(&rust_biguint!(0u64));
    setup.b_wrapper.set_esdt_balance(&user_2, KNOWN_TOKEN_1, &rust_biguint!(known_token_amount_1));
    setup.b_wrapper.set_esdt_balance(&user_2, KNOWN_TOKEN_2, &rust_biguint!(known_token_amount_2));
    let payments = [
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_1.to_vec(),
            nonce: 0,
            value: rust_biguint!(known_token_amount_1 / 5)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_2.to_vec(),
            nonce: 0,
            value: rust_biguint!(known_token_amount_2 / 5)
        }
    ];

    let amount_out = compute_token_out_amount(known_token_amount_1 / 5, KNOWN_TOKEN_1) + compute_token_out_amount(known_token_amount_2 / 5, KNOWN_TOKEN_2);
    let protocol_fee = amount_out * 500u64 / MAX_PERCENTAGE;
    let referral_fee = protocol_fee * TIER_1_FEE_PERCENT / MAX_PERCENTAGE;
    let total = amount_out - protocol_fee;
    let length = payments.len();
    setup.swap_dust_token(&payments, &user_2, length, total, None, Some(tag));
    setup.swap_dust_token(&payments, &user_2, length, total, None, Some(tag));
    setup.swap_dust_token(&payments, &user_2, length, total, None, Some(tag));

    setup.check_referral_fee_amount(tag, referral_fee * 3);

    setup.update_tier(&user_1, Some("No tier upgrade found"));
}

#[test]
fn test_tier_with_0_fee() {
    let mut setup = DustConvertorSetup::new(dust_converter::contract_obj, WRAPPED_TOKEN, USDC_TOKEN, pair_mock::contract_obj);
    setup.add_known_tokens(WRAPPED_TOKEN, vec![KNOWN_TOKEN_1, KNOWN_TOKEN_2]);
    setup.resume();

    setup.add_tier_details(b"Bronze", 0u64, 500u64, Some("Tier already exists"));
    setup.remove_tier_details(b"Bronze", None);

    setup.add_tier_details(b"Bronze", 500u64, TIER_1_FEE_PERCENT, None);
    setup.add_tier_details(b"Wood", 0u64, 0u64, None);

    let user_1 = setup.b_wrapper.create_user_account(&rust_biguint!(0u64));
    let tag = "TEST5".to_string();
    setup.register_referral_tag(&user_1, tag.as_bytes());

    setup.check_referral_fee_percentage(0u64, tag.as_bytes());

    let token_amount = 3_000_000u64;
    let user_2 = setup.b_wrapper.create_user_account(&rust_biguint!(0u64));
    setup.b_wrapper.set_esdt_balance(&user_2, KNOWN_TOKEN_1, &rust_biguint!(token_amount));
    setup.b_wrapper.set_esdt_balance(&user_2, KNOWN_TOKEN_2, &rust_biguint!(token_amount));
    let payments = [
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_1.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_amount)
        },
        TxTokenTransfer {
            token_identifier: KNOWN_TOKEN_2.to_vec(),
            nonce: 0,
            value: rust_biguint!(token_amount)
        }
    ];

    let amount_out = compute_token_out_amount(token_amount, KNOWN_TOKEN_1) + compute_token_out_amount(token_amount, KNOWN_TOKEN_2);
    let protocol_fee = amount_out * 500u64 / MAX_PERCENTAGE;
    let total = amount_out - protocol_fee;
    setup.swap_dust_token(&payments, &user_2, payments.len(), total, None, Some(tag.as_bytes()));

    setup.check_referral_fee_amount(tag.as_bytes(), 0u64);

    setup.update_tier(&user_1, None);
    setup.check_referral_fee_percentage(TIER_1_FEE_PERCENT, tag.as_bytes());
}