use dkn_compute::{
    node::DriaComputeNode, utils::payload::TaskRequestPayload, waku::message::P2PMessage,
};
use fastbloom_rs::{FilterBuilder, Membership};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct MockPayload {
    number: usize,
}

#[tokio::test]
async fn test_two_tasks() {
    let topic = "testing";
    let time = Duration::from_secs(10).as_nanos();
    let input = MockPayload { number: 42 };
    let node = DriaComputeNode::new();
    let mut messages: Vec<P2PMessage> = Vec::new();

    // create filter with your own address
    let mut filter = FilterBuilder::new(128, 0.01).build_bloom_filter();
    filter.add(&node.config.address);

    let payload_tasked = TaskRequestPayload::new(input.clone(), filter, time, None);
    let payload_str = serde_json::to_string(&payload_tasked).unwrap();
    messages.push(P2PMessage::new(payload_str, topic));

    // create another filter without your own address
    let mut filter = FilterBuilder::new(128, 0.01).build_bloom_filter();
    filter.add(&Uuid::new_v4().to_string().as_bytes()); // something dummy

    let payload_not_tasked = TaskRequestPayload::new(input, filter, time, None);
    let payload_str = serde_json::to_string(&payload_not_tasked).unwrap();
    messages.push(P2PMessage::new(payload_str, topic));

    let tasks = node.parse_messages::<MockPayload>(messages.clone(), false);
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_id, payload_tasked.task_id);
    assert_ne!(tasks[0].task_id, payload_not_tasked.task_id);
}
