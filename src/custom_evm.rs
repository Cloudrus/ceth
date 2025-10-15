use reth_ethereum::evm::{primitives::Database, EthEvm, EthEvmConfig};
use reth_ethereum::evm::revm::handler::EthPrecompiles;
use reth_ethereum::node::{
    api::{FullNodeTypes, NodeTypes},
    builder::{components::ExecutorBuilder, BuilderContext},
};
use reth_evm::{eth::EthEvmContext, precompiles::PrecompilesMap, EvmFactory};
use reth_ethereum::evm::revm::context::{Context, TxEnv};
use reth_ethereum::evm::revm::{MainContext, MainBuilder};
use reth_evm::revm::context_interface::result::{EVMError, HaltReason};
use reth_evm::revm::inspector::{Inspector, NoOpInspector};
use reth_evm::revm::interpreter::interpreter::EthInterpreter;
use reth_evm::revm::precompile::{PrecompileFn, PrecompileOutput, PrecompileResult, Precompiles};
use reth_evm::revm::primitives::{hardfork::SpecId, Address, Bytes};
use reth_ethereum::EthPrimitives;
use reth_ethereum::chainspec::ChainSpec;
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, Copy)]
pub struct MyEvmFactory;

impl EvmFactory for MyEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> = EthEvm<DB, I, Self::Precompiles>;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(&self, db: DB, input: reth_ethereum::evm::primitives::EvmEnv) -> Self::Evm<DB, NoOpInspector> {
        let spec = input.cfg_env.spec;

        let mut evm = Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(PrecompilesMap::from_static(EthPrecompiles::default().precompiles));

        if spec >= SpecId::PRAGUE {
            evm = evm.with_precompiles(PrecompilesMap::from_static(custom_precompiles()));
        }

        EthEvm::new(evm, false)
    }

    fn create_evm_with_inspector<DB: Database, I: Inspector<Self::Context<DB>, EthInterpreter>>(
        &self,
        db: DB,
        input: reth_ethereum::evm::primitives::EvmEnv,
        inspector: I,
    ) -> Self::Evm<DB, I> {
        EthEvm::new(self.create_evm(db, input).into_inner().with_inspector(inspector), true)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct MyExecutorBuilder;

impl<Node> ExecutorBuilder<Node> for MyExecutorBuilder
where
    Node: FullNodeTypes<Types: NodeTypes<ChainSpec = ChainSpec, Primitives = EthPrimitives>>,
{
    type EVM = EthEvmConfig<ChainSpec, MyEvmFactory>;

    async fn build_evm(self, ctx: &BuilderContext<Node>) -> eyre::Result<Self::EVM> {
        Ok(EthEvmConfig::new_with_evm_factory(ctx.chain_spec(), MyEvmFactory))
    }
}

pub fn custom_precompiles() -> &'static Precompiles {
    static INSTANCE: OnceLock<Precompiles> = OnceLock::new();
    INSTANCE.get_or_init(|| {
        let mut precompiles = Precompiles::prague().clone();

        // 地址 0x100 -> 20 字节：18 个 0，后两字节 0x01, 0x00
        let mut addr_bytes = [0u8; 20];
        addr_bytes[18] = 0x01;
        addr_bytes[19] = 0x00;
        let addr = Address::from_slice(&addr_bytes);

        // 预编译实现：TestRead.read() 返回 uint256(100) 的 ABI 编码（32 字节，末位为 100）
        let pc: PrecompileFn = |_, _| -> PrecompileResult {
            let mut out = vec![0u8; 32];
            out[31] = 100;
            PrecompileResult::Ok(PrecompileOutput::new(0, Bytes::from(out)))
        };

        precompiles.extend([(addr, pc).into()]);
        precompiles
    })
}


