use alloy_primitives::keccak256;
use reth_ethereum::EthPrimitives;
use reth_ethereum::chainspec::ChainSpec;
use reth_ethereum::evm::revm::context::{Context, TxEnv};
use reth_ethereum::evm::revm::handler::EthPrecompiles;
use reth_ethereum::evm::revm::{MainBuilder, MainContext};
use reth_ethereum::evm::{EthEvm, EthEvmConfig, primitives::Database};
use reth_ethereum::node::{
    api::{FullNodeTypes, NodeTypes},
    builder::{BuilderContext, components::ExecutorBuilder},
};
use reth_evm::revm::context_interface::result::{EVMError, HaltReason};
use reth_evm::revm::inspector::{Inspector, NoOpInspector};
use reth_evm::revm::interpreter::interpreter::EthInterpreter;
use reth_evm::revm::precompile::{PrecompileFn, PrecompileOutput, PrecompileResult, Precompiles};
use reth_evm::revm::primitives::{Address, Bytes, hardfork::SpecId};
use reth_evm::{EvmFactory, eth::EthEvmContext, precompiles::PrecompilesMap};
use std::sync::OnceLock;

#[derive(Debug, Clone, Default, Copy)]
pub struct MyEvmFactory;

impl EvmFactory for MyEvmFactory {
    type Evm<DB: Database, I: Inspector<EthEvmContext<DB>, EthInterpreter>> =
        EthEvm<DB, I, Self::Precompiles>;
    type Tx = TxEnv;
    type Error<DBError: core::error::Error + Send + Sync + 'static> = EVMError<DBError>;
    type HaltReason = HaltReason;
    type Context<DB: Database> = EthEvmContext<DB>;
    type Spec = SpecId;
    type Precompiles = PrecompilesMap;

    fn create_evm<DB: Database>(
        &self,
        db: DB,
        input: reth_ethereum::evm::primitives::EvmEnv,
    ) -> Self::Evm<DB, NoOpInspector> {
        let spec = input.cfg_env.spec;

        let mut evm = Context::mainnet()
            .with_db(db)
            .with_cfg(input.cfg_env)
            .with_block(input.block_env)
            .build_mainnet_with_inspector(NoOpInspector {})
            .with_precompiles(PrecompilesMap::from_static(
                EthPrecompiles::default().precompiles,
            ));

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
        EthEvm::new(
            self.create_evm(db, input)
                .into_inner()
                .with_inspector(inspector),
            true,
        )
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
        Ok(EthEvmConfig::new_with_evm_factory(
            ctx.chain_spec(),
            MyEvmFactory,
        ))
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
        let pc: PrecompileFn = |data: &[u8], _gas: u64| -> PrecompileResult {
            static READ_SELECTOR: OnceLock<[u8; 4]> = OnceLock::new();
            let sel = *READ_SELECTOR.get_or_init(|| {
                let h = keccak256("read()");
                let mut s = [0u8; 4];
                s.copy_from_slice(&h.as_slice()[0..4]);
                s
            });
            if data.len() >= 4 && data[0..4] == sel {
                let mut out = vec![0u8; 32];
                out[31] = 100;
                println!("RestRead: {}", Bytes::from(out.clone()));
                return PrecompileResult::Ok(PrecompileOutput::new(0, Bytes::from(out)));
            }
            PrecompileResult::Ok(PrecompileOutput::new(0, Bytes::new()))
        };

        precompiles.extend([(addr, pc).into()]);
        // 有状态预编译
        // 地址 0x200 -> 20 字节：18 个 0，后两字节 0x02, 0x00
        let mut num_addr_bytes = [0u8; 20];
        num_addr_bytes[18] = 0x02;
        num_addr_bytes[19] = 0x00;
        let num_addr = Address::from_slice(&num_addr_bytes);

        let num_pc: PrecompileFn = |data: &[u8], _gas: u64| -> PrecompileResult {
            use std::sync::atomic::{AtomicU64, Ordering};

            // 全局静态原子变量，用作存储
            static NUM: AtomicU64 = AtomicU64::new(0);

            // 函数选择器
            static SET_SELECTOR: OnceLock<[u8; 4]> = OnceLock::new();
            static GET_SELECTOR: OnceLock<[u8; 4]> = OnceLock::new();

            let set_sel = *SET_SELECTOR.get_or_init(|| {
                let h = keccak256("setNum(uint64)");
                let mut s = [0u8; 4];
                s.copy_from_slice(&h.as_slice()[0..4]);
                s
            });

            let get_sel = *GET_SELECTOR.get_or_init(|| {
                let h = keccak256("getNum()");
                let mut s = [0u8; 4];
                s.copy_from_slice(&h.as_slice()[0..4]);
                s
            });

            if data.len() >= 4 {
                // setNum(uint64)
                if data[0..4] == set_sel && data.len() >= 36 {
                    let mut buf = [0u8; 8];
                    buf.copy_from_slice(&data[data.len() - 8..]); // uint64 参数位于最后8字节
                    let val = u64::from_be_bytes(buf);
                    NUM.store(val, Ordering::SeqCst);
                    println!("setNum called, new value: {}", val);
                    return PrecompileResult::Ok(PrecompileOutput::new(0, Bytes::new()));
                }

                // getNum()
                if data[0..4] == get_sel {
                    let val = NUM.load(Ordering::SeqCst);
                    println!("getNum called, current value: {}", val);
                    let mut out = vec![0u8; 32];
                    out[24..32].copy_from_slice(&val.to_be_bytes()); // uint256 ABI编码
                    return PrecompileResult::Ok(PrecompileOutput::new(0, Bytes::from(out)));
                }
            }

            PrecompileResult::Ok(PrecompileOutput::new(0, Bytes::new()))
        };
        precompiles.extend([(num_addr, num_pc).into()]);

        precompiles
    })
}
