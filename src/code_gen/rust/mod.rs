use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Write},
};

use anyhow::{anyhow, bail};

use crate::definitions::{CustomType, Definition, FieldType};

pub struct RustCodeGen {}

struct TypeInfo<'a> {
    value: &'a CustomType,
    package: String,
    cyclic_ref_group_id: Option<u32>,
}

struct TypeNode<'a> {
    value: &'a CustomType,
    info: NodeInfo,
}
#[derive(Clone)]
struct NodeInfo {
    index: u64,
    low_link: u64,
}
impl NodeInfo {
    fn empty() -> Self {
        Self {
            index: 0,
            low_link: 0,
        }
    }
}

struct TypeGraph<'a> {
    type_nodes: HashMap<String, TypeNode<'a>>,
    // traverse stack (type names)
    stack: Vec<String>,
    // list of cyclic referenced group of types, including the single isolated ones
    sub_graphs: Vec<Vec<TypeNode<'a>>>,
}

struct TraverseResult {
    index: u64,
    low_link: u64,
}

impl RustCodeGen {
    pub fn code_gen(definitions: Vec<Definition>) -> anyhow::Result<()> {
        let mut type_nodes: HashMap<String, TypeNode> = HashMap::new();
        let all_types: Vec<CustomType> = definitions
            .into_iter()
            .flat_map(|d| d.custom_types)
            .collect();
        for t in all_types.iter() {
            let name = get_type_name(&t);
            if type_nodes.contains_key(&name) {
                bail!("Duplicate type name: {}", &name);
            }
            let node = TypeNode {
                value: t,
                info: NodeInfo::empty(),
            };
            type_nodes.insert(name, node);
        }

        // Detect the cyclic referenced types
        println!("detecting cyclic referenced types...");
        let graph = TypeGraph::new(type_nodes);
        let type_sub_graphs = graph.group_cyclic_referenced_types(all_types.iter().collect())?;

        let mut type_dict = HashMap::new();
        let mut group_id = 0;
        for sub_graph in type_sub_graphs {
            let opt_group_id = match sub_graph.len() {
                v if v <= 1 => None,
                _ => {
                    group_id = group_id + 1;
                    Some(group_id)
                }
            };
            for node in sub_graph {
                type_dict.insert(
                    get_type_name(node.value),
                    TypeInfo {
                        value: node.value,
                        // TODO: extract package info
                        package: "protocols".to_owned(),
                        cyclic_ref_group_id: opt_group_id,
                    },
                );
            }
        }

        // gen codes for each type now
        for (type_name, type_info) in type_dict.iter() {
            gen_code_for(&type_dict, type_name, type_info)?;
        }

        Ok(())
    }
}

fn gen_code_for(
    type_dict: &HashMap<String, TypeInfo<'_>>,
    type_name: &str,
    type_info: &TypeInfo<'_>,
) -> anyhow::Result<()> {
    let output_path = "/tmp/test_gen/";
    let output_file_name = format!("{}/{}.rs", output_path, type_name);
    let file = File::create(output_file_name)?;
    let mut writer = BufWriter::new(file);
    match type_info.value {
        CustomType::Object { name, fields } => {
            let refs = get_ref_types(type_info.value);
            // write package usages
            for ref_type_name in refs {
                let ref_type = type_dict
                    .get(&ref_type_name)
                    .ok_or_else(|| anyhow!("Cannot find referenced type: {}", ref_type_name))?;
                writer.write_all(
                    format!(
                        "use {}::{}\n",
                        ref_type.package,
                        get_type_name(ref_type.value)
                    )
                    .as_bytes(),
                )?;
            }
            writer.write_all("\n".as_bytes())?;
            // write type
            writer.write_all(format!("pub struct {} {{\n", name).as_bytes())?;
            // write fields
            for field in fields.iter() {
                let type_to_write = match &field.field_type {
                    FieldType::Simple(t) => t.to_string(),
                    FieldType::Custom { name } => {
                        // check if the field type and current type is in same cyclic ref group
                        let ref_type = type_dict
                            .get(name)
                            .ok_or_else(|| anyhow!("Cannot find field type: {}", name))?;
                        if type_info.cyclic_ref_group_id != None
                            && ref_type.cyclic_ref_group_id == type_info.cyclic_ref_group_id
                        {
                            format!("Box<{}>", name)
                        } else {
                            name.clone()
                        }
                    }
                };
                writer.write_all(format!("  pub {}: {},\n", field.name, type_to_write).as_bytes())?;
            }
            writer.write_all("}\n".as_bytes())?;
        }
        CustomType::Enum { name, values } => {
            // write type
            writer.write_all(format!("pub enum {} {{\n", name).as_bytes())?;
            // write fields
            for value in values.iter() {
                writer.write_all(format!("  {},\n", value).as_bytes())?;
            }
            writer.write_all("}".as_bytes())?;
        }
    };
    Ok(())
}

impl<'g> TypeGraph<'g> {
    fn new(type_nodes: HashMap<String, TypeNode<'g>>) -> Self {
        Self {
            type_nodes,
            stack: Vec::new(),
            sub_graphs: Vec::new(),
        }
    }

    fn group_cyclic_referenced_types(
        mut self,
        all_types: Vec<&CustomType>,
    ) -> anyhow::Result<Vec<Vec<TypeNode<'g>>>> {
        // traverse the graph to find all cyclic referenced groups of type nodes.
        let mut index = 1;
        for typ in all_types {
            let name = get_type_name(&typ);
            // skip if type node has been traversed
            if self
                .get_node_info(&name, |n| n.info.index == 0)?
                .unwrap_or(false)
            {
                let r = self.traverse(&name, index)?;
                index = r.index;
            }
        }

        // output the cyclic referenced groups
        Ok(self.sub_graphs)
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
            n.info.index = index;
            n.info.low_link = index;
        })?;
        self.stack.push(curr_type_name.to_owned());

        let mut low_link = index;
        let mut index = index;
        let ref_type_names: Vec<String> = self
            .get_node_info(curr_type_name, |n| get_ref_types(n.value))?
            .ok_or_else(|| anyhow!("cannot find type: {}", curr_type_name))?;
        for ref_type_name in ref_type_names {
            let opt_node_info = self.get_node_info(&ref_type_name, |n| n.info.clone())?;
            match opt_node_info {
                Some(node) if node.index == 0 => {
                    // node is not yet traversed
                    let r = self.traverse(&ref_type_name, index + 1)?;
                    index = r.index;
                    if low_link > r.low_link {
                        self.update_node(curr_type_name, |n| {
                            n.info.low_link = r.low_link;
                        })?;
                        low_link = r.low_link;
                    }
                }
                Some(node) => {
                    // node is traversed and on stack
                    if low_link > node.low_link {
                        self.update_node(curr_type_name, |n| {
                            n.info.low_link = node.low_link;
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
            .get_node_info(curr_type_name, |n| n.info.index == n.info.low_link)?
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

fn get_ref_types(value: &CustomType) -> Vec<String> {
    match value {
        CustomType::Object { name, fields } => fields
            .iter()
            .filter(|f| is_custom_type(&f.field_type))
            .map(|f| get_field_type_name(&f.field_type))
            .collect(),
        CustomType::Enum { name, values } => vec![],
    }
}

fn get_field_type_name(field_type: &FieldType) -> String {
    match field_type {
        FieldType::Simple(t) => format!("{:?}", t),
        FieldType::Custom { name } => name.to_string(),
    }
}

fn is_custom_type(field_type: &FieldType) -> bool {
    match field_type {
        FieldType::Simple(_) => false,
        FieldType::Custom { name } => true,
    }
}

fn get_type_name(t: &CustomType) -> String {
    match t {
        CustomType::Object { name, fields } => name.to_string(),
        CustomType::Enum { name, values } => name.to_string(),
    }
}
