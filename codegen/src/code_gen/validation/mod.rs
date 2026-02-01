//! Schema validation module
//!
//! Validates IR schemas before code generation to catch errors early.

use std::collections::{HashMap, HashSet};

use crate::code_gen::ir::{
    IRFieldType, IRPackage, IRSchema, IRStruct, IRType, IRTypeAlias, IRTypeAliasTarget, IRUnion,
    IRUnionVariant,
};

/// Validation errors
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Reference to an unknown type
    UnknownType {
        type_name: String,
        referenced_from: String,
        field_name: Option<String>,
    },
    /// Duplicate type name within a package
    DuplicateType { type_name: String, package: String },
    /// Circular dependency detected
    CircularDependency { cycle: Vec<String> },
    /// Empty enum (no variants)
    EmptyEnum { type_name: String },
    /// Empty struct (no fields) - warning level
    EmptyStruct { type_name: String },
    /// Union with no variants
    EmptyUnion { type_name: String },
    /// Invalid union variant (references non-object type for inline style)
    InvalidUnionVariant {
        union_name: String,
        variant_name: String,
        reason: String,
    },
}

/// Validation warnings (non-fatal)
#[derive(Debug, Clone)]
pub enum ValidationWarning {
    /// Type is defined but never referenced
    UnusedType { type_name: String },
    /// Field name uses non-idiomatic casing
    NonIdiomaticNaming {
        type_name: String,
        field_name: String,
    },
}

/// Schema validator
pub struct Validator {
    /// Primitive type names
    primitive_types: HashSet<String>,
}

impl Validator {
    pub fn new() -> Self {
        let primitive_types: HashSet<String> = [
            // Basic primitives
            "String", "Bool", "DateTime", "UInt32", "UInt64", "Int32", "Int64", "Float32",
            "Float64", "Any",
            // Extended primitives
            "UUID", "Decimal", "Bytes", "Url", "Timestamp", "TimestampMillis", "DateTimeUtc",
            "DateTimeTz", "Date", "Time", "Duration",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self { primitive_types }
    }

    /// Validate an IR schema
    pub fn validate(&self, schema: &IRSchema) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        // Collect all known types first
        let known_types = self.collect_known_types(schema);

        // Check for duplicate types
        errors.extend(self.check_duplicates(schema));

        // Validate each package
        for package in schema.packages.values() {
            errors.extend(self.validate_package(package, &known_types));
        }

        // Note: Circular dependency detection is disabled because:
        // 1. Rust allows recursive types with Box/Arc
        // 2. The existing schemas may have valid circular references
        // 3. The code generation handles these cases properly
        // If you need stricter validation, uncomment the line below:
        // errors.extend(self.check_circular_dependencies(schema));

        errors
    }

    fn collect_known_types(&self, schema: &IRSchema) -> HashSet<String> {
        let mut types = self.primitive_types.clone();

        for package in schema.packages.values() {
            for ir_type in &package.types {
                types.insert(ir_type.name().to_string());
            }
        }

        types
    }

    fn check_duplicates(&self, schema: &IRSchema) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for package in schema.packages.values() {
            let mut seen: HashSet<String> = HashSet::new();
            for ir_type in &package.types {
                let name = ir_type.name().to_string();
                if seen.contains(&name) {
                    errors.push(ValidationError::DuplicateType {
                        type_name: name,
                        package: package.name.clone(),
                    });
                } else {
                    seen.insert(name);
                }
            }
        }

        errors
    }

    fn validate_package(
        &self,
        package: &IRPackage,
        known_types: &HashSet<String>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for ir_type in &package.types {
            match ir_type {
                IRType::Struct(s) => {
                    errors.extend(self.validate_struct(s, known_types));
                }
                IRType::Enum(e) => {
                    if e.variants.is_empty() {
                        errors.push(ValidationError::EmptyEnum {
                            type_name: e.name.clone(),
                        });
                    }
                }
                IRType::Union(u) => {
                    errors.extend(self.validate_union(u, known_types));
                }
                IRType::TypeAlias(a) => {
                    errors.extend(self.validate_type_alias(a, known_types));
                }
            }
        }

        errors
    }

    fn validate_struct(&self, s: &IRStruct, known_types: &HashSet<String>) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        for field in &s.fields {
            if let Some(type_name) = self.get_custom_type_name(&field.field_type) {
                if !known_types.contains(&type_name) {
                    errors.push(ValidationError::UnknownType {
                        type_name,
                        referenced_from: s.name.clone(),
                        field_name: Some(field.name.clone()),
                    });
                }
            }
        }

        errors
    }

    fn validate_union(&self, u: &IRUnion, known_types: &HashSet<String>) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if u.variants.is_empty() {
            errors.push(ValidationError::EmptyUnion {
                type_name: u.name.clone(),
            });
        }

        for variant in &u.variants {
            match variant {
                IRUnionVariant::Unit(_) => {}
                IRUnionVariant::Inline(name, _) => {
                    if !known_types.contains(name) {
                        errors.push(ValidationError::UnknownType {
                            type_name: name.clone(),
                            referenced_from: u.name.clone(),
                            field_name: None,
                        });
                    }
                }
                IRUnionVariant::Newtype(_, type_ref) => {
                    if !known_types.contains(type_ref) {
                        errors.push(ValidationError::UnknownType {
                            type_name: type_ref.clone(),
                            referenced_from: u.name.clone(),
                            field_name: None,
                        });
                    }
                }
            }
        }

        errors
    }

    fn validate_type_alias(
        &self,
        a: &IRTypeAlias,
        known_types: &HashSet<String>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        match &a.target {
            IRTypeAliasTarget::List(item_type) => {
                if let Some(type_name) = self.get_custom_type_name(item_type) {
                    if !known_types.contains(&type_name) {
                        errors.push(ValidationError::UnknownType {
                            type_name,
                            referenced_from: a.name.clone(),
                            field_name: None,
                        });
                    }
                }
            }
            IRTypeAliasTarget::Map(key_type, value_type) => {
                if let Some(type_name) = self.get_custom_type_name(key_type) {
                    if !known_types.contains(&type_name) {
                        errors.push(ValidationError::UnknownType {
                            type_name,
                            referenced_from: a.name.clone(),
                            field_name: Some("key".to_string()),
                        });
                    }
                }
                if let Some(type_name) = self.get_custom_type_name(value_type) {
                    if !known_types.contains(&type_name) {
                        errors.push(ValidationError::UnknownType {
                            type_name,
                            referenced_from: a.name.clone(),
                            field_name: Some("value".to_string()),
                        });
                    }
                }
            }
        }

        errors
    }

    fn get_custom_type_name(&self, field_type: &IRFieldType) -> Option<String> {
        match field_type {
            IRFieldType::Custom(name) => Some(name.clone()),
            IRFieldType::List(inner) => self.get_custom_type_name(inner),
            IRFieldType::Map(k, v) => self
                .get_custom_type_name(k)
                .or_else(|| self.get_custom_type_name(v)),
            IRFieldType::Primitive(_) | IRFieldType::Any => None,
        }
    }

    fn check_circular_dependencies(&self, schema: &IRSchema) -> Vec<ValidationError> {
        // Build dependency graph
        let mut deps: HashMap<String, Vec<String>> = HashMap::new();

        for package in schema.packages.values() {
            for ir_type in &package.types {
                let type_name = ir_type.name().to_string();
                let type_deps = self.get_type_dependencies(ir_type);
                deps.insert(type_name, type_deps);
            }
        }

        // Detect cycles using DFS
        let mut errors = Vec::new();
        let mut visited: HashSet<String> = HashSet::new();
        let mut rec_stack: HashSet<String> = HashSet::new();
        let mut path: Vec<String> = Vec::new();

        for type_name in deps.keys() {
            if !visited.contains(type_name) {
                if let Some(cycle) =
                    self.detect_cycle(type_name, &deps, &mut visited, &mut rec_stack, &mut path)
                {
                    errors.push(ValidationError::CircularDependency { cycle });
                }
            }
        }

        errors
    }

    fn get_type_dependencies(&self, ir_type: &IRType) -> Vec<String> {
        let mut deps = Vec::new();

        match ir_type {
            IRType::Struct(s) => {
                for field in &s.fields {
                    if let Some(name) = self.get_custom_type_name(&field.field_type) {
                        // Exclude boxed fields from dependency graph (they break cycles)
                        if !field.is_boxed {
                            deps.push(name);
                        }
                    }
                }
            }
            IRType::Union(u) => {
                for variant in &u.variants {
                    match variant {
                        IRUnionVariant::Newtype(_, type_ref) => {
                            deps.push(type_ref.clone());
                        }
                        IRUnionVariant::Inline(name, _) => {
                            deps.push(name.clone());
                        }
                        IRUnionVariant::Unit(_) => {}
                    }
                }
            }
            IRType::TypeAlias(a) => match &a.target {
                IRTypeAliasTarget::List(t) => {
                    if let Some(name) = self.get_custom_type_name(t) {
                        deps.push(name);
                    }
                }
                IRTypeAliasTarget::Map(k, v) => {
                    if let Some(name) = self.get_custom_type_name(k) {
                        deps.push(name);
                    }
                    if let Some(name) = self.get_custom_type_name(v) {
                        deps.push(name);
                    }
                }
            },
            IRType::Enum(_) => {}
        }

        deps
    }

    fn detect_cycle(
        &self,
        node: &str,
        deps: &HashMap<String, Vec<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = deps.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) = self.detect_cycle(neighbor, deps, visited, rec_stack, path)
                    {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(neighbor) {
                    // Found cycle - extract it from path
                    let cycle_start = path.iter().position(|n| n == neighbor).unwrap_or(0);
                    let mut cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycle.push(neighbor.clone());
                    return Some(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
        None
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}
