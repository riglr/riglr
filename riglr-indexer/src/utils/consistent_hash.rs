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
            ring: BTreeMap::new(),
            virtual_nodes,
            nodes: HashMap::new(),
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
            return Vec::new();
        }

        let hash = self.hash_key(key);
        let mut result = Vec::new();
        let mut seen_nodes = std::collections::HashSet::new();

        // Start from the primary node and walk the ring
        let start_iter = self.ring.range(hash..);
        let wrap_iter = self.ring.range(..hash);
        let combined_iter = start_iter.chain(wrap_iter);

        for (_, node_id) in combined_iter {
            if !seen_nodes.contains(node_id) {
                if let Some(node_info) = self.nodes.get(node_id) {
                    result.push(node_info);
                    seen_nodes.insert(node_id);

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
        let mut distribution = HashMap::new();

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
        let mut hasher = DefaultHasher::new();
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
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            virtual_nodes: 100, // Default virtual nodes
            nodes: Vec::new(),
        }
    }

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
        self.add_node(node_id, 100, HashMap::new())
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
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consistent_hash_basic() {
        let mut ring = ConsistentHash::new(100);

        // Add some nodes
        ring.add_node("node1".to_string(), 100, HashMap::new());
        ring.add_node("node2".to_string(), 100, HashMap::new());
        ring.add_node("node3".to_string(), 100, HashMap::new());

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

        ring.add_node("node1".to_string(), 100, HashMap::new());
        ring.add_node("node2".to_string(), 100, HashMap::new());

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

        ring.add_node("node1".to_string(), 100, HashMap::new());
        ring.add_node("node2".to_string(), 100, HashMap::new());
        ring.add_node("node3".to_string(), 100, HashMap::new());

        let nodes = ring.get_nodes("test_key", 2);

        assert_eq!(nodes.len(), 2);
        assert_ne!(nodes[0].id, nodes[1].id); // Should be different nodes
    }

    #[test]
    fn test_consistent_hash_builder() {
        let ring = ConsistentHashBuilder::new()
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

        ring.add_node("node1".to_string(), 100, HashMap::new());
        ring.add_node("node2".to_string(), 100, HashMap::new());

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
}
