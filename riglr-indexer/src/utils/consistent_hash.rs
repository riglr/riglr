//! Consistent hashing for horizontal scaling

use std::collections::{BTreeMap, HashMap};
use std::hash::{Hash, Hasher};

/// Consistent hash ring for distributing work across nodes
pub struct ConsistentHash {
    /// Virtual nodes on the hash ring
    ring: BTreeMap<u64, String>,
    /// Number of virtual nodes per physical node
    virtual_nodes: usize,
    /// Active nodes
    nodes: HashMap<String, NodeInfo>,
}

/// Information about a node
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// Unique node identifier
    pub id: String,
    /// Node weight for virtual node allocation
    pub weight: u32,
    /// Additional node metadata
    pub metadata: HashMap<String, String>,
}

impl ConsistentHash {
    /// Create a new consistent hash ring
    pub fn new(virtual_nodes: usize) -> Self {
        Self {
            ring: BTreeMap::default(),
            virtual_nodes,
            nodes: HashMap::default(),
        }
    }

    /// Add a node to the hash ring
    pub fn add_node(&mut self, node_id: String, weight: u32, metadata: HashMap<String, String>) {
        let node_info = NodeInfo {
            id: node_id.clone(),
            weight,
            metadata,
        };

        self.nodes.insert(node_id.clone(), node_info);

        // Add virtual nodes based on weight
        let virtual_node_count = (self.virtual_nodes as f64 * (weight as f64 / 100.0)) as usize;
        let virtual_node_count = virtual_node_count.max(1); // At least 1 virtual node

        for i in 0..virtual_node_count {
            let virtual_node_key = format!("{}:{}", node_id, i);
            let hash = self.hash_key(&virtual_node_key);
            self.ring.insert(hash, node_id.clone());
        }
    }

    /// Remove a node from the hash ring
    pub fn remove_node(&mut self, node_id: &str) -> bool {
        if !self.nodes.contains_key(node_id) {
            return false;
        }

        self.nodes.remove(node_id);

        // Remove all virtual nodes for this physical node
        let keys_to_remove: Vec<u64> = self
            .ring
            .iter()
            .filter(|(_, id)| *id == node_id)
            .map(|(key, _)| *key)
            .collect();

        for key in keys_to_remove {
            self.ring.remove(&key);
        }

        true
    }

    /// Get the node responsible for a given key
    pub fn get_node(&self, key: &str) -> Option<&NodeInfo> {
        if self.ring.is_empty() {
            return None;
        }

        let hash = self.hash_key(key);

        // Find the first node with hash >= our key hash
        let node_id = if let Some((_, node_id)) = self.ring.range(hash..).next() {
            node_id
        } else {
            // Wrap around to the beginning
            self.ring.iter().next()?.1
        };

        self.nodes.get(node_id)
    }

    /// Get multiple nodes for replication (returns in order of preference)
    pub fn get_nodes(&self, key: &str, count: usize) -> Vec<&NodeInfo> {
        if self.ring.is_empty() {
            return Vec::default();
        }

        let hash = self.hash_key(key);
        let mut result = Vec::default();
        let mut seen_nodes: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Start from the primary node and walk the ring
        let start_iter = self.ring.range(hash..);
        let wrap_iter = self.ring.range(..hash);
        let combined_iter = start_iter.chain(wrap_iter);

        for (_, node_id) in combined_iter {
            if !seen_nodes.contains(node_id) {
                if let Some(node_info) = self.nodes.get(node_id) {
                    result.push(node_info);
                    seen_nodes.insert(node_id.clone());

                    if result.len() >= count {
                        break;
                    }
                }
            }
        }

        result
    }

    /// Get all active nodes
    pub fn get_all_nodes(&self) -> Vec<&NodeInfo> {
        self.nodes.values().collect()
    }

    /// Get number of active nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Check if a node exists
    pub fn has_node(&self, node_id: &str) -> bool {
        self.nodes.contains_key(node_id)
    }

    /// Get load distribution (for debugging/monitoring)
    pub fn get_load_distribution(&self, sample_keys: &[String]) -> HashMap<String, usize> {
        let mut distribution = HashMap::default();

        for node_info in self.nodes.values() {
            distribution.insert(node_info.id.clone(), 0);
        }

        for key in sample_keys {
            if let Some(node) = self.get_node(key) {
                *distribution.entry(node.id.clone()).or_insert(0) += 1;
            }
        }

        distribution
    }

    /// Hash a key to a position on the ring
    fn hash_key(&self, key: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::default();
        key.hash(&mut hasher);
        hasher.finish()
    }
}

/// Builder for consistent hash ring
pub struct ConsistentHashBuilder {
    virtual_nodes: usize,
    nodes: Vec<(String, u32, HashMap<String, String>)>,
}

impl ConsistentHashBuilder {
    /// Set number of virtual nodes per physical node
    pub fn virtual_nodes(mut self, count: usize) -> Self {
        self.virtual_nodes = count;
        self
    }

    /// Add a node
    pub fn add_node(
        mut self,
        node_id: String,
        weight: u32,
        metadata: HashMap<String, String>,
    ) -> Self {
        self.nodes.push((node_id, weight, metadata));
        self
    }

    /// Add a simple node with default weight
    pub fn add_simple_node(self, node_id: String) -> Self {
        self.add_node(node_id, 100, HashMap::default())
    }

    /// Build the consistent hash ring
    pub fn build(self) -> ConsistentHash {
        let mut ring = ConsistentHash::new(self.virtual_nodes);

        for (node_id, weight, metadata) in self.nodes {
            ring.add_node(node_id, weight, metadata);
        }

        ring
    }
}

impl Default for ConsistentHashBuilder {
    fn default() -> Self {
        Self {
            virtual_nodes: 100, // Default virtual nodes
            nodes: Vec::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_hash_basic() {
        let mut ring = ConsistentHash::new(100);

        // Add some nodes
        ring.add_node("node1".to_string(), 100, HashMap::default());
        ring.add_node("node2".to_string(), 100, HashMap::default());
        ring.add_node("node3".to_string(), 100, HashMap::default());

        // Test key distribution
        let test_keys = ["key1", "key2", "key3", "key4", "key5"];

        for key in &test_keys {
            let node = ring.get_node(key);
            assert!(node.is_some(), "Should find a node for key {}", key);
        }
    }

    #[test]
    fn test_consistent_hash_node_removal() {
        let mut ring = ConsistentHash::new(100);

        ring.add_node("node1".to_string(), 100, HashMap::default());
        ring.add_node("node2".to_string(), 100, HashMap::default());

        let key = "test_key";
        let original_node = ring.get_node(key).unwrap().id.clone();

        // Remove a node
        ring.remove_node("node1");

        let new_node = ring.get_node(key).unwrap().id.clone();

        // Key should still be assignable
        assert!(!new_node.is_empty());

        // If the original node was removed, key should move to the other node
        if original_node == "node1" {
            assert_eq!(new_node, "node2");
        }
    }

    #[test]
    fn test_consistent_hash_replication() {
        let mut ring = ConsistentHash::new(100);

        ring.add_node("node1".to_string(), 100, HashMap::default());
        ring.add_node("node2".to_string(), 100, HashMap::default());
        ring.add_node("node3".to_string(), 100, HashMap::default());

        let nodes = ring.get_nodes("test_key", 2);

        assert_eq!(nodes.len(), 2);
        assert_ne!(nodes[0].id, nodes[1].id); // Should be different nodes
    }

    #[test]
    fn test_consistent_hash_builder() {
        let ring = ConsistentHashBuilder::default()
            .virtual_nodes(50)
            .add_simple_node("node1".to_string())
            .add_simple_node("node2".to_string())
            .build();

        assert_eq!(ring.node_count(), 2);
        assert!(ring.has_node("node1"));
        assert!(ring.has_node("node2"));
    }

    #[test]
    fn test_load_distribution() {
        let mut ring = ConsistentHash::new(100);

        ring.add_node("node1".to_string(), 100, HashMap::default());
        ring.add_node("node2".to_string(), 100, HashMap::default());

        let sample_keys: Vec<String> = (0..1000).map(|i| format!("key{}", i)).collect();
        let distribution = ring.get_load_distribution(&sample_keys);

        assert_eq!(distribution.len(), 2);

        // Should have some distribution (not all keys to one node)
        let node1_load = distribution.get("node1").unwrap_or(&0);
        let node2_load = distribution.get("node2").unwrap_or(&0);

        assert!(*node1_load > 0);
        assert!(*node2_load > 0);
        assert_eq!(node1_load + node2_load, 1000);

        // Distribution should be reasonably balanced (within 30% of expected)
        let expected = 500;
        let tolerance = 150;
        assert!((*node1_load as i32 - expected).abs() < tolerance);
        assert!((*node2_load as i32 - expected).abs() < tolerance);
    }

    // Additional comprehensive tests for 100% code coverage

    #[test]
    fn test_consistent_hash_new() {
        let ring = ConsistentHash::new(50);
        assert_eq!(ring.virtual_nodes, 50);
        assert_eq!(ring.node_count(), 0);
        assert!(ring.ring.is_empty());
        assert!(ring.nodes.is_empty());
    }

    #[test]
    fn test_add_node_with_different_weights() {
        let mut ring = ConsistentHash::new(100);

        let mut metadata = HashMap::new();
        metadata.insert("region".to_string(), "us-east".to_string());

        ring.add_node("high_weight".to_string(), 200, metadata.clone());
        ring.add_node("low_weight".to_string(), 50, HashMap::new());
        ring.add_node("zero_weight".to_string(), 0, HashMap::new());

        assert_eq!(ring.node_count(), 3);
        assert!(ring.has_node("high_weight"));
        assert!(ring.has_node("low_weight"));
        assert!(ring.has_node("zero_weight"));

        // Check metadata is stored correctly
        let node_info = ring.nodes.get("high_weight").unwrap();
        assert_eq!(node_info.weight, 200);
        assert_eq!(node_info.metadata.get("region").unwrap(), "us-east");
    }

    #[test]
    fn test_remove_node_nonexistent() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("node1".to_string(), 100, HashMap::new());

        // Try to remove a node that doesn't exist
        assert!(!ring.remove_node("nonexistent"));
        assert_eq!(ring.node_count(), 1);
        assert!(ring.has_node("node1"));
    }

    #[test]
    fn test_remove_node_success() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("node1".to_string(), 100, HashMap::new());
        ring.add_node("node2".to_string(), 100, HashMap::new());

        assert_eq!(ring.node_count(), 2);
        assert!(ring.remove_node("node1"));
        assert_eq!(ring.node_count(), 1);
        assert!(!ring.has_node("node1"));
        assert!(ring.has_node("node2"));
    }

    #[test]
    fn test_get_node_empty_ring() {
        let ring = ConsistentHash::new(100);
        assert!(ring.get_node("any_key").is_none());
    }

    #[test]
    fn test_get_node_single_node() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("only_node".to_string(), 100, HashMap::new());

        let node = ring.get_node("test_key").unwrap();
        assert_eq!(node.id, "only_node");
    }

    #[test]
    fn test_get_node_wrap_around() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("node1".to_string(), 100, HashMap::new());
        ring.add_node("node2".to_string(), 100, HashMap::new());

        // Test that we can get a node for any key (testing wrap-around logic)
        let node = ring.get_node("test_key_for_wraparound");
        assert!(node.is_some());
    }

    #[test]
    fn test_get_nodes_empty_ring() {
        let ring = ConsistentHash::new(100);
        let nodes = ring.get_nodes("any_key", 3);
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_get_nodes_single_node() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("only_node".to_string(), 100, HashMap::new());

        let nodes = ring.get_nodes("test_key", 3);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].id, "only_node");
    }

    #[test]
    fn test_get_nodes_multiple_nodes() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("node1".to_string(), 100, HashMap::new());
        ring.add_node("node2".to_string(), 100, HashMap::new());
        ring.add_node("node3".to_string(), 100, HashMap::new());

        let nodes = ring.get_nodes("test_key", 2);
        assert_eq!(nodes.len(), 2);

        // Should return unique nodes
        assert_ne!(nodes[0].id, nodes[1].id);
    }

    #[test]
    fn test_get_nodes_request_more_than_available() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("node1".to_string(), 100, HashMap::new());
        ring.add_node("node2".to_string(), 100, HashMap::new());

        let nodes = ring.get_nodes("test_key", 5);
        assert_eq!(nodes.len(), 2); // Should only return available nodes
    }

    #[test]
    fn test_get_all_nodes() {
        let mut ring = ConsistentHash::new(100);
        assert!(ring.get_all_nodes().is_empty());

        ring.add_node("node1".to_string(), 100, HashMap::new());
        ring.add_node("node2".to_string(), 100, HashMap::new());

        let all_nodes = ring.get_all_nodes();
        assert_eq!(all_nodes.len(), 2);

        let node_ids: std::collections::HashSet<_> = all_nodes.iter().map(|n| &n.id).collect();
        assert!(node_ids.contains(&"node1".to_string()));
        assert!(node_ids.contains(&"node2".to_string()));
    }

    #[test]
    fn test_node_count() {
        let mut ring = ConsistentHash::new(100);
        assert_eq!(ring.node_count(), 0);

        ring.add_node("node1".to_string(), 100, HashMap::new());
        assert_eq!(ring.node_count(), 1);

        ring.add_node("node2".to_string(), 100, HashMap::new());
        assert_eq!(ring.node_count(), 2);

        ring.remove_node("node1");
        assert_eq!(ring.node_count(), 1);
    }

    #[test]
    fn test_has_node() {
        let mut ring = ConsistentHash::new(100);
        assert!(!ring.has_node("node1"));

        ring.add_node("node1".to_string(), 100, HashMap::new());
        assert!(ring.has_node("node1"));
        assert!(!ring.has_node("node2"));

        ring.remove_node("node1");
        assert!(!ring.has_node("node1"));
    }

    #[test]
    fn test_get_load_distribution_empty_ring() {
        let ring = ConsistentHash::new(100);
        let sample_keys = vec!["key1".to_string(), "key2".to_string()];
        let distribution = ring.get_load_distribution(&sample_keys);
        assert!(distribution.is_empty());
    }

    #[test]
    fn test_get_load_distribution_no_keys() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("node1".to_string(), 100, HashMap::new());
        ring.add_node("node2".to_string(), 100, HashMap::new());

        let sample_keys = vec![];
        let distribution = ring.get_load_distribution(&sample_keys);

        assert_eq!(distribution.len(), 2);
        assert_eq!(*distribution.get("node1").unwrap(), 0);
        assert_eq!(*distribution.get("node2").unwrap(), 0);
    }

    #[test]
    fn test_hash_key_consistency() {
        let ring = ConsistentHash::new(100);
        let key = "test_key";

        let hash1 = ring.hash_key(key);
        let hash2 = ring.hash_key(key);

        assert_eq!(hash1, hash2); // Same key should produce same hash
    }

    #[test]
    fn test_hash_key_different_keys() {
        let ring = ConsistentHash::new(100);

        let hash1 = ring.hash_key("key1");
        let hash2 = ring.hash_key("key2");

        assert_ne!(hash1, hash2); // Different keys should produce different hashes
    }

    #[test]
    fn test_node_info_creation() {
        let mut metadata = HashMap::new();
        metadata.insert("datacenter".to_string(), "dc1".to_string());
        metadata.insert("rack".to_string(), "rack42".to_string());

        let node_info = NodeInfo {
            id: "test_node".to_string(),
            weight: 150,
            metadata: metadata.clone(),
        };

        assert_eq!(node_info.id, "test_node");
        assert_eq!(node_info.weight, 150);
        assert_eq!(node_info.metadata.get("datacenter").unwrap(), "dc1");
        assert_eq!(node_info.metadata.get("rack").unwrap(), "rack42");
    }

    #[test]
    fn test_consistent_hash_builder_default() {
        let builder = ConsistentHashBuilder::default();
        assert_eq!(builder.virtual_nodes, 100);
        assert!(builder.nodes.is_empty());
    }

    #[test]
    fn test_consistent_hash_builder_virtual_nodes() {
        let builder = ConsistentHashBuilder::default().virtual_nodes(50);
        assert_eq!(builder.virtual_nodes, 50);
    }

    #[test]
    fn test_consistent_hash_builder_add_node() {
        let mut metadata = HashMap::new();
        metadata.insert("type".to_string(), "ssd".to_string());

        let builder =
            ConsistentHashBuilder::default().add_node("node1".to_string(), 120, metadata.clone());

        assert_eq!(builder.nodes.len(), 1);
        assert_eq!(builder.nodes[0].0, "node1");
        assert_eq!(builder.nodes[0].1, 120);
        assert_eq!(builder.nodes[0].2.get("type").unwrap(), "ssd");
    }

    #[test]
    fn test_consistent_hash_builder_add_simple_node() {
        let builder = ConsistentHashBuilder::default().add_simple_node("simple_node".to_string());

        assert_eq!(builder.nodes.len(), 1);
        assert_eq!(builder.nodes[0].0, "simple_node");
        assert_eq!(builder.nodes[0].1, 100); // Default weight
        assert!(builder.nodes[0].2.is_empty()); // Default empty metadata
    }

    #[test]
    fn test_consistent_hash_builder_build() {
        let ring = ConsistentHashBuilder::default()
            .virtual_nodes(50)
            .add_simple_node("node1".to_string())
            .add_node("node2".to_string(), 150, HashMap::new())
            .build();

        assert_eq!(ring.virtual_nodes, 50);
        assert_eq!(ring.node_count(), 2);
        assert!(ring.has_node("node1"));
        assert!(ring.has_node("node2"));

        let node2 = ring.nodes.get("node2").unwrap();
        assert_eq!(node2.weight, 150);
    }

    #[test]
    fn test_consistent_hash_builder_chaining() {
        let ring = ConsistentHashBuilder::default()
            .virtual_nodes(75)
            .add_simple_node("node1".to_string())
            .add_simple_node("node2".to_string())
            .add_simple_node("node3".to_string())
            .build();

        assert_eq!(ring.node_count(), 3);
        assert!(ring.has_node("node1"));
        assert!(ring.has_node("node2"));
        assert!(ring.has_node("node3"));
    }

    #[test]
    fn test_virtual_node_minimum() {
        let mut ring = ConsistentHash::new(100);

        // Add node with zero weight - should still get at least 1 virtual node
        ring.add_node("zero_weight_node".to_string(), 0, HashMap::new());

        // The node should be findable, proving it has at least 1 virtual node
        let node = ring.get_node("test_key");
        assert!(node.is_some());
        assert_eq!(node.unwrap().id, "zero_weight_node");
    }

    #[test]
    fn test_edge_case_empty_string_key() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("node1".to_string(), 100, HashMap::new());

        let node = ring.get_node("");
        assert!(node.is_some());
        assert_eq!(node.unwrap().id, "node1");
    }

    #[test]
    fn test_edge_case_very_long_key() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("node1".to_string(), 100, HashMap::new());

        let long_key = "a".repeat(10000);
        let node = ring.get_node(&long_key);
        assert!(node.is_some());
        assert_eq!(node.unwrap().id, "node1");
    }

    #[test]
    fn test_edge_case_unicode_key() {
        let mut ring = ConsistentHash::new(100);
        ring.add_node("node1".to_string(), 100, HashMap::new());

        let unicode_key = "测试🚀日本語";
        let node = ring.get_node(unicode_key);
        assert!(node.is_some());
        assert_eq!(node.unwrap().id, "node1");
    }
}
