// Code generated by the elrond-wasm multi-contract system. DO NOT EDIT.

////////////////////////////////////////////////////
////////////////// AUTO-GENERATED //////////////////
////////////////////////////////////////////////////

// Init:                                 1
// Endpoints:                           22
// Async Callback (empty):               1
// Total number of exported functions:  24

#![no_std]

elrond_wasm_node::wasm_endpoints! {
    dust_converter
    (
        swapDustTokens
        sellDustTokens
        topUp
        extractFees
        setFeePercentage
        setSlippagePercentage
        addKnownTokens
        removeKnownTokens
        getAllTokens
        getTokenThreshold
        getProtocolFeePercent
        getSlippagePercent
        getWrappedTokenId
        addAdmin
        removeAdmin
        updateOwnerOrAdmin
        getPermissions
        addToPauseWhitelist
        removeFromPauseWhitelist
        pause
        resume
        getState
    )
}

elrond_wasm_node::wasm_empty_callback! {}
