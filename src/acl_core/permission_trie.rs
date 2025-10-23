use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tracing::debug;

pub use crate::acl_core::radix_node::{PermissionRule, RadixNode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionTrie {
    root: RadixNode,
}

impl Default for PermissionTrie {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionTrie {
    /// Creates a new empty permission trie
    pub fn new() -> Self {
        Self {
            root: RadixNode::new(),
        }
    }

    #[allow(dead_code)]
    pub fn insert(&mut self, path: &str, rule: PermissionRule) {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        let mut current_node = &mut self.root;

        for segment in segments {
            if segment.starts_with('{') && segment.ends_with('}') {
                // Variable path, e.g., {id}
                let var_name = segment[1..segment.len() - 1].to_string();
                if let Some(boxed_node) = &current_node.variable_child {
                    let (existing_name, _) = &**boxed_node;
                    if *existing_name != var_name {
                        panic!(
                            "Conflict: tried to insert path with variable '{{{}}}' but '{{{}}}' already exists at this level.",
                            var_name, existing_name
                        );
                    }
                }
                if current_node.wildcard_child.is_some() {
                    panic!(
                        "Conflict: cannot insert variable path '{{{}}}' because a wildcard '*' already exists at this level.",
                        var_name
                    );
                }

                let entry = current_node
                    .variable_child
                    .get_or_insert_with(|| Box::new((var_name.clone(), RadixNode::new())));
                current_node = &mut entry.1;
            } else if segment.starts_with('*') {
                // Wildcard path, e.g., *filepath
                let wildcard_name = segment[1..].to_string();
                if current_node.variable_child.is_some() || current_node.wildcard_child.is_some() {
                    panic!(
                        "Conflict: cannot insert wildcard path '*{}' because another dynamic path already exists at this level.",
                        wildcard_name
                    );
                }
                let entry = current_node
                    .wildcard_child
                    .get_or_insert_with(|| Box::new((wildcard_name.clone(), RadixNode::new())));
                current_node = &mut entry.1;
            } else {
                // Static path
                current_node = current_node
                    .static_children
                    .entry(segment.to_string())
                    .or_insert_with(RadixNode::new);
            }
        }
        // Wrap rule with Arc for shared ownership
        current_node.rule = Some(Arc::new(rule));
    }

    /// Finds a matching permission rule for the given path
    /// Returns the rule and extracted path parameters
    #[allow(dead_code)]
    pub(crate) fn find(
        &self,
        path: &str,
    ) -> Option<(Arc<PermissionRule>, HashMap<String, String>)> {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        debug!("Finding path: {}, segments: {:?}", path, segments);
        self.find_recursive(&self.root, &segments, &mut HashMap::new())
    }

    pub(crate) fn find_recursive(
        &self,
        node: &RadixNode,
        remaining_segments: &[&str],
        params: &mut HashMap<String, String>,
    ) -> Option<(Arc<PermissionRule>, HashMap<String, String>)> {
        if remaining_segments.is_empty() {
            return node
                .rule
                .as_ref()
                .map(|rule_arc| (rule_arc.clone(), params.clone()));
        }

        let current_segment = remaining_segments[0];
        let remaining = &remaining_segments[1..];

        // Priority 1: Static path matching
        if let Some(child_node) = node.static_children.get(current_segment) {
            if let Some(result) = self.find_recursive(child_node, remaining, params) {
                return Some(result);
            }
        }

        // Priority 2: Variable path matching
        if let Some(boxed_node) = &node.variable_child {
            let (var_name, child_node) = &**boxed_node;
            params.insert(var_name.clone(), current_segment.to_string());
            if let Some(result) = self.find_recursive(child_node, remaining, params) {
                return Some(result);
            }
            // Backtracking: Remove parameter if subsequent path doesn't match
            params.remove(var_name);
        }

        // Priority 3: Wildcard path matching
        if let Some((star_name, child_node)) = node.wildcard_child.as_deref() {
            params.insert(star_name.clone(), current_segment.to_string());
            if let Some(result) = self.find_recursive(child_node, remaining, params) {
                return Some(result);
            }
            params.remove(star_name);
        }

        None
    }
}
