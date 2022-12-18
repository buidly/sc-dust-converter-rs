#![no_std]

elrond_wasm::imports!();

pub mod config;
pub mod proxy;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[elrond_wasm::contract]
pub trait EmptyContract:
    config::ConfigModule +
    proxy::ProxyModule 
{
    
    #[init]
    fn init(&self) {}
}
