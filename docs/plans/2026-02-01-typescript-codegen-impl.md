# TypeScript Code Generation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add TypeScript code generation to Fluorite that generates idiomatic TypeScript interfaces, enums, and discriminated unions from YAML schemas.

**Architecture:** Follow the same pattern as Rust: reuse IRBuilder for schema parsing, Validator for validation, and FileSystem for I/O. Create TsTemplateGenerator with Askama templates for TypeScript output. Extend CLI with `ts` subcommand.

**Tech Stack:** Rust, Askama templates, clap CLI, TypeScript (for E2E tests)

---

## Task 1: Create TypeScript Options

**Files:**
- Create: `codegen/src/code_gen/ts/options.rs`

**Step 1: Write the failing test**

```rust
// In codegen/tests/ts_code_gen.rs (create this file)
use fluorite_codegen::code_gen::ts::TypeScriptOptions;

#[test]
fn test_typescript_options_default() {
    let options = TypeScriptOptions::new("/output".to_string());

    assert_eq!(options.output_dir, "/output");
    assert!(!options.single_file);
    assert_eq!(options.any_type, "unknown");
    assert!(!options.use_readonly);
}

#[test]
fn test_typescript_options_builder() {
    let options = TypeScriptOptions::new("/output".to_string())
        .with_single_file(true)
        .with_any_type("any")
        .with_readonly(true);

    assert!(options.single_file);
    assert_eq!(options.any_type, "any");
    assert!(options.use_readonly);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package fluorite_codegen test_typescript_options`
Expected: FAIL with "unresolved import"

**Step 3: Write implementation**

```rust
// codegen/src/code_gen/ts/options.rs
use crate::code_gen::utils::to_camel_case;

#[derive(Debug, Clone)]
pub struct TypeScriptOptions {
    pub output_dir: String,
    pub single_file: bool,
    pub any_type: String,
    pub use_readonly: bool,
}

impl TypeScriptOptions {
    pub fn new(output_dir: String) -> Self {
        Self {
            output_dir,
            single_file: false,
            any_type: "unknown".to_owned(),
            use_readonly: false,
        }
    }

    pub fn with_single_file(mut self, single_file: bool) -> Self {
        self.single_file = single_file;
        self
    }

    pub fn with_any_type(mut self, any_type: &str) -> Self {
        self.any_type = any_type.to_owned();
        self
    }

    pub fn with_readonly(mut self, use_readonly: bool) -> Self {
        self.use_readonly = use_readonly;
        self
    }

    pub fn type_to_file_name(&self, type_name: &str) -> String {
        to_camel_case(type_name)
    }
}
```

**Step 4: Create the module file**

```rust
// codegen/src/code_gen/ts/mod.rs
mod options;
pub use options::*;
```

**Step 5: Export in parent module**

Add to `codegen/src/code_gen/mod.rs`:
```rust
pub mod ts;
```

**Step 6: Add to_camel_case utility**

Add to `codegen/src/code_gen/utils.rs`:
```rust
pub fn to_camel_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;

    for (i, c) in s.chars().enumerate() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else if i == 0 {
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
```

**Step 7: Run test to verify it passes**

Run: `cargo test --package fluorite_codegen test_typescript_options`
Expected: PASS

**Step 8: Commit**

```bash
git add codegen/src/code_gen/ts/ codegen/src/code_gen/utils.rs codegen/src/code_gen/mod.rs
git commit -m "feat(ts): add TypeScriptOptions configuration"
```

---

## Task 2: Create TypeScript Templates

**Files:**
- Create: `codegen/templates/ts/interface.ts.j2`
- Create: `codegen/templates/ts/enum.ts.j2`
- Create: `codegen/templates/ts/union.ts.j2`
- Create: `codegen/templates/ts/type_alias.ts.j2`
- Create: `codegen/templates/ts/index.ts.j2`
- Create: `codegen/src/code_gen/ts/templates.rs`

**Step 1: Create interface template**

```jinja2
{# codegen/templates/ts/interface.ts.j2 #}
export interface {{ name }} {
{%- for field in fields %}
{%- if use_readonly %}
  readonly {{ field.code_name }}{% if field.is_optional %}?{% endif %}: {{ field.type_str }};
{%- else %}
  {{ field.code_name }}{% if field.is_optional %}?{% endif %}: {{ field.type_str }};
{%- endif %}
{%- endfor %}
}

```

**Step 2: Create enum template**

```jinja2
{# codegen/templates/ts/enum.ts.j2 #}
export enum {{ name }} {
{%- for variant in variants %}
  {{ variant }} = "{{ variant }}",
{%- endfor %}
}

```

**Step 3: Create union template**

```jinja2
{# codegen/templates/ts/union.ts.j2 #}
export type {{ name }} =
{%- for variant in variants %}
{%- match variant %}
{%- when TsUnionVariantTemplate::Unit with (name) %}
  | { {{ tag_field }}: "{{ name }}" }
{%- when TsUnionVariantTemplate::Inline with { name, fields } %}
  | { {{ tag_field }}: "{{ name }}";{% for field in fields %} {{ field.code_name }}{% if field.is_optional %}?{% endif %}: {{ field.type_str }};{% endfor %} }
{%- when TsUnionVariantTemplate::Newtype with { name, type_str } %}
  | ({ {{ tag_field }}: "{{ name }}" } & {{ type_str }})
{%- endmatch %}
{%- endfor %};

```

**Step 4: Create type alias template**

```jinja2
{# codegen/templates/ts/type_alias.ts.j2 #}
export type {{ name }} = {{ target_type }};

```

**Step 5: Create index template**

```jinja2
{# codegen/templates/ts/index.ts.j2 #}
{%- for module in modules %}
export * from './{{ module.file_name }}';
{%- endfor %}

```

**Step 6: Create template structs**

```rust
// codegen/src/code_gen/ts/templates.rs
use askama::Template;

/// Field information for TypeScript templates
#[derive(Clone)]
pub struct TsFieldTemplate {
    pub code_name: String,
    pub type_str: String,
    pub is_optional: bool,
}

/// Template for rendering a TypeScript interface
#[derive(Template)]
#[template(path = "ts/interface.ts.j2")]
pub struct InterfaceTemplate {
    pub name: String,
    pub fields: Vec<TsFieldTemplate>,
    pub use_readonly: bool,
}

/// Template for rendering a TypeScript enum
#[derive(Template)]
#[template(path = "ts/enum.ts.j2")]
pub struct TsEnumTemplate {
    pub name: String,
    pub variants: Vec<String>,
}

/// Union variant types for template
#[derive(Clone)]
pub enum TsUnionVariantTemplate {
    Unit(String),
    Inline {
        name: String,
        fields: Vec<TsFieldTemplate>,
    },
    Newtype {
        name: String,
        type_str: String,
    },
}

/// Template for rendering a TypeScript discriminated union
#[derive(Template)]
#[template(path = "ts/union.ts.j2")]
pub struct TsUnionTemplate {
    pub name: String,
    pub tag_field: String,
    pub variants: Vec<TsUnionVariantTemplate>,
}

/// Template for rendering a TypeScript type alias
#[derive(Template)]
#[template(path = "ts/type_alias.ts.j2")]
pub struct TsTypeAliasTemplate {
    pub name: String,
    pub target_type: String,
}

/// Template for rendering an index file
#[derive(Template)]
#[template(path = "ts/index.ts.j2")]
pub struct TsIndexTemplate {
    pub modules: Vec<TsModuleEntry>,
}

#[derive(Clone)]
pub struct TsModuleEntry {
    pub file_name: String,
}
```

**Step 7: Update mod.rs**

```rust
// codegen/src/code_gen/ts/mod.rs
mod options;
pub mod templates;
pub use options::*;
```

**Step 8: Run build to verify templates compile**

Run: `cargo build --package fluorite_codegen`
Expected: PASS (Askama validates templates at compile time)

**Step 9: Commit**

```bash
git add codegen/templates/ts/ codegen/src/code_gen/ts/templates.rs codegen/src/code_gen/ts/mod.rs
git commit -m "feat(ts): add Askama templates for TypeScript generation"
```

---

## Task 3: Implement TsTemplateGenerator

**Files:**
- Create: `codegen/src/code_gen/ts/template_generator.rs`
- Modify: `codegen/src/code_gen/ts/mod.rs`

**Step 1: Write the failing test**

```rust
// Add to codegen/tests/ts_code_gen.rs
use std::sync::Arc;
use fluorite_codegen::code_gen::fs::MemoryFileSystem;
use fluorite_codegen::code_gen::ts::{TypeScriptOptions, TsTemplateGenerator};

fn deserialize_definition_file(file_path: &str) -> anyhow::Result<fluorite_codegen::definitions::Definition> {
    let file_content = std::fs::read_to_string(file_path)?;
    let r = serde_yaml::from_str(&file_content)?;
    Ok(r)
}

#[test]
fn test_ts_generates_interface() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let content = fs.get_string("/output/protocols/users/user.ts").unwrap();
    assert!(content.contains("export interface User"));
    assert!(content.contains("firstName: string"));
    assert!(content.contains("age: number"));

    Ok(())
}

#[test]
fn test_ts_generates_enum() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let content = fs.get_string("/output/protocols/users/gender.ts").unwrap();
    assert!(content.contains("export enum Gender"));
    assert!(content.contains("Male = \"Male\""));
    assert!(content.contains("Female = \"Female\""));

    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package fluorite_codegen test_ts_generates`
Expected: FAIL with "unresolved import TsTemplateGenerator"

**Step 3: Write TsTemplateGenerator implementation**

```rust
// codegen/src/code_gen/ts/template_generator.rs
//! Template-based TypeScript code generator using askama templates

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{anyhow, Result};
use askama::Template;

use crate::code_gen::fs::FileSystem;
use crate::code_gen::ir::{
    IRBuilder, IRField, IRFieldType, IRPrimitive, IRSchema, IRStruct, IRType, IRTypeAlias,
    IRTypeAliasTarget, IRUnion, IRUnionStyle, IRUnionVariant,
};
use crate::code_gen::utils::to_camel_case;
use crate::code_gen::validation::{ValidationError, Validator};
use crate::definitions::Definition;

use super::templates::{
    InterfaceTemplate, TsEnumTemplate, TsFieldTemplate, TsIndexTemplate, TsModuleEntry,
    TsTypeAliasTemplate, TsUnionTemplate, TsUnionVariantTemplate,
};
use super::TypeScriptOptions;

/// Template-based TypeScript code generator
pub struct TsTemplateGenerator {
    options: TypeScriptOptions,
    fs: Arc<dyn FileSystem>,
}

impl TsTemplateGenerator {
    pub fn new(options: TypeScriptOptions, fs: Arc<dyn FileSystem>) -> Self {
        Self { options, fs }
    }

    /// Generate TypeScript code from definitions
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
                                other @ IRUnionVariant::Unit(_)
                                | other @ IRUnionVariant::Newtype(..) => {
                                    resolved_variants.push(other.clone())
                                }
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
            // Generate all types in index.ts
            let index_path = format!("{}/index.ts", output_path);
            let mut content = String::new();

            for ir_type in types.iter().filter(|t| !t.is_internal()) {
                content.push_str(&self.render_type(ir_type, schema)?);
            }

            self.fs.write_file(&index_path, content.as_bytes())?;
        } else {
            // Generate each type in separate file + index.ts
            let mut modules = Vec::new();

            for ir_type in types.iter().filter(|t| !t.is_internal()) {
                let file_name = to_camel_case(ir_type.name());
                let file_path = format!("{}/{}.ts", output_path, file_name);
                let content = self.render_type(ir_type, schema)?;

                self.fs.write_file(&file_path, content.as_bytes())?;
                modules.push(TsModuleEntry { file_name });
            }

            // Generate index.ts
            let index_template = TsIndexTemplate { modules };
            let index_content = index_template.render()?;
            let index_path = format!("{}/index.ts", output_path);
            self.fs.write_file(&index_path, index_content.as_bytes())?;
        }

        Ok(())
    }

    fn render_type(&self, ir_type: &IRType, schema: &IRSchema) -> Result<String> {
        match ir_type {
            IRType::Struct(s) => self.render_interface(s, schema),
            IRType::Enum(e) => self.render_enum(e),
            IRType::Union(u) => self.render_union(u, schema),
            IRType::TypeAlias(a) => self.render_type_alias(a, schema),
        }
    }

    fn render_interface(&self, s: &IRStruct, schema: &IRSchema) -> Result<String> {
        let fields: Vec<TsFieldTemplate> = s
            .fields
            .iter()
            .map(|f| self.convert_field(f, schema))
            .collect::<Result<Vec<_>>>()?;

        let template = InterfaceTemplate {
            name: s.name.clone(),
            fields,
            use_readonly: self.options.use_readonly,
        };

        Ok(template.render()?)
    }

    fn render_enum(&self, e: &crate::code_gen::ir::IREnum) -> Result<String> {
        let template = TsEnumTemplate {
            name: e.name.clone(),
            variants: e.variants.clone(),
        };

        Ok(template.render()?)
    }

    fn render_union(&self, u: &IRUnion, schema: &IRSchema) -> Result<String> {
        let variants: Vec<TsUnionVariantTemplate> = u
            .variants
            .iter()
            .map(|v| self.convert_union_variant(v, schema))
            .collect::<Result<Vec<_>>>()?;

        let template = TsUnionTemplate {
            name: u.name.clone(),
            tag_field: u.tag_field.clone(),
            variants,
        };

        Ok(template.render()?)
    }

    fn render_type_alias(&self, a: &IRTypeAlias, schema: &IRSchema) -> Result<String> {
        let target_type = match &a.target {
            IRTypeAliasTarget::List(item_type) => {
                let item_str = self.format_type(item_type, schema)?;
                format!("{}[]", item_str)
            }
            IRTypeAliasTarget::Map(key_type, value_type) => {
                let key_str = self.format_type(key_type, schema)?;
                let value_str = self.format_type(value_type, schema)?;
                format!("Record<{}, {}>", key_str, value_str)
            }
        };

        let template = TsTypeAliasTemplate {
            name: a.name.clone(),
            target_type,
        };

        Ok(template.render()?)
    }

    fn convert_field(&self, field: &IRField, schema: &IRSchema) -> Result<TsFieldTemplate> {
        let type_str = self.format_type(&field.field_type, schema)?;

        // Use camelCase for TypeScript field names
        let code_name = if let Some(rename) = &field.rename {
            to_camel_case(rename)
        } else {
            to_camel_case(&field.name)
        };

        Ok(TsFieldTemplate {
            code_name,
            type_str,
            is_optional: field.is_optional,
        })
    }

    fn convert_union_variant(
        &self,
        variant: &IRUnionVariant,
        schema: &IRSchema,
    ) -> Result<TsUnionVariantTemplate> {
        match variant {
            IRUnionVariant::Unit(name) => Ok(TsUnionVariantTemplate::Unit(name.clone())),
            IRUnionVariant::Inline(name, fields) => {
                let field_templates: Vec<TsFieldTemplate> = fields
                    .iter()
                    .map(|f| self.convert_field(f, schema))
                    .collect::<Result<Vec<_>>>()?;

                Ok(TsUnionVariantTemplate::Inline {
                    name: name.clone(),
                    fields: field_templates,
                })
            }
            IRUnionVariant::Newtype(name, type_ref) => {
                let type_str = type_ref.clone();
                Ok(TsUnionVariantTemplate::Newtype {
                    name: name.clone(),
                    type_str,
                })
            }
        }
    }

    fn format_type(&self, field_type: &IRFieldType, schema: &IRSchema) -> Result<String> {
        match field_type {
            IRFieldType::Primitive(p) => Ok(self.format_primitive(*p)),
            IRFieldType::Custom(name) => Ok(name.clone()),
            IRFieldType::Any => Ok(self.options.any_type.clone()),
            IRFieldType::List(item) => {
                let item_str = self.format_type(item, schema)?;
                Ok(format!("{}[]", item_str))
            }
            IRFieldType::Map(key, value) => {
                let key_str = self.format_type(key, schema)?;
                let value_str = self.format_type(value, schema)?;
                Ok(format!("Record<{}, {}>", key_str, value_str))
            }
        }
    }

    fn format_primitive(&self, p: IRPrimitive) -> String {
        match p {
            IRPrimitive::String => "string".to_string(),
            IRPrimitive::Bool => "boolean".to_string(),
            IRPrimitive::DateTime => "string".to_string(),
            IRPrimitive::UInt32 | IRPrimitive::UInt64 |
            IRPrimitive::Int32 | IRPrimitive::Int64 |
            IRPrimitive::Float32 | IRPrimitive::Float64 => "number".to_string(),
        }
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

**Step 4: Update mod.rs to export TsTemplateGenerator**

```rust
// codegen/src/code_gen/ts/mod.rs
mod options;
pub mod templates;
mod template_generator;

pub use options::*;
pub use template_generator::*;
```

**Step 5: Run tests to verify they pass**

Run: `cargo test --package fluorite_codegen test_ts_generates`
Expected: PASS

**Step 6: Commit**

```bash
git add codegen/src/code_gen/ts/
git commit -m "feat(ts): implement TsTemplateGenerator for TypeScript code generation"
```

---

## Task 4: Add CLI Support for TypeScript

**Files:**
- Modify: `codegen/src/main.rs`

**Step 1: Add Ts command variant**

```rust
// Add to Commands enum in main.rs
#[derive(Subcommand)]
enum Commands {
    /// Generate Rust code from YAML definitions
    Rust { /* existing */ },

    /// Generate TypeScript code from YAML definitions
    Ts {
        /// Input YAML files
        #[arg(short, long, required = true, num_args = 1..)]
        inputs: Vec<String>,

        /// Output directory
        #[arg(short, long)]
        output: String,

        /// Generate all types in a single index.ts file per package
        #[arg(long, default_value = "false")]
        single_file: bool,

        /// Custom type to use for 'Any' fields (default: unknown)
        #[arg(long, default_value = "unknown")]
        any_type: String,

        /// Generate readonly properties
        #[arg(long, default_value = "false")]
        readonly: bool,
    },
}
```

**Step 2: Add handler for Ts command**

```rust
// Add to main() match
Commands::Ts {
    inputs,
    output,
    single_file,
    any_type,
    readonly,
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
    let options = code_gen::ts::TypeScriptOptions::new(output)
        .with_single_file(single_file)
        .with_any_type(&any_type)
        .with_readonly(readonly);

    // Generate
    let fs = Arc::new(RealFileSystem::new());
    let generator = code_gen::ts::TsTemplateGenerator::new(options, fs);
    generator.generate(&definitions)?;

    println!("TypeScript code generation complete!");
}
```

**Step 3: Test CLI manually**

Run: `cargo run --package fluorite_codegen --bin fluorite -- ts --inputs ../examples/users.yml --output /tmp/ts_test`
Expected: "TypeScript code generation complete!"

Run: `ls /tmp/ts_test/protocols/users/`
Expected: `user.ts  gender.ts  index.ts`

**Step 4: Commit**

```bash
git add codegen/src/main.rs
git commit -m "feat(ts): add 'ts' subcommand to CLI"
```

---

## Task 5: Add Integration Tests for TypeScript

**Files:**
- Create: `codegen/tests/ts_code_gen.rs`

**Step 1: Write comprehensive test file**

```rust
// codegen/tests/ts_code_gen.rs
use std::fs;
use std::sync::Arc;

use fluorite_codegen::{
    code_gen::{
        fs::MemoryFileSystem,
        ts::{TypeScriptOptions, TsTemplateGenerator},
    },
    definitions::Definition,
};

fn deserialize_definition_file(file_path: &str) -> anyhow::Result<Definition> {
    let file_content = fs::read_to_string(file_path)?;
    let r = serde_yaml::from_str(&file_content)?;
    Ok(r)
}

#[test]
fn test_typescript_options_default() {
    let options = TypeScriptOptions::new("/output".to_string());

    assert_eq!(options.output_dir, "/output");
    assert!(!options.single_file);
    assert_eq!(options.any_type, "unknown");
    assert!(!options.use_readonly);
}

#[test]
fn test_typescript_options_builder() {
    let options = TypeScriptOptions::new("/output".to_string())
        .with_single_file(true)
        .with_any_type("any")
        .with_readonly(true);

    assert!(options.single_file);
    assert_eq!(options.any_type, "any");
    assert!(options.use_readonly);
}

#[test]
fn test_ts_generates_interface() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let content = fs.get_string("/output/protocols/users/user.ts").unwrap();
    assert!(content.contains("export interface User"), "Should have User interface");
    assert!(content.contains("firstName: string"), "Should have firstName field");
    assert!(content.contains("lastName: string"), "Should have lastName field");
    assert!(content.contains("age: number"), "Should have age as number");
    assert!(content.contains("active: boolean"), "Should have active as boolean");

    Ok(())
}

#[test]
fn test_ts_generates_enum() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let content = fs.get_string("/output/protocols/users/gender.ts").unwrap();
    assert!(content.contains("export enum Gender"), "Should have Gender enum");
    assert!(content.contains("Male = \"Male\""), "Should have Male variant");
    assert!(content.contains("Female = \"Female\""), "Should have Female variant");

    Ok(())
}

#[test]
fn test_ts_generates_discriminated_union() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    let content = fs.get_string("/output/protocols/orders/address.ts").unwrap();
    assert!(content.contains("export type Address"), "Should have Address type");
    assert!(content.contains("type: \""), "Should have discriminant field");

    Ok(())
}

#[test]
fn test_ts_generates_type_alias() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    let order_list_content = fs.get_string("/output/protocols/orders/orderList.ts").unwrap();
    assert!(order_list_content.contains("export type OrderList = Order[]"),
        "Should have list alias. Got: {}", order_list_content);

    let order_map_content = fs.get_string("/output/protocols/orders/orderMap.ts").unwrap();
    assert!(order_map_content.contains("export type OrderMap = Record<string, Order>"),
        "Should have map alias. Got: {}", order_map_content);

    Ok(())
}

#[test]
fn test_ts_single_file_mode() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned()).with_single_file(true);
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    // Should only have index.ts
    let files = fs.files();
    assert!(files.contains_key("/output/protocols/users/index.ts"), "Should have index.ts");
    assert!(!files.contains_key("/output/protocols/users/user.ts"), "Should NOT have user.ts");

    let content = fs.get_string("/output/protocols/users/index.ts").unwrap();
    assert!(content.contains("export interface User"), "Should have User in index.ts");
    assert!(content.contains("export enum Gender"), "Should have Gender in index.ts");

    Ok(())
}

#[test]
fn test_ts_readonly_option() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned()).with_readonly(true);
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let content = fs.get_string("/output/protocols/users/user.ts").unwrap();
    assert!(content.contains("readonly firstName: string"),
        "Should have readonly fields. Got: {}", content);

    Ok(())
}

#[test]
fn test_ts_any_type_option() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned()).with_any_type("any");
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    // PostCode has an 'instruction' field of type Any
    let content = fs.get_string("/output/protocols/orders/postCode.ts").unwrap();
    assert!(content.contains(": any"), "Should use custom any type. Got: {}", content);
    assert!(!content.contains(": unknown"), "Should NOT use unknown");

    Ok(())
}

#[test]
fn test_ts_optional_fields() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;
    let d2 = deserialize_definition_file("../examples/orders.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1, d2])?;

    let content = fs.get_string("/output/protocols/orders/order.ts").unwrap();
    // shipping field is optional
    assert!(content.contains("shipping?:"),
        "Should have optional shipping field. Got: {}", content);

    Ok(())
}

#[test]
fn test_ts_index_file_exports() -> anyhow::Result<()> {
    let d1 = deserialize_definition_file("../examples/users.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());
    let generator = TsTemplateGenerator::new(options, fs.clone());
    generator.generate(&[d1])?;

    let content = fs.get_string("/output/protocols/users/index.ts").unwrap();
    assert!(content.contains("export * from './user'"), "Should export user");
    assert!(content.contains("export * from './gender'"), "Should export gender");

    Ok(())
}

#[test]
fn test_ts_empty_definition_list() {
    let fs = Arc::new(MemoryFileSystem::new());
    let options = TypeScriptOptions::new("/output".to_owned());

    let generator = TsTemplateGenerator::new(options, fs.clone());
    let result = generator.generate(&[]);

    assert!(result.is_ok());
    assert!(fs.files().is_empty());
}
```

**Step 2: Run all TypeScript tests**

Run: `cargo test --package fluorite_codegen ts_`
Expected: All tests PASS

**Step 3: Commit**

```bash
git add codegen/tests/ts_code_gen.rs
git commit -m "test(ts): add comprehensive integration tests for TypeScript generation"
```

---

## Task 6: Create E2E TypeScript Project

**Files:**
- Create: `codegen/tests/ts_e2e/package.json`
- Create: `codegen/tests/ts_e2e/tsconfig.json`
- Create: `codegen/tests/ts_e2e/schemas/test.yaml`
- Create: `codegen/tests/ts_e2e/src/test.ts`

**Step 1: Create package.json**

```json
{
  "name": "fluorite-ts-e2e-test",
  "version": "1.0.0",
  "private": true,
  "scripts": {
    "generate": "cd ../../../.. && cargo run --package fluorite_codegen --bin fluorite -- ts --inputs codegen/tests/ts_e2e/schemas/test.yaml --output codegen/tests/ts_e2e/generated",
    "typecheck": "tsc --noEmit",
    "test": "npm run generate && npm run typecheck"
  },
  "devDependencies": {
    "typescript": "^5.0.0"
  }
}
```

**Step 2: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "node",
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  },
  "include": ["src/**/*", "generated/**/*"]
}
```

**Step 3: Create test schema**

```yaml
# codegen/tests/ts_e2e/schemas/test.yaml
configs:
  rust_package: "test.types"
types:
  - name: User
    type: Object
    fields:
      - name: id
        type: UInt64
      - name: name
        type: String
      - name: email
        type: String
        optional: true
      - name: status
        type: UserStatus
      - name: metadata
        type: Any

  - name: UserStatus
    type: Enum
    values:
      - Active
      - Inactive
      - Pending

  - name: Event
    type: Union
    type_tag: eventType
    values:
      - UserCreated
      - UserUpdated
      - UserDeleted

  - name: UserCreated
    type: Object
    fields:
      - name: user
        type: User
      - name: timestamp
        type: DateTime

  - name: UserUpdated
    type: Object
    fields:
      - name: user
        type: User
      - name: changes
        type: String

  - name: UserDeleted
    type: Object
    fields:
      - name: userId
        type: UInt64

  - name: UserList
    type: List
    item_type: User

  - name: UserMap
    type: Map
    key_type: String
    value_type: User
```

**Step 4: Create test usage file**

```typescript
// codegen/tests/ts_e2e/src/test.ts
// This file uses the generated types to verify they work correctly

import {
  User,
  UserStatus,
  Event,
  UserList,
  UserMap
} from '../generated/test/types';

// Test interface usage
const user: User = {
  id: 1,
  name: "John Doe",
  email: "john@example.com",
  status: UserStatus.Active,
  metadata: { role: "admin" }
};

// Test optional field
const userWithoutEmail: User = {
  id: 2,
  name: "Jane Doe",
  status: UserStatus.Pending,
  metadata: null
};

// Test enum
function getUserStatusLabel(status: UserStatus): string {
  switch (status) {
    case UserStatus.Active:
      return "Active User";
    case UserStatus.Inactive:
      return "Inactive User";
    case UserStatus.Pending:
      return "Pending Approval";
  }
}

// Test discriminated union with type narrowing
function handleEvent(event: Event): string {
  switch (event.eventType) {
    case "UserCreated":
      return `User ${event.user.name} created at ${event.timestamp}`;
    case "UserUpdated":
      return `User ${event.user.name} updated: ${event.changes}`;
    case "UserDeleted":
      return `User ${event.userId} deleted`;
  }
}

// Test type aliases
const users: UserList = [user, userWithoutEmail];
const userById: UserMap = {
  "1": user,
  "2": userWithoutEmail
};

// Verify everything compiles
console.log("User:", user);
console.log("Status label:", getUserStatusLabel(user.status));
console.log("Users count:", users.length);
console.log("User map keys:", Object.keys(userById));
```

**Step 5: Run E2E test**

```bash
cd codegen/tests/ts_e2e
npm install
npm test
```
Expected: Exit code 0 (no TypeScript errors)

**Step 6: Commit**

```bash
git add codegen/tests/ts_e2e/
git commit -m "test(ts): add E2E TypeScript project for type verification"
```

---

## Task 7: Create npm Package Structure

**Files:**
- Create: `npm/fluorite-cli/package.json`
- Create: `npm/fluorite-cli/install.js`
- Create: `npm/fluorite-cli/bin/fluorite.js`
- Create: `npm/fluorite-cli/README.md`

**Step 1: Create package.json**

```json
{
  "name": "@zhxiaogg/fluorite-cli",
  "version": "0.1.0",
  "description": "Code generator from YAML schema definitions - generates Rust and TypeScript",
  "bin": {
    "fluorite": "./bin/fluorite.js"
  },
  "scripts": {
    "postinstall": "node install.js"
  },
  "keywords": [
    "codegen",
    "typescript",
    "rust",
    "schema",
    "yaml"
  ],
  "author": "shawn",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/blossomstack/fluorite.git"
  },
  "os": ["darwin", "linux", "win32"],
  "cpu": ["x64", "arm64"],
  "engines": {
    "node": ">=16.0.0"
  }
}
```

**Step 2: Create install.js**

```javascript
#!/usr/bin/env node
// install.js - Downloads the appropriate binary for the current platform

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const VERSION = '0.1.0';
const REPO = 'zhxiaogg/fluorite';

function getPlatformInfo() {
  const platform = process.platform;
  const arch = process.arch;

  const platformMap = {
    'darwin': 'apple-darwin',
    'linux': 'unknown-linux-gnu',
    'win32': 'pc-windows-msvc'
  };

  const archMap = {
    'x64': 'x86_64',
    'arm64': 'aarch64'
  };

  const targetPlatform = platformMap[platform];
  const targetArch = archMap[arch];

  if (!targetPlatform || !targetArch) {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }

  return {
    target: `${targetArch}-${targetPlatform}`,
    extension: platform === 'win32' ? '.exe' : ''
  };
}

async function downloadBinary() {
  const { target, extension } = getPlatformInfo();
  const binaryName = `fluorite${extension}`;
  const assetName = `fluorite-${target}${extension}`;
  const downloadUrl = `https://github.com/${REPO}/releases/download/v${VERSION}/${assetName}`;

  const binDir = path.join(__dirname, 'bin');
  const binaryPath = path.join(binDir, binaryName);

  // Skip if binary already exists
  if (fs.existsSync(binaryPath)) {
    console.log('Binary already installed.');
    return;
  }

  console.log(`Downloading fluorite binary for ${target}...`);
  console.log(`URL: ${downloadUrl}`);

  // Create bin directory if it doesn't exist
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }

  // For now, just create a placeholder that tells users to build from source
  // TODO: Implement actual binary download when releases are available
  const placeholderScript = `#!/bin/sh
echo "Error: Pre-built binaries not yet available."
echo "Please build from source: cargo build --release --package fluorite_codegen"
exit 1
`;

  fs.writeFileSync(binaryPath, placeholderScript);
  fs.chmodSync(binaryPath, '755');

  console.log('Note: Pre-built binaries not yet available. See README for build instructions.');
}

downloadBinary().catch(err => {
  console.error('Failed to install:', err.message);
  process.exit(1);
});
```

**Step 3: Create bin/fluorite.js**

```javascript
#!/usr/bin/env node
// bin/fluorite.js - Wrapper that invokes the Rust binary

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

const binDir = path.join(__dirname);
const ext = process.platform === 'win32' ? '.exe' : '';
const binaryPath = path.join(binDir, `fluorite${ext}`);

if (!fs.existsSync(binaryPath)) {
  console.error('Error: fluorite binary not found.');
  console.error('Please run: npm rebuild @zhxiaogg/fluorite-cli');
  process.exit(1);
}

const child = spawn(binaryPath, process.argv.slice(2), {
  stdio: 'inherit'
});

child.on('error', (err) => {
  console.error('Failed to execute fluorite:', err.message);
  process.exit(1);
});

child.on('exit', (code) => {
  process.exit(code ?? 0);
});
```

**Step 4: Create README.md**

```markdown
# @zhxiaogg/fluorite-cli

Code generator from YAML schema definitions - generates Rust and TypeScript code.

## Installation

```bash
npm install -D @zhxiaogg/fluorite-cli
```

## Usage

### Generate TypeScript

```bash
npx fluorite ts --inputs ./schemas/*.yaml --output ./src/generated
```

### Generate Rust

```bash
npx fluorite rust --inputs ./schemas/*.yaml --output ./src/generated
```

## Options

### TypeScript (`ts`)

| Option | Default | Description |
|--------|---------|-------------|
| `--inputs` | required | Input YAML files |
| `--output` | required | Output directory |
| `--single-file` | false | Generate all types in a single file |
| `--any-type` | unknown | Type to use for Any fields |
| `--readonly` | false | Generate readonly properties |

### Rust (`rust`)

| Option | Default | Description |
|--------|---------|-------------|
| `--inputs` | required | Input YAML files |
| `--output` | required | Output directory |
| `--single-file` | true | Generate all types in a single file |
| `--any-type` | fluorite::Any | Type to use for Any fields |
| `--derives` | | Custom derives (comma-separated) |
| `--extra-derives` | | Additional derives |
| `--generate-new` | true | Generate derive_new |
| `--visibility` | public | Type visibility |

## Example package.json

```json
{
  "scripts": {
    "generate": "fluorite ts --inputs ./schemas/*.yaml --output ./src/generated",
    "build": "npm run generate && tsc"
  },
  "devDependencies": {
    "@zhxiaogg/fluorite-cli": "^0.1.0"
  }
}
```

## Building from Source

If pre-built binaries are not available for your platform:

```bash
git clone https://github.com/blossomstack/fluorite.git
cd fluorite
cargo build --release --package fluorite_codegen
```

The binary will be at `target/release/fluorite`.
```

**Step 5: Commit**

```bash
git add npm/
git commit -m "feat(npm): add @zhxiaogg/fluorite-cli npm package structure"
```

---

## Task 8: Update Documentation

**Files:**
- Modify: `CLAUDE.md`
- Modify: `README.md` (if exists at project root)

**Step 1: CLAUDE.md is already updated from design phase**

Verify the TypeScript section exists in CLAUDE.md.

**Step 2: Create/Update project README.md**

Check if root README.md exists. If not, create minimal one or update existing to include TypeScript generation info.

**Step 3: Commit**

```bash
git add CLAUDE.md README.md
git commit -m "docs: update documentation for TypeScript generation"
```

---

## Task 9: Run Full Test Suite

**Step 1: Run all Rust tests**

Run: `cargo test --package fluorite_codegen`
Expected: All tests PASS

**Step 2: Run E2E TypeScript tests**

```bash
cd codegen/tests/ts_e2e
npm install
npm test
```
Expected: Exit code 0

**Step 3: Verify CLI works**

```bash
cargo run --package fluorite_codegen --bin fluorite -- ts --help
cargo run --package fluorite_codegen --bin fluorite -- ts \
  --inputs examples/users.yml examples/orders.yml \
  --output /tmp/ts_verify
cat /tmp/ts_verify/protocols/users/user.ts
```
Expected: Valid TypeScript interface output

**Step 4: Final commit**

```bash
git add -A
git commit -m "feat: complete TypeScript code generation implementation"
```

---

## Summary of Deliverables

After completing all tasks:

1. **TypeScript Options** - `codegen/src/code_gen/ts/options.rs`
2. **Askama Templates** - `codegen/templates/ts/*.j2`
3. **Template Structs** - `codegen/src/code_gen/ts/templates.rs`
4. **TsTemplateGenerator** - `codegen/src/code_gen/ts/template_generator.rs`
5. **CLI Command** - `ts` subcommand in `codegen/src/main.rs`
6. **Integration Tests** - `codegen/tests/ts_code_gen.rs`
7. **E2E Tests** - `codegen/tests/ts_e2e/`
8. **npm Package** - `npm/fluorite-cli/`
9. **Documentation** - Updated CLAUDE.md

## Acceptance Criteria Verification

| Criterion | Verified By |
|-----------|-------------|
| Generated TypeScript compiles with `tsc --strict` | E2E test in Task 6 |
| Interfaces correctly represent Object types | Integration test: `test_ts_generates_interface` |
| Enums have string values | Integration test: `test_ts_generates_enum` |
| Discriminated unions work | Integration test: `test_ts_generates_discriminated_union` |
| List/Map type aliases resolve | Integration test: `test_ts_generates_type_alias` |
| Cross-package type references work | E2E test uses types across packages |
| CLI works | Task 4 manual verification |
| npm package structure ready | Task 7 |
