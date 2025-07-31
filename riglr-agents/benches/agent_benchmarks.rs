use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use riglr_agents::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

#[derive(Clone)]
struct BenchmarkAgent {
    id: AgentId,
    capabilities: Vec<String>,
    execution_delay: Duration,
}

#[async_trait::async_trait]
impl Agent for BenchmarkAgent {
    async fn execute_task(&self, task: Task) -> Result<TaskResult> {
        if self.execution_delay > Duration::from_millis(0) {
            tokio::time::sleep(self.execution_delay).await;
        }

        Ok(TaskResult::success(
            serde_json::json!({
                "agent_id": self.id.as_str(),
                "task_id": task.id
            }),
            None,
            self.execution_delay,
        ))
    }

    fn id(&self) -> &AgentId {
        &self.id
    }

    fn capabilities(&self) -> Vec<String> {
        self.capabilities.clone()
    }
}

fn benchmark_agent_registration(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("agent_registration", |b| {
        b.iter(|| {
            rt.block_on(async {
                let registry = LocalAgentRegistry::new();
                
                for i in 0..black_box(100) {
                    let agent = Arc::new(BenchmarkAgent {
                        id: AgentId::new(format!("agent-{}", i)),
                        capabilities: vec!["trading".to_string()],
                        execution_delay: Duration::from_millis(0),
                    });
                    registry.register_agent(agent).await.unwrap();
                }
            });
        })
    });
}

fn benchmark_task_dispatch(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("task_dispatch", |b| {
        b.to_async(&rt).iter(|| async {
            let registry = Arc::new(LocalAgentRegistry::new());
            let dispatcher = AgentDispatcher::new(registry.clone());
            
            // Register an agent
            let agent = Arc::new(BenchmarkAgent {
                id: AgentId::new("benchmark-agent"),
                capabilities: vec!["trading".to_string()],
                execution_delay: Duration::from_millis(1),
            });
            registry.register_agent(agent).await.unwrap();
            
            // Dispatch a task
            let task = Task::new(
                TaskType::Trading,
                serde_json::json!({"symbol": "BTC/USD"}),
            );
            
            black_box(dispatcher.dispatch_task(task).await.unwrap());
        });
    });
}

fn benchmark_message_passing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    
    c.bench_function("message_passing", |b| {
        b.to_async(&rt).iter(|| async {
            let comm = ChannelCommunication::new();
            let agent_id = AgentId::new("test-agent");
            
            let _receiver = comm.subscribe(&agent_id).await.unwrap();
            
            let message = AgentMessage::new(
                AgentId::new("sender"),
                Some(agent_id.clone()),
                "benchmark".to_string(),
                serde_json::json!({"data": "test"}),
            );
            
            comm.send_message(message).await.unwrap();
            black_box(());
        });
    });
}

criterion_group!(benches, benchmark_agent_registration, benchmark_task_dispatch, benchmark_message_passing);
criterion_main!(benches);