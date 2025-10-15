use alloy_genesis::Genesis;
use reth_ethereum::chainspec::{Chain, ChainSpec};

pub fn custom_chainspec() -> ChainSpec {
    ChainSpec::builder()
        .chain(Chain::mainnet())
        .genesis(Genesis::default())
        .london_activated()
        .paris_activated()
        .shanghai_activated()
        .cancun_activated()
        .prague_activated()
        .build()
}


