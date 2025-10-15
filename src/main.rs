mod custom_evm;
mod chain;

use reth_ethereum::{ node::EthereumNode, node::core::args::RpcServerArgs, node::node::EthereumAddOns };
use reth_node_builder::NodeBuilder;
use reth_node_core::node_config::NodeConfig;
use reth_tasks::TaskManager;

#[tokio::main]
async fn main() -> eyre::Result<()> {
// Build a test node with HTTP RPC enabled (defaults to 127.0.0.1:8545) and custom chainspec
    let spec = chain::custom_chainspec();
    let config = NodeConfig::test().with_rpc(RpcServerArgs::default().with_http()).with_chain(spec);

    let task_manager = TaskManager::current();
    let executor = task_manager.executor();

    // 使用临时数据库的测试节点，并启用以太坊默认组件与 RPC
    let handle = NodeBuilder::new(config)
        .testing_node(executor)
        .with_types::<EthereumNode>()
        .with_components(EthereumNode::components().executor(custom_evm::MyExecutorBuilder))
        .with_add_ons(EthereumAddOns::default())
        .launch()
        .await?;

    handle.wait_for_node_exit().await?;

    Ok(())
}
