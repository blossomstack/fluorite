# Code Generation Improvements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve the Rust code generation system for simplicity, maintainability, and extensibility by adding configurable derives, validation, IR layer, askama templates, and abstract file system.

**Architecture:** Introduce a language-agnostic Intermediate Representation (IR) layer between type definitions and code generation. Replace string formatting with askama templates. Add a validation phase before generation. Abstract file system operations for testability.

**Tech Stack:** Rust, askama (compile-time templates), existing serde/anyhow

---

## Task 1: Add askama Dependency and Create Template Directory

**Files:**
- Modify: `codegen/Cargo.toml`
- Create: `codegen/templates/` directory

**Step 1: Add askama dependency to Cargo.toml**

Edit `codegen/Cargo.toml` to add:

```toml
[dependencies]
anyhow = "1.0"
askama = "0.12"
clap = { version = "4.5", features = ["derive"] }
serde = { version = "1.0", features = ["serde_derive"] }
serde_yaml = "0.9"
fluorite = { path = "../runtime/", version = "0.1" }
derive-new = "0.7"
```

**Step 2: Create templates directory**

Run: `mkdir -p codegen/templates/rust`

**Step 3: Verify build works**

Run: `cargo build --package fluorite_codegen`
Expected: Build succeeds

**Step 4: Commit**

```bash
git add codegen/Cargo.toml
git commit -m "chore: add askama dependency for template-based code generation"
```

---

## Task 2: Extend RustOptions with Configurable Derives

**Files:**
- Modify: `codegen/src/code_gen/rust/options.rs`

**Step 1: Write test for configurable derives**

Add to `codegen/tests/rust_code_gen.rs`:

```rust
#[test]
fn test_custom_derives() {
    let options = RustOptions::new("/tmp/test".to_owned())
        .with_derives(vec!["Debug".to_string(), "Clone".to_string()]);
    assert_eq!(options.derives, vec!["Debug", "Clone"]);
}

#[test]
fn test_default_derives() {
    let options = RustOptions::new("/tmp/test".to_owned());
    assert!(options.derives.contains(&"Debug".to_string()));
    assert!(options.derives.contains(&"serde::Serialize".to_string()));
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --package fluorite_codegen test_custom_derives`
Expected: FAIL - no method `with_derives`

**Step 3: Implement configurable derives in RustOptions**

Replace `codegen/src/code_gen/rust/options.rs`:

```rust
use crate::code_gen::utils::to_snake_case;

#[derive(Debug, Clone)]
pub struct RustOptions {
    pub output_dir: String,
    pub single_file: bool,
    pub any_type: String,
    pub derives: Vec<String>,
    pub visibility: Visibility,
    pub generate_new: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Visibility {
    Public,
    PublicCrate,
    Private,
}

impl Default for Visibility {
    fn default() -> Self {
        Visibility::Public
    }
}

impl RustOptions {
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            single_file: true,
            any_type: "fluorite::Any".to_owned(),
            derives: Self::default_derives(),
            visibility: Visibility::Public,
            generate_new: true,
        }
    }

    pub fn default_derives() -> Vec<String> {
        vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "PartialEq".to_string(),
            "serde::Serialize".to_string(),
            "serde::Deserialize".to_string(),
        ]
    }

    pub fn with_single_file(mut self, single_file: bool) -> Self {
        self.single_file = single_file;
        self
    }

    pub fn with_any_type(mut self, any_type: &str) -> Self {
        self.any_type = any_type.to_owned();
        self
    }

    pub fn with_derives(mut self, derives: Vec<String>) -> Self {
        self.derives = derives;
        self
    }

    pub fn with_additional_derives(mut self, derives: Vec<String>) -> Self {
        self.derives.extend(derives);
        self
    }

    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    pub fn with_generate_new(mut self, generate_new: bool) -> Self {
        self.generate_new = generate_new;
        self
    }

    pub fn type_to_file_name(&self, type_name: &str) -> String {
        to_snake_case(type_name)
    }

    pub fn get_derives_string(&self) -> String {
        let mut derives = self.derives.clone();
        if self.generate_new {
            derives.push("derive_new::new".to_string());
        }
        format!("#[derive({})]", derives.join(", "))
    }

    pub fn get_visibility_string(&self) -> &'static str {
        match self.visibility {
            Visibility::Public => "pub",
            Visibility::PublicCrate => "pub(crate)",
            Visibility::Private => "",
        }
    }

    pub(crate) fn get_simple_type(&self, t: &crate::definitions::SimpleType) -> String {
        match t {
            crate::definitions::SimpleType::String => "String".to_string(),
            crate::definitions::SimpleType::Bool => "bool".to_string(),
            crate::definitions::SimpleType::DateTime => "DateTime".to_string(),
            crate::definitions::SimpleType::UInt32 => "u32".to_string(),
            crate::definitions::SimpleType::UInt64 => "u64".to_string(),
            crate::definitions::SimpleType::Int32 => "i32".to_string(),
            crate::definitions::SimpleType::Int64 => "i64".to_string(),
            crate::definitions::SimpleType::Float32 => "f32".to_string(),
            crate::definitions::SimpleType::Float64 => "f64".to_string(),
        }
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test --package fluorite_codegen test_custom_derives test_default_derives`
Expected: PASS

**Step 5: Commit**

```bash
git add codegen/src/code_gen/rust/options.rs codegen/tests/rust_code_gen.rs
git commit -m "feat: add configurable derives and visibility to RustOptions"
```

---

## Task 3: Create Intermediate Representation (IR) Module

**Files:**
- Create: `codegen/src/code_gen/ir/mod.rs`
- Create: `codegen/src/code_gen/ir/types.rs`
- Modify: `codegen/src/code_gen/mod.rs`

**Step 1: Create IR types module**

Create `codegen/src/code_gen/ir/types.rs`:

```rust
//! Language-agnostic Intermediate Representation for code generation.
//!
//! This IR sits between the parsed YAML definitions and language-specific
//! code generation, providing a clean abstraction layer.

use std::collections::HashMap;

/// Represents a complete schema ready for code generation
#[derive(Debug, Clone)]
pub struct IRSchema {
    pub packages: HashMap<String, IRPackage>,
}

/// A package/module containing types
#[derive(Debug, Clone)]
pub struct IRPackage {
    pub name: String,
    pub types: Vec<IRType>,
}

/// A type in the IR
#[derive(Debug, Clone)]
pub enum IRType {
    Struct(IRStruct),
    Enum(IREnum),
    Union(IRUnion),
    TypeAlias(IRTypeAlias),
}

impl IRType {
    pub fn name(&self) -> &str {
        match self {
            IRType::Struct(s) => &s.name,
            IRType::Enum(e) => &e.name,
            IRType::Union(u) => &u.name,
            IRType::TypeAlias(a) => &a.name,
        }
    }

    pub fn is_internal(&self) -> bool {
        match self {
            IRType::Struct(s) => s.is_union_variant,
            IRType::Enum(_) | IRType::Union(_) | IRType::TypeAlias(_) => false,
        }
    }
}

/// A struct type
#[derive(Debug, Clone)]
pub struct IRStruct {
    pub name: String,
    pub fields: Vec<IRField>,
    pub is_union_variant: bool,
    pub doc: Option<String>,
}

/// A field within a struct
#[derive(Debug, Clone)]
pub struct IRField {
    pub name: String,
    pub field_type: IRFieldType,
    pub is_optional: bool,
    pub is_boxed: bool,
    pub rename: Option<String>,
    pub doc: Option<String>,
}

impl IRField {
    /// Returns the name to use in generated code (respects rename)
    pub fn code_name(&self) -> &str {
        self.rename.as_deref().unwrap_or(&self.name)
    }

    /// Returns the original name (for serde rename attribute)
    pub fn original_name(&self) -> &str {
        &self.name
    }

    /// Whether this field needs a serde rename attribute
    pub fn needs_rename(&self) -> bool {
        self.rename.is_some()
    }
}

/// Field type representation
#[derive(Debug, Clone)]
pub enum IRFieldType {
    Primitive(IRPrimitive),
    Custom(String),
    Any,
    List(Box<IRFieldType>),
    Map(Box<IRFieldType>, Box<IRFieldType>),
}

/// Primitive types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IRPrimitive {
    String,
    Bool,
    DateTime,
    UInt32,
    UInt64,
    Int32,
    Int64,
    Float32,
    Float64,
}

/// An enum type (simple variants without data)
#[derive(Debug, Clone)]
pub struct IREnum {
    pub name: String,
    pub variants: Vec<String>,
    pub doc: Option<String>,
}

/// A tagged union type
#[derive(Debug, Clone)]
pub struct IRUnion {
    pub name: String,
    pub tag_field: String,
    pub variants: Vec<IRUnionVariant>,
    pub style: IRUnionStyle,
    pub doc: Option<String>,
}

/// Union variant
#[derive(Debug, Clone)]
pub enum IRUnionVariant {
    /// Simple variant with no data (unit variant)
    Unit(String),
    /// Variant with inlined struct fields
    Inline(String, Vec<IRField>),
    /// Variant wrapping another type
    Newtype(String, String),
}

impl IRUnionVariant {
    pub fn name(&self) -> &str {
        match self {
            IRUnionVariant::Unit(n) => n,
            IRUnionVariant::Inline(n, _) => n,
            IRUnionVariant::Newtype(n, _) => n,
        }
    }
}

/// How to generate the union
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IRUnionStyle {
    /// Inline fields into enum variants
    Inline,
    /// Use newtype pattern wrapping external types
    Extern,
}

/// Type alias (for List and Map types)
#[derive(Debug, Clone)]
pub struct IRTypeAlias {
    pub name: String,
    pub target: IRTypeAliasTarget,
    pub doc: Option<String>,
}

#[derive(Debug, Clone)]
pub enum IRTypeAliasTarget {
    List(IRFieldType),
    Map(IRFieldType, IRFieldType),
}
```

**Step 2: Create IR mod.rs**

Create `codegen/src/code_gen/ir/mod.rs`:

```rust
mod types;

pub use types::*;
```

**Step 3: Update code_gen/mod.rs to include IR**

Edit `codegen/src/code_gen/mod.rs`:

```rust
pub mod abi;
pub mod generator;
pub mod rust;
pub mod ts;
pub mod utils;
pub mod ir;

pub use abi::*;
pub use generator::*;
```

**Step 4: Verify build**

Run: `cargo build --package fluorite_codegen`
Expected: Build succeeds

**Step 5: Commit**

```bash
git add codegen/src/code_gen/ir/
git add codegen/src/code_gen/mod.rs
git commit -m "feat: add language-agnostic Intermediate Representation (IR) module"
```

---

## Task 4: Create IR Builder from Type Definitions

**Files:**
- Create: `codegen/src/code_gen/ir/builder.rs`
- Modify: `codegen/src/code_gen/ir/mod.rs`

**Step 1: Write test for IR builder**

Add to `codegen/tests/rust_code_gen.rs`:

```rust
use fluorite_codegen::code_gen::ir::{IRBuilder, IRType, IRUnionStyle};

#[test]
fn test_ir_builder_creates_schema() {
    let d1 = deserialize_definition_file("../examples/users.yml").unwrap();
    let d2 = deserialize_definition_file("../examples/orders.yml").unwrap();

    let schema = IRBuilder::new().build(&vec![d1, d2]).unwrap();

    // Should have two packages
    assert_eq!(schema.packages.len(), 2);
    assert!(schema.packages.contains_key("protocols.users"));
    assert!(schema.packages.contains_key("protocols.orders"));

    // Users package should have User and Gender
    let users_pkg = schema.packages.get("protocols.users").unwrap();
    assert_eq!(users_pkg.types.len(), 2);
}

#[test]
fn test_ir_builder_handles_unions() {
    let d = deserialize_definition_file("../examples/orders.yml").unwrap();
    let schema = IRBuilder::new().build(&vec![d]).unwrap();

    let orders_pkg = schema.packages.get("protocols.orders").unwrap();
    let address_union = orders_pkg.types.iter()
        .find(|t| t.name() == "Address")
        .unwrap();

    if let IRType::Union(u) = address_union {
        assert_eq!(u.tag_field, "type");
        assert_eq!(u.variants.len(), 3);
        assert_eq!(u.style, IRUnionStyle::Inline);
    } else {
        panic!("Expected union type");
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --package fluorite_codegen test_ir_builder`
Expected: FAIL - no IRBuilder

**Step 3: Implement IR builder**

Create `codegen/src/code_gen/ir/builder.rs`:

```rust
//! Builds IR from YAML definitions

use std::collections::{HashMap, HashSet};
use anyhow::{anyhow, Result};

use crate::definitions::{CustomType, Definition, Field, SimpleType, UnionStyle};

use super::{
    IREnum, IRField, IRFieldType, IRPackage, IRPrimitive, IRSchema, IRStruct,
    IRType, IRTypeAlias, IRTypeAliasTarget, IRUnion, IRUnionStyle, IRUnionVariant,
};

/// Builds an IRSchema from definitions
pub struct IRBuilder {
    /// All type names across all definitions (for resolving references)
    all_type_names: HashSet<String>,
    /// Types that are used as inline union variants
    union_variant_names: HashSet<String>,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            all_type_names: HashSet::new(),
            union_variant_names: HashSet::new(),
        }
    }

    /// Build IR schema from definitions
    pub fn build(mut self, definitions: &[Definition]) -> Result<IRSchema> {
        // First pass: collect all type names and identify union variants
        self.collect_type_info(definitions);

        // Second pass: build IR types
        let mut packages: HashMap<String, IRPackage> = HashMap::new();

        for def in definitions {
            let package_name = def
                .configs
                .rust_package
                .as_ref()
                .ok_or_else(|| anyhow!("Missing rust_package in definition"))?
                .clone();

            let package = packages.entry(package_name.clone()).or_insert_with(|| {
                IRPackage {
                    name: package_name,
                    types: Vec::new(),
                }
            });

            for custom_type in &def.types {
                let ir_type = self.convert_type(custom_type)?;
                package.types.push(ir_type);
            }
        }

        Ok(IRSchema { packages })
    }

    fn collect_type_info(&mut self, definitions: &[Definition]) {
        for def in definitions {
            for t in &def.types {
                self.all_type_names.insert(t.type_name().to_owned());

                // Identify inline union variants
                if let CustomType::Union { values, configs, .. } = t {
                    let is_inline = configs
                        .as_ref()
                        .and_then(|c| c.union_style.as_ref())
                        .map(|s| *s != UnionStyle::Extern)
                        .unwrap_or(true);

                    if is_inline {
                        for v in values {
                            if self.all_type_names.contains(v) ||
                               definitions.iter().any(|d| d.types.iter().any(|t| t.type_name() == v)) {
                                self.union_variant_names.insert(v.clone());
                            }
                        }
                    }
                }
            }
        }

        // Second pass to catch union variants that reference types defined later
        for def in definitions {
            for t in &def.types {
                if let CustomType::Union { values, configs, .. } = t {
                    let is_inline = configs
                        .as_ref()
                        .and_then(|c| c.union_style.as_ref())
                        .map(|s| *s != UnionStyle::Extern)
                        .unwrap_or(true);

                    if is_inline {
                        for v in values {
                            if self.all_type_names.contains(v) {
                                self.union_variant_names.insert(v.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    fn convert_type(&self, custom_type: &CustomType) -> Result<IRType> {
        match custom_type {
            CustomType::Object { name, fields } => {
                let is_union_variant = self.union_variant_names.contains(name);
                let ir_fields = fields.iter().map(|f| self.convert_field(f)).collect();

                Ok(IRType::Struct(IRStruct {
                    name: name.clone(),
                    fields: ir_fields,
                    is_union_variant,
                    doc: None,
                }))
            }

            CustomType::Enum { name, values } => {
                Ok(IRType::Enum(IREnum {
                    name: name.clone(),
                    variants: values.clone(),
                    doc: None,
                }))
            }

            CustomType::Union {
                name,
                type_tag,
                values,
                configs,
            } => {
                let style = configs
                    .as_ref()
                    .and_then(|c| c.union_style.as_ref())
                    .map(|s| match s {
                        UnionStyle::Inline => IRUnionStyle::Inline,
                        UnionStyle::Extern => IRUnionStyle::Extern,
                    })
                    .unwrap_or(IRUnionStyle::Inline);

                let variants = values
                    .iter()
                    .map(|v| {
                        if self.all_type_names.contains(v) {
                            // Reference to a custom type
                            match style {
                                IRUnionStyle::Inline => {
                                    // Will be resolved later during generation
                                    IRUnionVariant::Inline(v.clone(), Vec::new())
                                }
                                IRUnionStyle::Extern => {
                                    IRUnionVariant::Newtype(v.clone(), v.clone())
                                }
                            }
                        } else {
                            // Simple unit variant
                            IRUnionVariant::Unit(v.clone())
                        }
                    })
                    .collect();

                Ok(IRType::Union(IRUnion {
                    name: name.clone(),
                    tag_field: type_tag.clone(),
                    variants,
                    style,
                    doc: None,
                }))
            }

            CustomType::List { name, item_type } => {
                let item = self.convert_field_type(item_type);
                Ok(IRType::TypeAlias(IRTypeAlias {
                    name: name.clone(),
                    target: IRTypeAliasTarget::List(item),
                    doc: None,
                }))
            }

            CustomType::Map {
                name,
                key_type,
                value_type,
            } => {
                let key = self.convert_field_type(key_type);
                let value = self.convert_field_type(value_type);
                Ok(IRType::TypeAlias(IRTypeAlias {
                    name: name.clone(),
                    target: IRTypeAliasTarget::Map(key, value),
                    doc: None,
                }))
            }
        }
    }

    fn convert_field(&self, field: &Field) -> IRField {
        let field_type = self.convert_field_type(&field.field_type);
        let is_boxed = field
            .configs
            .as_ref()
            .and_then(|c| c.rust_type_wrapper.as_ref())
            .is_some();
        let rename = field.configs.as_ref().and_then(|c| c.rename.clone());

        IRField {
            name: field.name.clone(),
            field_type,
            is_optional: field.optional.unwrap_or(false),
            is_boxed,
            rename,
            doc: None,
        }
    }

    fn convert_field_type(&self, type_str: &str) -> IRFieldType {
        if type_str == "Any" {
            return IRFieldType::Any;
        }

        if let Some(primitive) = self.parse_primitive(type_str) {
            return IRFieldType::Primitive(primitive);
        }

        IRFieldType::Custom(type_str.to_owned())
    }

    fn parse_primitive(&self, s: &str) -> Option<IRPrimitive> {
        match s {
            "String" => Some(IRPrimitive::String),
            "Bool" => Some(IRPrimitive::Bool),
            "DateTime" => Some(IRPrimitive::DateTime),
            "UInt32" => Some(IRPrimitive::UInt32),
            "UInt64" => Some(IRPrimitive::UInt64),
            "Int32" => Some(IRPrimitive::Int32),
            "Int64" => Some(IRPrimitive::Int64),
            "Float32" => Some(IRPrimitive::Float32),
            "Float64" => Some(IRPrimitive::Float64),
            _ => None,
        }
    }
}

impl Default for IRBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CustomType {
    pub(crate) fn type_name(&self) -> &str {
        match self {
            CustomType::Object { name, .. } => name,
            CustomType::Enum { name, .. } => name,
            CustomType::Union { name, .. } => name,
            CustomType::List { name, .. } => name,
            CustomType::Map { name, .. } => name,
        }
    }
}
```

**Step 4: Update IR mod.rs**

Edit `codegen/src/code_gen/ir/mod.rs`:

```rust
mod types;
mod builder;

pub use types::*;
pub use builder::*;
```

**Step 5: Run tests**

Run: `cargo test --package fluorite_codegen test_ir_builder`
Expected: PASS

**Step 6: Commit**

```bash
git add codegen/src/code_gen/ir/
git add codegen/tests/rust_code_gen.rs
git commit -m "feat: add IR builder to convert definitions to intermediate representation"
```

---

## Task 5: Add Validation Module

**Files:**
- Create: `codegen/src/code_gen/validation/mod.rs`
- Modify: `codegen/src/code_gen/mod.rs`

**Step 1: Write tests for validation**

Add to `codegen/tests/rust_code_gen.rs`:

```rust
use fluorite_codegen::code_gen::validation::{Validator, ValidationError};
use fluorite_codegen::code_gen::ir::IRBuilder;

#[test]
fn test_validation_detects_missing_type() {
    // Create a definition that references a non-existent type
    let yaml = r#"
configs:
  rust_package: "test"
types:
  - name: Foo
    type: Object
    fields:
      - name: bar
        type: NonExistent
"#;
    let def: Definition = serde_yaml::from_str(yaml).unwrap();
    let schema = IRBuilder::new().build(&vec![def]).unwrap();

    let errors = Validator::new().validate(&schema);
    assert!(!errors.is_empty());
    assert!(errors.iter().any(|e| matches!(e, ValidationError::UnknownType { .. })));
}

#[test]
fn test_validation_passes_for_valid_schema() {
    let d1 = deserialize_definition_file("../examples/users.yml").unwrap();
    let d2 = deserialize_definition_file("../examples/orders.yml").unwrap();
    let schema = IRBuilder::new().build(&vec![d1, d2]).unwrap();

    let errors = Validator::new().validate(&schema);
    assert!(errors.is_empty(), "Expected no errors but got: {:?}", errors);
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --package fluorite_codegen test_validation`
Expected: FAIL - no validation module

**Step 3: Implement validation module**

Create `codegen/src/code_gen/validation/mod.rs`:

```rust
//! Schema validation module
//!
//! Validates IR schemas before code generation to catch errors early.

use std::collections::{HashMap, HashSet};

use crate::code_gen::ir::{
    IRFieldType, IRPackage, IRSchema, IRStruct, IRType, IRTypeAlias, IRTypeAliasTarget,
    IRUnion, IRUnionVariant,
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
    DuplicateType {
        type_name: String,
        package: String,
    },
    /// Circular dependency detected
    CircularDependency {
        cycle: Vec<String>,
    },
    /// Empty enum (no variants)
    EmptyEnum {
        type_name: String,
    },
    /// Empty struct (no fields) - warning level
    EmptyStruct {
        type_name: String,
    },
    /// Union with no variants
    EmptyUnion {
        type_name: String,
    },
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
    NonIdiomaticNaming { type_name: String, field_name: String },
}

/// Schema validator
pub struct Validator {
    /// All known type names (package.name -> type name)
    known_types: HashSet<String>,
    /// Primitive type names
    primitive_types: HashSet<String>,
}

impl Validator {
    pub fn new() -> Self {
        let primitive_types: HashSet<String> = [
            "String", "Bool", "DateTime",
            "UInt32", "UInt64", "Int32", "Int64",
            "Float32", "Float64", "Any",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();

        Self {
            known_types: HashSet::new(),
            primitive_types,
        }
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

        // Check for circular dependencies
        errors.extend(self.check_circular_dependencies(schema, &known_types));

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

    fn validate_struct(
        &self,
        s: &IRStruct,
        known_types: &HashSet<String>,
    ) -> Vec<ValidationError> {
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

    fn validate_union(
        &self,
        u: &IRUnion,
        known_types: &HashSet<String>,
    ) -> Vec<ValidationError> {
        let mut errors = Vec::new();

        if u.variants.is_empty() {
            errors.push(ValidationError::EmptyUnion {
                type_name: u.name.clone(),
            });
        }

        for variant in &u.variants {
            match variant {
                IRUnionVariant::Unit(_) => {}
                IRUnionVariant::Inline(name, _) | IRUnionVariant::Newtype(name, type_ref) => {
                    let type_to_check = match variant {
                        IRUnionVariant::Newtype(_, t) => t,
                        _ => name,
                    };
                    if !known_types.contains(type_to_check) {
                        errors.push(ValidationError::UnknownType {
                            type_name: type_to_check.clone(),
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
            IRFieldType::Map(k, v) => {
                self.get_custom_type_name(k).or_else(|| self.get_custom_type_name(v))
            }
            IRFieldType::Primitive(_) | IRFieldType::Any => None,
        }
    }

    fn check_circular_dependencies(
        &self,
        schema: &IRSchema,
        known_types: &HashSet<String>,
    ) -> Vec<ValidationError> {
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
                if let Some(cycle) = self.detect_cycle(
                    type_name,
                    &deps,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                ) {
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
            IRType::TypeAlias(a) => {
                match &a.target {
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
                }
            }
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
                    if let Some(cycle) = self.detect_cycle(neighbor, deps, visited, rec_stack, path) {
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
```

**Step 4: Update code_gen/mod.rs**

Edit `codegen/src/code_gen/mod.rs`:

```rust
pub mod abi;
pub mod generator;
pub mod rust;
pub mod ts;
pub mod utils;
pub mod ir;
pub mod validation;

pub use abi::*;
pub use generator::*;
```

**Step 5: Run tests**

Run: `cargo test --package fluorite_codegen test_validation`
Expected: PASS

**Step 6: Commit**

```bash
git add codegen/src/code_gen/validation/
git add codegen/src/code_gen/mod.rs
git add codegen/tests/rust_code_gen.rs
git commit -m "feat: add schema validation module with circular dependency detection"
```

---

## Task 6: Create Abstract FileSystem Trait

**Files:**
- Create: `codegen/src/code_gen/fs/mod.rs`
- Modify: `codegen/src/code_gen/mod.rs`

**Step 1: Write tests for filesystem abstraction**

Add to `codegen/tests/rust_code_gen.rs`:

```rust
use fluorite_codegen::code_gen::fs::{FileSystem, MemoryFileSystem};

#[test]
fn test_memory_filesystem_write_and_read() {
    let fs = MemoryFileSystem::new();
    fs.write_file("test/file.txt", b"hello world").unwrap();

    let content = fs.read_file("test/file.txt").unwrap();
    assert_eq!(content, b"hello world");
}

#[test]
fn test_memory_filesystem_append() {
    let fs = MemoryFileSystem::new();
    fs.write_file("test/file.txt", b"hello").unwrap();
    fs.append_file("test/file.txt", b" world").unwrap();

    let content = fs.read_file("test/file.txt").unwrap();
    assert_eq!(content, b"hello world");
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test --package fluorite_codegen test_memory_filesystem`
Expected: FAIL - no fs module

**Step 3: Implement filesystem abstraction**

Create `codegen/src/code_gen/fs/mod.rs`:

```rust
//! Abstract filesystem for code generation
//!
//! Provides both a real filesystem implementation and an in-memory
//! implementation for testing.

use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, RwLock};

use anyhow::Result;

/// Abstract filesystem trait
pub trait FileSystem: Send + Sync {
    /// Write content to a file, creating parent directories as needed
    fn write_file(&self, path: &str, content: &[u8]) -> Result<()>;

    /// Append content to a file, creating it if it doesn't exist
    fn append_file(&self, path: &str, content: &[u8]) -> Result<()>;

    /// Read a file's contents
    fn read_file(&self, path: &str) -> Result<Vec<u8>>;

    /// Check if a file exists
    fn exists(&self, path: &str) -> bool;

    /// Create a directory and all parent directories
    fn create_dir_all(&self, path: &str) -> Result<()>;
}

/// Real filesystem implementation
#[derive(Debug, Default)]
pub struct RealFileSystem;

impl RealFileSystem {
    pub fn new() -> Self {
        Self
    }
}

impl FileSystem for RealFileSystem {
    fn write_file(&self, path: &str, content: &[u8]) -> Result<()> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = File::create(path)?;
        file.write_all(content)?;
        Ok(())
    }

    fn append_file(&self, path: &str, content: &[u8]) -> Result<()> {
        let path = Path::new(path);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        file.write_all(content)?;
        Ok(())
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        Ok(fs::read(path)?)
    }

    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn create_dir_all(&self, path: &str) -> Result<()> {
        fs::create_dir_all(path)?;
        Ok(())
    }
}

/// In-memory filesystem for testing
#[derive(Debug, Clone)]
pub struct MemoryFileSystem {
    files: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryFileSystem {
    pub fn new() -> Self {
        Self {
            files: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get all files in the filesystem (for testing)
    pub fn files(&self) -> HashMap<String, Vec<u8>> {
        self.files.read().unwrap().clone()
    }

    /// Get content of a file as a string (for testing)
    pub fn get_string(&self, path: &str) -> Option<String> {
        self.files
            .read()
            .unwrap()
            .get(path)
            .map(|b| String::from_utf8_lossy(b).to_string())
    }
}

impl Default for MemoryFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystem for MemoryFileSystem {
    fn write_file(&self, path: &str, content: &[u8]) -> Result<()> {
        let mut files = self.files.write().unwrap();
        files.insert(path.to_string(), content.to_vec());
        Ok(())
    }

    fn append_file(&self, path: &str, content: &[u8]) -> Result<()> {
        let mut files = self.files.write().unwrap();
        let entry = files.entry(path.to_string()).or_insert_with(Vec::new);
        entry.extend_from_slice(content);
        Ok(())
    }

    fn read_file(&self, path: &str) -> Result<Vec<u8>> {
        let files = self.files.read().unwrap();
        files
            .get(path)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("File not found: {}", path))
    }

    fn exists(&self, path: &str) -> bool {
        self.files.read().unwrap().contains_key(path)
    }

    fn create_dir_all(&self, _path: &str) -> Result<()> {
        // No-op for memory filesystem
        Ok(())
    }
}

/// Writer that writes to abstract filesystem
pub struct FsWriter {
    fs: Arc<dyn FileSystem>,
    path: String,
    buffer: Vec<u8>,
    append: bool,
}

impl FsWriter {
    pub fn new(fs: Arc<dyn FileSystem>, path: String, append: bool) -> Self {
        Self {
            fs,
            path,
            buffer: Vec::new(),
            append,
        }
    }
}

impl Write for FsWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        let result = if self.append {
            self.fs.append_file(&self.path, &self.buffer)
        } else {
            self.fs.write_file(&self.path, &self.buffer)
        };

        result.map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        self.buffer.clear();
        Ok(())
    }
}

impl Drop for FsWriter {
    fn drop(&mut self) {
        // Flush on drop, ignore errors
        let _ = self.flush();
    }
}
```

**Step 4: Update code_gen/mod.rs**

Edit `codegen/src/code_gen/mod.rs`:

```rust
pub mod abi;
pub mod generator;
pub mod rust;
pub mod ts;
pub mod utils;
pub mod ir;
pub mod validation;
pub mod fs;

pub use abi::*;
pub use generator::*;
```

**Step 5: Run tests**

Run: `cargo test --package fluorite_codegen test_memory_filesystem`
Expected: PASS

**Step 6: Commit**

```bash
git add codegen/src/code_gen/fs/
git add codegen/src/code_gen/mod.rs
git add codegen/tests/rust_code_gen.rs
git commit -m "feat: add abstract filesystem trait with memory implementation for testing"
```

---

## Task 7: Create Askama Templates for Rust Code Generation

**Files:**
- Create: `codegen/templates/rust/struct.rs.j2`
- Create: `codegen/templates/rust/enum.rs.j2`
- Create: `codegen/templates/rust/union.rs.j2`
- Create: `codegen/templates/rust/list_alias.rs.j2`
- Create: `codegen/templates/rust/map_alias.rs.j2`
- Create: `codegen/templates/rust/mod.rs.j2`

**Step 1: Create struct template**

Create `codegen/templates/rust/struct.rs.j2`:

```
{{ derives }}
pub struct {{ name }} {
{% for field in fields %}
{% if field.needs_rename %}
    #[serde(rename = "{{ field.original_name }}")]
{% endif %}
{% if field.is_optional %}
    #[serde(skip_serializing_if = "Option::is_none")]
{% endif %}
    pub {{ field.code_name }}: {{ field.type_str }},
{% endfor %}
}

```

**Step 2: Create enum template**

Create `codegen/templates/rust/enum.rs.j2`:

```
{{ derives }}
pub enum {{ name }} {
{% for variant in variants %}
    {{ variant }},
{% endfor %}
}

```

**Step 3: Create union template**

Create `codegen/templates/rust/union.rs.j2`:

```
{{ derives }}
#[serde(tag = "{{ tag_field }}")]
pub enum {{ name }} {
{% for variant in variants %}
{% match variant %}
{% when UnionVariantTemplate::Unit with (name) %}
    {{ name }},
{% when UnionVariantTemplate::Inline with { name, fields } %}
    {{ name }} {
{% for field in fields %}
{% if field.needs_rename %}
        #[serde(rename = "{{ field.original_name }}")]
{% endif %}
        {{ field.code_name }}: {{ field.type_str }},
{% endfor %}
    },
{% when UnionVariantTemplate::Newtype with { name, type_str } %}
    {{ name }}({{ type_str }}),
{% endmatch %}
{% endfor %}
}

```

**Step 4: Create list alias template**

Create `codegen/templates/rust/list_alias.rs.j2`:

```
pub type {{ name }} = Vec<{{ item_type }}>;

```

**Step 5: Create map alias template**

Create `codegen/templates/rust/map_alias.rs.j2`:

```
use std::collections::HashMap;

pub type {{ name }} = HashMap<{{ key_type }}, {{ value_type }}>;

```

**Step 6: Create mod template**

Create `codegen/templates/rust/mod.rs.j2`:

```
{% for module in modules %}
mod {{ module.file_name }};
pub use crate::{{ package }}::{{ module.file_name }}::*;
{% endfor %}
```

**Step 7: Verify templates are valid**

Run: `cargo build --package fluorite_codegen`
Expected: Build succeeds (askama validates templates at compile time)

**Step 8: Commit**

```bash
git add codegen/templates/
git commit -m "feat: add askama templates for Rust code generation"
```

---

## Task 8: Create Template Renderer Using Askama

**Files:**
- Create: `codegen/src/code_gen/rust/templates.rs`
- Modify: `codegen/src/code_gen/rust/mod.rs`

**Step 1: Write test for template rendering**

Add to `codegen/tests/rust_code_gen.rs`:

```rust
use fluorite_codegen::code_gen::rust::templates::{StructTemplate, FieldTemplate};

#[test]
fn test_struct_template_renders() {
    let template = StructTemplate {
        derives: "#[derive(Debug, Clone)]".to_string(),
        name: "User".to_string(),
        fields: vec![
            FieldTemplate {
                code_name: "first_name".to_string(),
                original_name: "first_name".to_string(),
                type_str: "String".to_string(),
                is_optional: false,
                needs_rename: false,
            },
        ],
    };

    let rendered = template.render().unwrap();
    assert!(rendered.contains("pub struct User"));
    assert!(rendered.contains("pub first_name: String"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package fluorite_codegen test_struct_template_renders`
Expected: FAIL - no templates module

**Step 3: Implement template structs**

Create `codegen/src/code_gen/rust/templates.rs`:

```rust
//! Askama templates for Rust code generation

use askama::Template;

/// Template for rendering a struct
#[derive(Template)]
#[template(path = "rust/struct.rs.j2")]
pub struct StructTemplate {
    pub derives: String,
    pub name: String,
    pub fields: Vec<FieldTemplate>,
}

/// Field information for templates
#[derive(Clone)]
pub struct FieldTemplate {
    pub code_name: String,
    pub original_name: String,
    pub type_str: String,
    pub is_optional: bool,
    pub needs_rename: bool,
}

impl FieldTemplate {
    pub fn code_name(&self) -> &str {
        &self.code_name
    }

    pub fn original_name(&self) -> &str {
        &self.original_name
    }

    pub fn needs_rename(&self) -> bool {
        self.needs_rename
    }
}

/// Template for rendering an enum
#[derive(Template)]
#[template(path = "rust/enum.rs.j2")]
pub struct EnumTemplate {
    pub derives: String,
    pub name: String,
    pub variants: Vec<String>,
}

/// Union variant types for template
#[derive(Clone)]
pub enum UnionVariantTemplate {
    Unit(String),
    Inline { name: String, fields: Vec<FieldTemplate> },
    Newtype { name: String, type_str: String },
}

/// Template for rendering a union (tagged enum)
#[derive(Template)]
#[template(path = "rust/union.rs.j2")]
pub struct UnionTemplate {
    pub derives: String,
    pub name: String,
    pub tag_field: String,
    pub variants: Vec<UnionVariantTemplate>,
}

/// Template for rendering a list type alias
#[derive(Template)]
#[template(path = "rust/list_alias.rs.j2")]
pub struct ListAliasTemplate {
    pub name: String,
    pub item_type: String,
}

/// Template for rendering a map type alias
#[derive(Template)]
#[template(path = "rust/map_alias.rs.j2")]
pub struct MapAliasTemplate {
    pub name: String,
    pub key_type: String,
    pub value_type: String,
}

/// Template for rendering a module file
#[derive(Template)]
#[template(path = "rust/mod.rs.j2")]
pub struct ModTemplate {
    pub package: String,
    pub modules: Vec<ModuleEntry>,
}

#[derive(Clone)]
pub struct ModuleEntry {
    pub file_name: String,
}
```

**Step 4: Update rust/mod.rs**

Add to `codegen/src/code_gen/rust/mod.rs`:

```rust
mod pre_processor;
pub use pre_processor::*;
mod package_writer;
pub use package_writer::*;
mod type_writer;
pub use type_writer::*;
mod options;
pub use options::*;
mod context;
pub use context::*;
pub mod templates;

use super::abi::{CodeGenProvider, PackageWriter, PreProcessor};

// ... rest of file unchanged
```

**Step 5: Run tests**

Run: `cargo test --package fluorite_codegen test_struct_template_renders`
Expected: PASS

**Step 6: Commit**

```bash
git add codegen/src/code_gen/rust/templates.rs
git add codegen/src/code_gen/rust/mod.rs
git commit -m "feat: add askama template structs for Rust code generation"
```

---

## Task 9: Create New Template-Based Generator

**Files:**
- Create: `codegen/src/code_gen/rust/template_generator.rs`
- Modify: `codegen/src/code_gen/rust/mod.rs`

**Step 1: Write test for template generator**

Add to `codegen/tests/rust_code_gen.rs`:

```rust
use fluorite_codegen::code_gen::rust::RustTemplateGenerator;
use fluorite_codegen::code_gen::fs::MemoryFileSystem;
use std::sync::Arc;

#[test]
fn test_template_generator_produces_valid_rust() {
    let d1 = deserialize_definition_file("../examples/users.yml").unwrap();
    let d2 = deserialize_definition_file("../examples/orders.yml").unwrap();

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());

    generator.generate(&vec![d1, d2]).unwrap();

    // Check that files were generated
    let files = fs.files();
    assert!(files.keys().any(|k| k.contains("mod.rs")));

    // Check content of generated struct
    let users_mod = fs.get_string("/output/protocols/users/mod.rs").unwrap();
    assert!(users_mod.contains("pub struct User"));
    assert!(users_mod.contains("first_name: String"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package fluorite_codegen test_template_generator`
Expected: FAIL - no RustTemplateGenerator

**Step 3: Implement template generator**

Create `codegen/src/code_gen/rust/template_generator.rs`:

```rust
//! Template-based Rust code generator using askama templates

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use askama::Template;

use crate::code_gen::fs::FileSystem;
use crate::code_gen::ir::{
    IRBuilder, IRField, IRFieldType, IRPrimitive, IRSchema, IRStruct, IRType,
    IRTypeAlias, IRTypeAliasTarget, IRUnion, IRUnionStyle, IRUnionVariant,
};
use crate::code_gen::utils::to_snake_case;
use crate::code_gen::validation::{ValidationError, Validator};
use crate::definitions::Definition;

use super::templates::{
    EnumTemplate, FieldTemplate, ListAliasTemplate, MapAliasTemplate, ModTemplate,
    ModuleEntry, StructTemplate, UnionTemplate, UnionVariantTemplate,
};
use super::RustOptions;

/// Template-based Rust code generator
pub struct RustTemplateGenerator {
    options: RustOptions,
    fs: Arc<dyn FileSystem>,
}

impl RustTemplateGenerator {
    pub fn new(options: RustOptions, fs: Arc<dyn FileSystem>) -> Self {
        Self { options, fs }
    }

    /// Generate Rust code from definitions
    pub fn generate(&self, definitions: &[Definition]) -> Result<()> {
        // Build IR
        let schema = IRBuilder::new().build(definitions)?;

        // Validate
        let errors = Validator::new().validate(&schema);
        if !errors.is_empty() {
            return Err(self.format_validation_errors(&errors));
        }

        // Resolve union variant fields
        let schema = self.resolve_union_variants(schema)?;

        // Generate code for each package
        for (package_name, package) in &schema.packages {
            self.generate_package(package_name, &package.types, &schema)?;
        }

        Ok(())
    }

    fn resolve_union_variants(&self, mut schema: IRSchema) -> Result<IRSchema> {
        // Collect all structs for lookup
        let mut structs: HashMap<String, IRStruct> = HashMap::new();
        for package in schema.packages.values() {
            for ir_type in &package.types {
                if let IRType::Struct(s) = ir_type {
                    structs.insert(s.name.clone(), s.clone());
                }
            }
        }

        // Resolve inline union variants
        for package in schema.packages.values_mut() {
            for ir_type in &mut package.types {
                if let IRType::Union(union) = ir_type {
                    if union.style == IRUnionStyle::Inline {
                        let mut resolved_variants = Vec::new();
                        for variant in &union.variants {
                            match variant {
                                IRUnionVariant::Inline(name, _) => {
                                    if let Some(struct_def) = structs.get(name) {
                                        resolved_variants.push(IRUnionVariant::Inline(
                                            name.clone(),
                                            struct_def.fields.clone(),
                                        ));
                                    } else {
                                        // Treat as unit variant if struct not found
                                        resolved_variants.push(IRUnionVariant::Unit(name.clone()));
                                    }
                                }
                                other => resolved_variants.push(other.clone()),
                            }
                        }
                        union.variants = resolved_variants;
                    }
                }
            }
        }

        Ok(schema)
    }

    fn generate_package(
        &self,
        package_name: &str,
        types: &[IRType],
        schema: &IRSchema,
    ) -> Result<()> {
        let package_path = package_name.replace('.', "/");
        let output_path = format!("{}/{}", self.options.output_dir, package_path);

        self.fs.create_dir_all(&output_path)?;

        if self.options.single_file {
            // Generate all types in mod.rs
            let mod_path = format!("{}/mod.rs", output_path);
            let mut content = String::new();

            for ir_type in types.iter().filter(|t| !t.is_internal()) {
                content.push_str(&self.render_type(ir_type, schema)?);
            }

            self.fs.write_file(&mod_path, content.as_bytes())?;
        } else {
            // Generate each type in separate file + mod.rs
            let mut modules = Vec::new();

            for ir_type in types.iter().filter(|t| !t.is_internal()) {
                let file_name = to_snake_case(ir_type.name());
                let file_path = format!("{}/{}.rs", output_path, file_name);
                let content = self.render_type(ir_type, schema)?;

                self.fs.write_file(&file_path, content.as_bytes())?;
                modules.push(ModuleEntry { file_name });
            }

            // Generate mod.rs
            let mod_template = ModTemplate {
                package: package_path.replace('/', "::"),
                modules,
            };
            let mod_content = mod_template.render()?;
            let mod_path = format!("{}/mod.rs", output_path);
            self.fs.write_file(&mod_path, mod_content.as_bytes())?;
        }

        Ok(())
    }

    fn render_type(&self, ir_type: &IRType, schema: &IRSchema) -> Result<String> {
        match ir_type {
            IRType::Struct(s) => self.render_struct(s, schema),
            IRType::Enum(e) => self.render_enum(e),
            IRType::Union(u) => self.render_union(u, schema),
            IRType::TypeAlias(a) => self.render_type_alias(a, schema),
        }
    }

    fn render_struct(&self, s: &IRStruct, schema: &IRSchema) -> Result<String> {
        let fields: Vec<FieldTemplate> = s
            .fields
            .iter()
            .map(|f| self.convert_field(f, schema))
            .collect::<Result<Vec<_>>>()?;

        let template = StructTemplate {
            derives: self.options.get_derives_string(),
            name: s.name.clone(),
            fields,
        };

        Ok(template.render()?)
    }

    fn render_enum(&self, e: &crate::code_gen::ir::IREnum) -> Result<String> {
        let template = EnumTemplate {
            derives: self.options.get_derives_string(),
            name: e.name.clone(),
            variants: e.variants.clone(),
        };

        Ok(template.render()?)
    }

    fn render_union(&self, u: &IRUnion, schema: &IRSchema) -> Result<String> {
        let variants: Vec<UnionVariantTemplate> = u
            .variants
            .iter()
            .map(|v| self.convert_union_variant(v, schema))
            .collect::<Result<Vec<_>>>()?;

        let template = UnionTemplate {
            derives: self.options.get_derives_string(),
            name: u.name.clone(),
            tag_field: u.tag_field.clone(),
            variants,
        };

        Ok(template.render()?)
    }

    fn render_type_alias(&self, a: &IRTypeAlias, schema: &IRSchema) -> Result<String> {
        match &a.target {
            IRTypeAliasTarget::List(item_type) => {
                let template = ListAliasTemplate {
                    name: a.name.clone(),
                    item_type: self.format_type(item_type, schema)?,
                };
                Ok(template.render()?)
            }
            IRTypeAliasTarget::Map(key_type, value_type) => {
                let template = MapAliasTemplate {
                    name: a.name.clone(),
                    key_type: self.format_type(key_type, schema)?,
                    value_type: self.format_type(value_type, schema)?,
                };
                Ok(template.render()?)
            }
        }
    }

    fn convert_field(&self, field: &IRField, schema: &IRSchema) -> Result<FieldTemplate> {
        let mut type_str = self.format_type(&field.field_type, schema)?;

        if field.is_boxed {
            type_str = format!("Box<{}>", type_str);
        }
        if field.is_optional {
            type_str = format!("Option<{}>", type_str);
        }

        Ok(FieldTemplate {
            code_name: field.code_name().to_string(),
            original_name: field.original_name().to_string(),
            type_str,
            is_optional: field.is_optional,
            needs_rename: field.needs_rename(),
        })
    }

    fn convert_union_variant(
        &self,
        variant: &IRUnionVariant,
        schema: &IRSchema,
    ) -> Result<UnionVariantTemplate> {
        match variant {
            IRUnionVariant::Unit(name) => Ok(UnionVariantTemplate::Unit(name.clone())),
            IRUnionVariant::Inline(name, fields) => {
                let field_templates: Vec<FieldTemplate> = fields
                    .iter()
                    .map(|f| self.convert_field(f, schema))
                    .collect::<Result<Vec<_>>>()?;

                Ok(UnionVariantTemplate::Inline {
                    name: name.clone(),
                    fields: field_templates,
                })
            }
            IRUnionVariant::Newtype(name, type_ref) => {
                let type_str = self.get_fqn_for_custom_type(type_ref, schema)?;
                Ok(UnionVariantTemplate::Newtype {
                    name: name.clone(),
                    type_str,
                })
            }
        }
    }

    fn format_type(&self, field_type: &IRFieldType, schema: &IRSchema) -> Result<String> {
        match field_type {
            IRFieldType::Primitive(p) => Ok(self.format_primitive(*p)),
            IRFieldType::Custom(name) => self.get_fqn_for_custom_type(name, schema),
            IRFieldType::Any => Ok(self.options.any_type.clone()),
            IRFieldType::List(item) => {
                let item_str = self.format_type(item, schema)?;
                Ok(format!("Vec<{}>", item_str))
            }
            IRFieldType::Map(key, value) => {
                let key_str = self.format_type(key, schema)?;
                let value_str = self.format_type(value, schema)?;
                Ok(format!("std::collections::HashMap<{}, {}>", key_str, value_str))
            }
        }
    }

    fn format_primitive(&self, p: IRPrimitive) -> String {
        match p {
            IRPrimitive::String => "String".to_string(),
            IRPrimitive::Bool => "bool".to_string(),
            IRPrimitive::DateTime => "DateTime".to_string(),
            IRPrimitive::UInt32 => "u32".to_string(),
            IRPrimitive::UInt64 => "u64".to_string(),
            IRPrimitive::Int32 => "i32".to_string(),
            IRPrimitive::Int64 => "i64".to_string(),
            IRPrimitive::Float32 => "f32".to_string(),
            IRPrimitive::Float64 => "f64".to_string(),
        }
    }

    fn get_fqn_for_custom_type(&self, type_name: &str, schema: &IRSchema) -> Result<String> {
        // Find the type in schema to get its package
        for (package_name, package) in &schema.packages {
            for ir_type in &package.types {
                if ir_type.name() == type_name {
                    let package_path = package_name.replace('.', "::");
                    return Ok(format!("crate::{}::{}", package_path, type_name));
                }
            }
        }

        Err(anyhow!("Unknown type: {}", type_name))
    }

    fn format_validation_errors(&self, errors: &[ValidationError]) -> anyhow::Error {
        let messages: Vec<String> = errors
            .iter()
            .map(|e| match e {
                ValidationError::UnknownType {
                    type_name,
                    referenced_from,
                    field_name,
                } => {
                    if let Some(field) = field_name {
                        format!(
                            "Unknown type '{}' in field '{}' of '{}'",
                            type_name, field, referenced_from
                        )
                    } else {
                        format!(
                            "Unknown type '{}' referenced from '{}'",
                            type_name, referenced_from
                        )
                    }
                }
                ValidationError::DuplicateType { type_name, package } => {
                    format!("Duplicate type '{}' in package '{}'", type_name, package)
                }
                ValidationError::CircularDependency { cycle } => {
                    format!("Circular dependency: {}", cycle.join(" -> "))
                }
                ValidationError::EmptyEnum { type_name } => {
                    format!("Empty enum '{}'", type_name)
                }
                ValidationError::EmptyStruct { type_name } => {
                    format!("Empty struct '{}'", type_name)
                }
                ValidationError::EmptyUnion { type_name } => {
                    format!("Empty union '{}'", type_name)
                }
                ValidationError::InvalidUnionVariant {
                    union_name,
                    variant_name,
                    reason,
                } => {
                    format!(
                        "Invalid variant '{}' in union '{}': {}",
                        variant_name, union_name, reason
                    )
                }
            })
            .collect();

        anyhow!("Validation errors:\n  - {}", messages.join("\n  - "))
    }
}
```

**Step 4: Update rust/mod.rs exports**

Edit `codegen/src/code_gen/rust/mod.rs`:

```rust
mod pre_processor;
pub use pre_processor::*;
mod package_writer;
pub use package_writer::*;
mod type_writer;
pub use type_writer::*;
mod options;
pub use options::*;
mod context;
pub use context::*;
pub mod templates;
mod template_generator;
pub use template_generator::*;

use super::abi::{CodeGenProvider, PackageWriter, PreProcessor};

pub struct RustProvider {
    options: RustOptions,
}

impl RustProvider {
    pub fn new(options: RustOptions) -> Self {
        Self { options }
    }
}
impl CodeGenProvider<RustContext> for RustProvider {
    fn get_pre_processor(&self) -> Box<dyn PreProcessor<RustContext>> {
        Box::new(RustPreProcessor {
            options: self.options.clone(),
        })
    }

    fn get_package_writer(&self) -> Option<Box<dyn PackageWriter<RustContext>>> {
        Some(Box::new(RustPackageWriter {}))
    }

    fn get_object_writer(&self) -> Box<dyn super::abi::ObjectWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }

    fn get_enum_writer(&self) -> Box<dyn super::abi::EnumWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }

    fn get_union_writer(&self) -> Box<dyn super::abi::UnionWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }

    fn get_list_writer(&self) -> Box<dyn super::abi::ListWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }

    fn get_map_writer(&self) -> Box<dyn super::abi::MapWriter<RustContext>> {
        Box::new(RustTypeWriter {})
    }
}
```

**Step 5: Run tests**

Run: `cargo test --package fluorite_codegen test_template_generator`
Expected: PASS

**Step 6: Commit**

```bash
git add codegen/src/code_gen/rust/template_generator.rs
git add codegen/src/code_gen/rust/mod.rs
git add codegen/tests/rust_code_gen.rs
git commit -m "feat: add template-based Rust code generator using askama"
```

---

## Task 10: Update Public API to Use New Generator

**Files:**
- Modify: `codegen/src/utils.rs`
- Modify: `codegen/src/lib.rs`

**Step 1: Update the compile functions**

Replace `codegen/src/utils.rs`:

```rust
use std::fs;
use std::sync::Arc;

use crate::code_gen::fs::RealFileSystem;
use crate::code_gen::rust::{RustOptions, RustTemplateGenerator};
use crate::definitions::Definition;

/// Compile YAML definitions to Rust code with custom options
pub fn compile_with_options(options: RustOptions, yaml_files: &[&str]) -> anyhow::Result<()> {
    let definitions: Vec<Definition> = yaml_files
        .iter()
        .map(|path| {
            let content = fs::read_to_string(path)?;
            let def: Definition = serde_yaml::from_str(&content)?;
            Ok(def)
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let fs = Arc::new(RealFileSystem::new());
    let generator = RustTemplateGenerator::new(options, fs);
    generator.generate(&definitions)?;

    Ok(())
}

/// Compile YAML definitions to Rust code with default options
pub fn compile(output_dir: &str, yaml_files: &[&str]) -> anyhow::Result<()> {
    let options = RustOptions::new(output_dir.to_string());
    compile_with_options(options, yaml_files)
}
```

**Step 2: Update existing test to use new API**

Update `codegen/tests/rust_code_gen.rs` - the test_rust_code_gen function:

```rust
#[test]
fn test_rust_code_gen() -> anyhow::Result<()> {
    use fluorite_codegen::code_gen::rust::RustTemplateGenerator;
    use fluorite_codegen::code_gen::fs::MemoryFileSystem;
    use std::sync::Arc;

    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/tmp/test_fluorite".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&vec![d1, d2])?;

    // Verify key files exist
    let files = fs.files();
    assert!(files.keys().any(|k| k.contains("users/mod.rs")));
    assert!(files.keys().any(|k| k.contains("orders/mod.rs")));

    Ok(())
}
```

**Step 3: Run all tests**

Run: `cargo test --package fluorite_codegen`
Expected: All tests pass

**Step 4: Commit**

```bash
git add codegen/src/utils.rs
git add codegen/tests/rust_code_gen.rs
git commit -m "feat: update public API to use template-based generator"
```

---

## Task 11: Update CLI to Use New Generator

**Files:**
- Modify: `codegen/src/main.rs`

**Step 1: Read current main.rs**

Read the file first to understand current structure.

**Step 2: Update main.rs to use new generator**

Replace `codegen/src/main.rs`:

```rust
use std::fs;
use std::sync::Arc;

use clap::{Parser, Subcommand};

use fluorite_codegen::code_gen::fs::RealFileSystem;
use fluorite_codegen::code_gen::rust::{RustOptions, RustTemplateGenerator, Visibility};
use fluorite_codegen::definitions::Definition;

#[derive(Parser)]
#[command(name = "fluorite")]
#[command(about = "Code generator from YAML schema definitions")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate Rust code from YAML definitions
    Rust {
        /// Input YAML files
        #[arg(short, long, required = true, num_args = 1..)]
        inputs: Vec<String>,

        /// Output directory
        #[arg(short, long)]
        output: String,

        /// Generate all types in a single mod.rs file per package
        #[arg(long, default_value = "true")]
        single_file: bool,

        /// Custom type to use for 'Any' fields
        #[arg(long, default_value = "fluorite::Any")]
        any_type: String,

        /// Custom derives (comma-separated, replaces defaults)
        #[arg(long)]
        derives: Option<String>,

        /// Additional derives to add to defaults (comma-separated)
        #[arg(long)]
        extra_derives: Option<String>,

        /// Generate derive_new::new implementation
        #[arg(long, default_value = "true")]
        generate_new: bool,

        /// Visibility for generated types (public, pub_crate, private)
        #[arg(long, default_value = "public")]
        visibility: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Rust {
            inputs,
            output,
            single_file,
            any_type,
            derives,
            extra_derives,
            generate_new,
            visibility,
        } => {
            // Parse definitions
            let definitions: Vec<Definition> = inputs
                .iter()
                .map(|path| {
                    let content = fs::read_to_string(path)?;
                    let def: Definition = serde_yaml::from_str(&content)?;
                    Ok(def)
                })
                .collect::<anyhow::Result<Vec<_>>>()?;

            // Build options
            let mut options = RustOptions::new(output)
                .with_single_file(single_file)
                .with_any_type(&any_type)
                .with_generate_new(generate_new);

            // Handle derives
            if let Some(custom_derives) = derives {
                let derives: Vec<String> = custom_derives
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                options = options.with_derives(derives);
            }

            if let Some(extra) = extra_derives {
                let extra: Vec<String> = extra
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                options = options.with_additional_derives(extra);
            }

            // Handle visibility
            let vis = match visibility.to_lowercase().as_str() {
                "public" | "pub" => Visibility::Public,
                "pub_crate" | "pub(crate)" => Visibility::PublicCrate,
                "private" => Visibility::Private,
                _ => {
                    eprintln!("Unknown visibility '{}', using 'public'", visibility);
                    Visibility::Public
                }
            };
            options = options.with_visibility(vis);

            // Generate
            let fs = Arc::new(RealFileSystem::new());
            let generator = RustTemplateGenerator::new(options, fs);
            generator.generate(&definitions)?;

            println!("Code generation complete!");
        }
    }

    Ok(())
}
```

**Step 3: Test CLI**

Run: `cargo run --package fluorite_codegen --bin fluorite -- rust --inputs examples/users.yml examples/orders.yml --output /tmp/fluorite_test`
Expected: "Code generation complete!" and files in /tmp/fluorite_test

**Step 4: Commit**

```bash
git add codegen/src/main.rs
git commit -m "feat: update CLI to use template-based generator with new options"
```

---

## Task 12: Clean Up Old Code and Final Integration Test

**Files:**
- Remove unused code from old implementation
- Add comprehensive integration test

**Step 1: Add comprehensive integration test**

Add to `codegen/tests/rust_code_gen.rs`:

```rust
#[test]
fn test_full_integration() -> anyhow::Result<()> {
    use fluorite_codegen::code_gen::rust::RustTemplateGenerator;
    use fluorite_codegen::code_gen::fs::MemoryFileSystem;
    use std::sync::Arc;

    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned())
        .with_derives(vec![
            "Debug".to_string(),
            "Clone".to_string(),
            "serde::Serialize".to_string(),
            "serde::Deserialize".to_string(),
        ])
        .with_generate_new(false);

    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&vec![d1, d2])?;

    // Verify users package
    let users_content = fs.get_string("/output/protocols/users/mod.rs")
        .expect("Users module should exist");

    assert!(users_content.contains("pub struct User"), "Should have User struct");
    assert!(users_content.contains("first_name: String"), "Should have first_name field");
    assert!(users_content.contains("pub enum Gender"), "Should have Gender enum");
    assert!(users_content.contains("Male"), "Should have Male variant");
    assert!(!users_content.contains("derive_new::new"), "Should not have derive_new");

    // Verify orders package
    let orders_content = fs.get_string("/output/protocols/orders/mod.rs")
        .expect("Orders module should exist");

    assert!(orders_content.contains("pub struct Order"), "Should have Order struct");
    assert!(orders_content.contains("pub type OrderList = Vec<"), "Should have OrderList");
    assert!(orders_content.contains("pub type OrderMap = HashMap<"), "Should have OrderMap");
    assert!(orders_content.contains("#[serde(tag = \"type\")]"), "Should have tagged union");
    assert!(orders_content.contains("pub enum Address"), "Should have Address union");

    // Verify field renaming
    assert!(orders_content.contains("#[serde(rename = \"type\")]"), "Should have rename attribute");
    assert!(orders_content.contains("order_type: String"), "Should use renamed field");

    // Verify optional + boxed fields
    assert!(orders_content.contains("Option<Box<"), "Should have optional boxed field");

    Ok(())
}

#[test]
fn test_multi_file_mode() -> anyhow::Result<()> {
    use fluorite_codegen::code_gen::rust::RustTemplateGenerator;
    use fluorite_codegen::code_gen::fs::MemoryFileSystem;
    use std::sync::Arc;

    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned())
        .with_single_file(false);

    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&vec![d1])?;

    let files = fs.files();

    // Should have separate files
    assert!(files.contains_key("/output/protocols/users/user.rs"), "Should have user.rs");
    assert!(files.contains_key("/output/protocols/users/gender.rs"), "Should have gender.rs");
    assert!(files.contains_key("/output/protocols/users/mod.rs"), "Should have mod.rs");

    // mod.rs should have module declarations
    let mod_content = fs.get_string("/output/protocols/users/mod.rs").unwrap();
    assert!(mod_content.contains("mod user;"), "Should declare user module");
    assert!(mod_content.contains("mod gender;"), "Should declare gender module");
    assert!(mod_content.contains("pub use"), "Should have pub use");

    Ok(())
}
```

**Step 2: Run all tests**

Run: `cargo test --package fluorite_codegen`
Expected: All tests pass

**Step 3: Run clippy**

Run: `cargo clippy --package fluorite_codegen -- -D warnings`
Expected: No warnings

**Step 4: Commit**

```bash
git add codegen/tests/rust_code_gen.rs
git commit -m "test: add comprehensive integration tests for template generator"
```

---

## Task 13: Update Documentation

**Files:**
- Modify: `CLAUDE.md`

**Step 1: Update CLAUDE.md with new architecture info**

Add section to CLAUDE.md:

```markdown
## New Template-Based Architecture (v2)

The code generator now uses a template-based approach with askama:

### Key Components

1. **IR (Intermediate Representation)** - `codegen/src/code_gen/ir/`
   - Language-agnostic representation of types
   - `IRBuilder` converts YAML definitions to IR
   - Separates parsing from code generation

2. **Validation** - `codegen/src/code_gen/validation/`
   - Validates schemas before generation
   - Detects: unknown types, circular dependencies, empty types

3. **Templates** - `codegen/templates/rust/`
   - Askama templates for Rust code output
   - Compile-time checked for correctness

4. **FileSystem Abstraction** - `codegen/src/code_gen/fs/`
   - `FileSystem` trait for I/O operations
   - `MemoryFileSystem` for testing
   - `RealFileSystem` for production

5. **RustTemplateGenerator** - `codegen/src/code_gen/rust/template_generator.rs`
   - Main entry point for Rust code generation
   - Uses IR + Validation + Templates

### Configuration Options

```rust
RustOptions::new(output_dir)
    .with_single_file(true)           // All types in mod.rs
    .with_any_type("serde_json::Value") // Custom Any type
    .with_derives(vec!["Debug", ...])  // Custom derives
    .with_additional_derives(vec![...]) // Extra derives
    .with_generate_new(true)           // derive_new::new
    .with_visibility(Visibility::Public)
```

### Adding a New Language

1. Create `codegen/templates/<lang>/` with templates
2. Create `codegen/src/code_gen/<lang>/template_generator.rs`
3. Implement type formatting and FQN resolution for the language
4. Add CLI subcommand
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with new template-based architecture"
```

---

## Summary

This plan implements:

1. **Configurable Derives** - `RustOptions` with `with_derives()`, `with_additional_derives()`
2. **Reduced Duplication** - Askama templates replace repetitive string formatting
3. **Validation Phase** - `Validator` checks for unknown types, circular deps, etc.
4. **Language-Agnostic IR** - `IRSchema`, `IRType`, etc. separate parsing from generation
5. **Consistent Indentation** - Templates handle indentation naturally
6. **Template-Based Generation** - Askama templates in `codegen/templates/rust/`
7. **Abstract FileSystem** - `FileSystem` trait with `MemoryFileSystem` for testing
8. **Extended Configuration** - Visibility, generate_new, custom derives
9. **Simplified Type Dict Building** - IR builder replaces two-pass algorithm

The old `CodeGenerator`, `RustTypeWriter`, and trait-based plugin system remain for backward compatibility but the new `RustTemplateGenerator` is the recommended approach.
