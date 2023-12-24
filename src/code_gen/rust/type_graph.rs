use std::collections::HashMap;

use crate::code_gen::abi::TypeInfo;

use anyhow::anyhow;

pub(crate) struct TypeNode<'a> {
    pub(crate) type_info: &'a TypeInfo,
    node_info: NodeInfo,
}

#[derive(Clone)]
pub(crate) struct NodeInfo {
    index: u64,
    low_link: u64,
}

impl NodeInfo {
    pub fn empty() -> Self {
        Self {
            index: 0,
            low_link: 0,
        }
    }
}

pub(crate) struct TypeGraph<'a> {
    type_nodes: HashMap<String, TypeNode<'a>>,
    // traverse stack (type names)
    stack: Vec<String>,
    // list of cyclic referenced group of types, including the single isolated ones
    sub_graphs: Vec<Vec<TypeNode<'a>>>,
}

pub(crate) struct TraverseResult {
    index: u64,
    low_link: u64,
}

impl<'g> TypeGraph<'g> {
    pub(crate) fn new(all_types: &'g HashMap<String, TypeInfo>) -> anyhow::Result<Self> {
        let mut type_nodes: HashMap<String, TypeNode> = HashMap::new();
        for (type_name, type_info) in all_types.iter() {
            if type_nodes.contains_key(type_name) {
                return Err(anyhow!("Duplicate type name: {}", &type_name));
            }
            let node = TypeNode {
                type_info,
                node_info: NodeInfo::empty(),
            };
            type_nodes.insert(type_name.clone(), node);
        }

        Ok(Self {
            type_nodes,
            stack: Vec::new(),
            sub_graphs: Vec::new(),
        })
    }

    pub(crate) fn group_cyclic_referenced_types(
        mut self,
    ) -> anyhow::Result<Vec<Vec<&'g TypeInfo>>> {
        // traverse the graph to find all cyclic referenced groups of type nodes.
        let mut index = 1;
        let cloned_type_names = self.type_nodes.keys().cloned().collect::<Vec<_>>();
        for name in cloned_type_names {
            // skip if type node has been traversed
            if self
                .get_node_info(&name, |n| n.node_info.index == 0)?
                .unwrap_or(false)
            {
                let r = self.traverse(&name, index)?;
                index = r.index;
            }
        }

        // output the cyclic referenced groups
        Ok(self
            .sub_graphs
            .iter()
            .map(|nodes| nodes.iter().map(|n| n.type_info).collect())
            .collect())
    }

    fn update_node(
        &mut self,
        name: &str,
        update: impl FnOnce(&mut TypeNode) -> (),
    ) -> anyhow::Result<()> {
        let node = self
            .type_nodes
            .get_mut(name)
            .ok_or_else(|| anyhow!("Cannot find type: {}", name))?;
        update(node);
        Ok(())
    }

    /// Traverse the type reference graph to get all cyclic referenced graphs (includes single
    /// isolated type).
    ///
    /// Returns the grown index and lowest link after traversal.
    fn traverse(&mut self, curr_type_name: &str, index: u64) -> anyhow::Result<TraverseResult> {
        self.update_node(curr_type_name, |n| {
            n.node_info.index = index;
            n.node_info.low_link = index;
        })?;
        self.stack.push(curr_type_name.to_owned());

        let mut low_link = index;
        let mut index = index;
        let ref_type_names: Vec<String> = self
            .get_node_info(curr_type_name, |n| n.type_info.get_referrenced_types())?
            .ok_or_else(|| anyhow!("cannot find type: {}", curr_type_name))?;
        println!(
            "find ref types for {}: {:?}",
            curr_type_name, ref_type_names
        );
        for ref_type_name in ref_type_names {
            let opt_node_info = self.get_node_info(&ref_type_name, |n| n.node_info.clone())?;
            match opt_node_info {
                Some(node) if node.index == 0 => {
                    // node is not yet traversed
                    let r = self.traverse(&ref_type_name, index + 1)?;
                    index = r.index;
                    if low_link > r.low_link {
                        self.update_node(curr_type_name, |n| {
                            n.node_info.low_link = r.low_link;
                        })?;
                        low_link = r.low_link;
                    }
                }
                Some(node) => {
                    // node is traversed and on stack
                    if low_link > node.low_link {
                        self.update_node(curr_type_name, |n| {
                            n.node_info.low_link = node.low_link;
                        })?;
                        low_link = node.low_link;
                    }
                }
                None => {
                    // node already traversed and off stack now, no need to traverse again
                }
            }
        }

        if self
            .get_node_info(curr_type_name, |n| {
                n.node_info.index == n.node_info.low_link
            })?
            .ok_or_else(|| anyhow!("cannot find type: {}", curr_type_name))?
        {
            println!("found root node: {}", curr_type_name);
            // found a cyclic referenced group of types
            let mut sub_graph = Vec::new();
            loop {
                let type_name = self
                    .stack
                    .pop()
                    .ok_or_else(|| anyhow!("shouldn't happen!"))?;
                let node = self
                    .remove_node(&type_name)?
                    .ok_or_else(|| anyhow!("shouldn't happen!"))?;
                sub_graph.push(node);
                if curr_type_name == type_name {
                    break;
                }
            }
            self.sub_graphs.push(sub_graph);
        }
        Ok(TraverseResult { index, low_link })
    }

    fn get_node_info<T>(
        &self,
        type_name: &str,
        f: impl FnOnce(&TypeNode) -> T,
    ) -> anyhow::Result<Option<T>> {
        let node = self.type_nodes.get(type_name);
        Ok(node.map(|n| f(n)))
    }

    fn remove_node(&mut self, type_name: &str) -> anyhow::Result<Option<TypeNode<'g>>> {
        let node = self.type_nodes.remove(type_name);
        Ok(node)
    }
}
