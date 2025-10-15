use reth_ethereum::node::EthereumNode;
use reth_node_builder::NodeBuilder;
use reth_node_core::node_config::NodeConfig;
use reth_tasks::TaskManager;

#[tokio::main]
async fn main() -> eyre::Result<()> {
// Build a custom node with modified components
    let config = NodeConfig::default();

    let task_manager = TaskManager::current();
    let executor = task_manager.executor();

    // 使用临时数据库的测试节点，满足 DB 与 Metrics 约束
    let handle = NodeBuilder::new(config)
        .testing_node(executor)
        .launch_node(EthereumNode::default())
        .await?;

    handle.wait_for_node_exit().await?;

    Ok(())
}
