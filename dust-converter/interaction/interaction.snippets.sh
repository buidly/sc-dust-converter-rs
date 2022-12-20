WALLET_PEM="../../../dev-wallet/main1.pem"
PROXY="https://devnet-gateway.elrond.com"
CHAIN_ID="D"

DUST_CONVERTER_ADDRESS="erd1qqqqqqqqqqqqqpgqx5dy9lnep2uj43f066xp995phtqfyth54jusdk8pg4"

# . ./interaction.snippets.sh && deploy 500 100 WEGLD-d7c6bb
deploy() {
    wegld_token_id="0x$(echo -n $3 | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract deploy --recall-nonce \
        --metadata-payable-by-sc \
        --pem=${WALLET_PEM} \
        --gas-limit=200000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --bytecode="../output/dust-converter.wasm" \
        --arguments $1 $2 $wegld_token_id \
        --outfile="deploy-stake-farm-internal.interaction.json" --send || return

    ADDRESS=$(erdpy data parse --file="deploy-stake-farm-internal.interaction.json" --expression="data['contractAddress']")

    echo ""
    echo "Smart Contract address: ${ADDRESS}"
}

# . ./interaction.snippets.sh && upgrade 500 100 WEGLD-d7c6bb
upgrade() {
    wegld_token_id="0x$(echo -n $3 | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract upgrade ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --metadata-payable-by-sc \
        --pem=${WALLET_PEM} \
        --gas-limit=200000000 \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --bytecode="../output/dust-converter.wasm" \
        --arguments $1 $2 $wegld_token_id \
        --outfile="deploy-stake-farm-internal.interaction.json" --send || return

    ADDRESS=$(erdpy data parse --file="deploy-stake-farm-internal.interaction.json" --expression="data['contractAddress']")

    echo ""
    echo "Smart Contract address: ${ADDRESS}"
}

# . ./interaction.snippets.sh && addKnownTokens USDC-8d4068 erd1qqqqqqqqqqqqqpgqq67uv84ma3cekpa55l4l68ajzhq8qm3u0n4s20ecvx 0x016345785d8a0000
# . ./interaction.snippets.sh && addKnownTokens ASH-4ce444 erd1qqqqqqqqqqqqqpgq53wlytsnh0g5hauxsx5fyf40eafzz9w00n4sswvfwq 0x016345785d8a0000
# . ./interaction.snippets.sh && addKnownTokens MEX-dc289c erd1qqqqqqqqqqqqqpgquu5rsa4ee6l4azz6vdu4hjp8z4p6tt8m0n4suht3dy 0x016345785d8a0000
# . ./interaction.snippets.sh && addKnownTokens DTK-5935ad erd1qqqqqqqqqqqqqpgq3lwucu7dx286sa9zps7ygpwknwmpav8c0n4sjulxqa 0x016345785d8a0000
# . ./interaction.snippets.sh && addKnownTokens RIDE-6e4c49 erd1qqqqqqqqqqqqqpgqe8m9w7cv2ekdc28q5ahku9x3hcregqpn0n4sum0e3u 0x016345785d8a0000
addKnownTokens() {
    token_id="0x$(echo -n $1 | xxd -p -u | tr -d '\n')"

    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --pem=${WALLET_PEM} \
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
        --pem=${WALLET_PEM} \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=10000000 \
        --function=removeKnownTokens \
        --arguments $token_id \
        --send || return
}

# . ./interaction.snippets.sh && sellDustTokens
sellDustTokens() {
    erdpy --verbose contract call ${DUST_CONVERTER_ADDRESS} --recall-nonce \
        --pem=${WALLET_PEM} \
        --proxy=${PROXY} --chain=${CHAIN_ID} \
        --gas-limit=30000000 \
        --function=sellDustTokens \
        --send || return
}