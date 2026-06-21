use parking_lot::RwLock;
use shared_types::{KnowledgeGraphEdge, KnowledgeGraphNode};
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

use ai_core::AiResult;

struct GraphState {
    nodes: HashMap<Uuid, KnowledgeGraphNode>,
    edges: Vec<KnowledgeGraphEdge>,
}

impl Default for GraphState {
    fn default() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: Vec::new(),
        }
    }
}

/// In-memory security knowledge graph.
pub struct SecurityKnowledgeGraph {
    state: RwLock<GraphState>,
}

impl SecurityKnowledgeGraph {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(GraphState::default()),
        }
    }

    pub fn add_node(&self, node: KnowledgeGraphNode) {
        self.state.write().nodes.insert(node.id, node);
    }

    pub fn add_edge(&self, edge: KnowledgeGraphEdge) -> AiResult<()> {
        let state = self.state.read();
        if !state.nodes.contains_key(&edge.source_id) || !state.nodes.contains_key(&edge.target_id) {
            return Err(ai_core::AiError::KnowledgeGraph("unknown node".into()));
        }
        drop(state);
        self.state.write().edges.push(edge);
        Ok(())
    }

    pub fn node_count(&self) -> usize {
        self.state.read().nodes.len()
    }

    pub fn edge_count(&self) -> usize {
        self.state.read().edges.len()
    }

    pub fn neighbors(&self, node_id: Uuid) -> Vec<Uuid> {
        self.state
            .read()
            .edges
            .iter()
            .filter(|e| e.source_id == node_id || e.target_id == node_id)
            .map(|e| {
                if e.source_id == node_id {
                    e.target_id
                } else {
                    e.source_id
                }
            })
            .collect()
    }

    pub fn traverse(&self, start: Uuid, max_depth: u32) -> Vec<Uuid> {
        let mut visited = Vec::new();
        let mut queue = VecDeque::from([(start, 0u32)]);

        while let Some((node, depth)) = queue.pop_front() {
            if visited.contains(&node) {
                continue;
            }
            visited.push(node);
            if depth >= max_depth {
                continue;
            }
            for neighbor in self.neighbors(node) {
                queue.push_back((neighbor, depth + 1));
            }
        }

        visited
    }

    pub fn shortest_path(&self, from: Uuid, to: Uuid) -> Option<Vec<Uuid>> {
        if from == to {
            return Some(vec![from]);
        }

        let mut queue = VecDeque::from([from]);
        let mut parent: HashMap<Uuid, Uuid> = HashMap::new();
        let mut seen = vec![from];

        while let Some(current) = queue.pop_front() {
            for neighbor in self.neighbors(current) {
                if seen.contains(&neighbor) {
                    continue;
                }
                seen.push(neighbor);
                parent.insert(neighbor, current);
                if neighbor == to {
                    let mut path = vec![to];
                    let mut cursor = to;
                    while let Some(&prev) = parent.get(&cursor) {
                        path.push(prev);
                        if prev == from {
                            break;
                        }
                        cursor = prev;
                    }
                    path.reverse();
                    return Some(path);
                }
                queue.push_back(neighbor);
            }
        }

        None
    }
}

impl Default for SecurityKnowledgeGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_node(label: &str) -> KnowledgeGraphNode {
        KnowledgeGraphNode {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            label: label.into(),
            node_kind: "asset".into(),
            properties: serde_json::json!({}),
        }
    }

    #[test]
    fn shortest_path_finds_route() {
        let graph = SecurityKnowledgeGraph::new();
        let a = sample_node("a");
        let b = sample_node("b");
        let c = sample_node("c");
        let a_id = a.id;
        let b_id = b.id;
        let c_id = c.id;
        graph.add_node(a);
        graph.add_node(b);
        graph.add_node(c);
        graph.add_edge(KnowledgeGraphEdge {
            id: Uuid::new_v4(),
            source_id: a_id,
            target_id: b_id,
            relation: "connects".into(),
            weight: 1.0,
        }).unwrap();
        graph.add_edge(KnowledgeGraphEdge {
            id: Uuid::new_v4(),
            source_id: b_id,
            target_id: c_id,
            relation: "connects".into(),
            weight: 1.0,
        }).unwrap();

        let path = graph.shortest_path(a_id, c_id).unwrap();
        assert_eq!(path, vec![a_id, b_id, c_id]);
    }

    #[test]
    fn traverse_respects_depth() {
        let graph = SecurityKnowledgeGraph::new();
        let a = sample_node("a");
        let b = sample_node("b");
        let a_id = a.id;
        let b_id = b.id;
        graph.add_node(a);
        graph.add_node(b);
        graph.add_edge(KnowledgeGraphEdge {
            id: Uuid::new_v4(),
            source_id: a_id,
            target_id: b_id,
            relation: "connects".into(),
            weight: 1.0,
        }).unwrap();

        let visited = graph.traverse(a_id, 0);
        assert_eq!(visited, vec![a_id]);
    }
}
