WALLET_PEM="../../../dev-wallet/main1.pem"
PROXY="https://devnet-gateway.multiversx.com"
CHAIN_ID="D"

DUST_CONVERTER_ADDRESS="erd1qqqqqqqqqqqqqpgq6u07hhkfsvuk5aae92g549s6pc2s9ycq0dps368jr5"


SLIPPAGE=50
FEE_PERCENTAGE=500
WEGLD=WEGLD-bd4d79

# . ./interaction.snippets.sh && deploy 500 100 WEGLD-d7c6bb
deploy() {
    wegld_token_id="0x$(echo -n $3 | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract deploy --recall-nonce \
        --metadata-payable-by-sc \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --gas-limit=200000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --bytecode="../output/dust-converter.wasm" \
        --arguments ${FEE_PERCENTAGE} ${SLIPPAGE} str:${WEGLD} \
        --outfile="deploy-stake-farm-internal.interaction.json" --send || return

    ADDRESS=$(erdpy data parse --file="deploy-stake-farm-internal.interaction.json" --expression="data['contractAddress']")

    echo ""
    echo "Smart Contract address: ${ADDRESS}"
}

# . ./interaction.snippets.sh && upgrade 500 100 WEGLD-d7c6bb
upgrade() {
    erdpy --verbose contract upgrade ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --metadata-payable-by-sc \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --gas-limit=200000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --bytecode="../output/dust-converter.wasm" \
        --arguments ${FEE_PERCENTAGE} ${SLIPPAGE} str:${WEGLD} \
        --outfile="deploy-stake-farm-internal.interaction.json" --send || return

    ADDRESS=$(erdpy data parse --file="deploy-stake-farm-internal.interaction.json" --expression="data['contractAddress']")

    echo ""
    echo "Smart Contract address: ${ADDRESS}"
}

resume() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=30000000 \
        --function=resume \
        --send || return
}

extractFees() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=extractFees \
        --send || return
}

# . ./interaction.snippets.sh && addKnownTokens USDC-8d4068 erd1qqqqqqqqqqqqqpgqq67uv84ma3cekpa55l4l68ajzhq8qm3u0n4s20ecvx 0x016345785d8a0000
# . ./interaction.snippets.sh && addKnownTokens ASH-4ce444 erd1qqqqqqqqqqqqqpgq53wlytsnh0g5hauxsx5fyf40eafzz9w00n4sswvfwq 0x016345785d8a0000
# . ./interaction.snippets.sh && addKnownTokens MEX-dc289c erd1qqqqqqqqqqqqqpgquu5rsa4ee6l4azz6vdu4hjp8z4p6tt8m0n4suht3dy 0x016345785d8a0000
# . ./interaction.snippets.sh && addKnownTokens DTK-5935ad erd1qqqqqqqqqqqqqpgq3lwucu7dx286sa9zps7ygpwknwmpav8c0n4sjulxqa 0x016345785d8a0000
# . ./interaction.snippets.sh && addKnownTokens RIDE-6e4c49 erd1qqqqqqqqqqqqqpgqe8m9w7cv2ekdc28q5ahku9x3hcregqpn0n4sum0e3u 0x016345785d8a0000
addKnownTokens() {
    token_id="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=addKnownTokens \
        --arguments $token_id $2 $3 \
        --send || return
}

# . ./interaction.snippets.sh && removeKnownTokens RIDE-6e4c49
removeKnownTokens() {
    token_id="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=removeKnownTokens \
        --arguments $token_id \
        --send || return
}

# . ./interaction.snippets.sh && sellDustTokens
sellDustTokens() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=30000000 \
        --function=sellDustTokens \
        --send || return
}
# source interaction.snippets.sh && registerReferralTag alexj172
registerReferralTag() {
    tag="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=30000000 \
        --function=registerReferralTag \
        --arguments $tag \
        --send || return
}

# . ./interaction.snippets.sh && addTierDetails Bronze 0x00 500 Silver 0x22b1c8c1227a0000 1000 Gold 0xd02ab486cedc0000 2000
# . ./interaction.snippets.sh && addTierDetails Platinum 0x056bc75e2d63100000 4000
# . ./interaction.snippets.sh && addTierDetails Gold 0x0a688906bd8b0000 2000
# . ./interaction.snippets.sh && addTierDetails Platinum 0x0de0b6b3a7640000 3000
addTierDetails() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=addTierDetails \
        --arguments str:$1 $2 $3 \
        --send || return
}


addTierDetails() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=addTierDetails \
        --arguments str:$1 $2 $3 \
        --send || return
}

# . ./interaction.snippets.sh && removeTierDetails Silver 
removeTierDetails() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=removeTierDetails \
        --arguments str:$1 \
        --send || return
}

setFeePercentage() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=setFeePercentage \
        --arguments $FEE_PERCENTAGE \
        --send || return
}

TAG=egld4ever
CUSTOM_FEE=2000
setReferralFeePercentage() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=setReferralFeePercentage \
        --arguments str:${TAG} $CUSTOM_FEE \
        --send || return
}

resume() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=resume \
        --send || return
}

pause() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=pause \
        --send || return
}


# . ./interaction.snippets.sh && removeKnownTokens RIDE-6e4c49
removeKnownTokens() {
    token_id="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=removeKnownTokens \
        --arguments $token_id \
        --send || return
}

# . ./interaction.snippets.sh && sellDustTokens
sellDustTokens() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --ledger --ledger-account-index 0 --ledger-address-index 4  \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=30000000 \
        --function=sellDustTokens \
        --send || return
}