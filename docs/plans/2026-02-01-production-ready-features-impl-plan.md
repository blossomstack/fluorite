# Production-Ready Features Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add new primitive types, serde features, and documentation support to make Fluorite production-ready for API contracts and event/message schema code generation.

**Architecture:** The implementation follows the existing template-based code generation flow: YAML → Definitions → IR → Templates → Rust code. New primitives are added to `IRPrimitive`, new configs to `FieldConfig/TypeConfig`, and template rendering updated accordingly.

**Tech Stack:** Rust, askama templates, serde, chrono, uuid, rust_decimal, url crates

---

## Phase 1: New Primitive Types

### Task 1.1: Add New Primitive Type Variants to IRPrimitive

**Files:**
- Modify: `codegen/src/code_gen/ir/types.rs:96-107`
- Test: `codegen/tests/rust_code_gen.rs`

**Step 1: Write the failing test**

Add to `codegen/tests/rust_code_gen.rs` in the `ir_builder_tests` module:

```rust
#[test]
fn test_new_primitive_types() {
    let def = create_definition(
        "test.package",
        vec![CustomType::Object {
            name: "NewPrimitives".to_string(),
            fields: vec![
                Field {
                    name: "uuid_field".to_string(),
                    field_type: "UUID".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "decimal_field".to_string(),
                    field_type: "Decimal".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "bytes_field".to_string(),
                    field_type: "Bytes".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "url_field".to_string(),
                    field_type: "Url".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "timestamp_field".to_string(),
                    field_type: "Timestamp".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "timestamp_millis_field".to_string(),
                    field_type: "TimestampMillis".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "datetime_utc_field".to_string(),
                    field_type: "DateTimeUtc".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "datetime_tz_field".to_string(),
                    field_type: "DateTimeTz".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "date_field".to_string(),
                    field_type: "Date".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "time_field".to_string(),
                    field_type: "Time".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "duration_field".to_string(),
                    field_type: "Duration".to_string(),
                    optional: None,
                    configs: None,
                },
            ],
        }],
    );

    let schema = IRBuilder::new().build(&[def]).unwrap();
    let pkg = schema.packages.get("test.package").unwrap();

    if let IRType::Struct(s) = &pkg.types[0] {
        assert_eq!(s.fields.len(), 11);
        assert!(matches!(s.fields[0].field_type, IRFieldType::Primitive(IRPrimitive::UUID)));
        assert!(matches!(s.fields[1].field_type, IRFieldType::Primitive(IRPrimitive::Decimal)));
        assert!(matches!(s.fields[2].field_type, IRFieldType::Primitive(IRPrimitive::Bytes)));
        assert!(matches!(s.fields[3].field_type, IRFieldType::Primitive(IRPrimitive::Url)));
        assert!(matches!(s.fields[4].field_type, IRFieldType::Primitive(IRPrimitive::Timestamp)));
        assert!(matches!(s.fields[5].field_type, IRFieldType::Primitive(IRPrimitive::TimestampMillis)));
        assert!(matches!(s.fields[6].field_type, IRFieldType::Primitive(IRPrimitive::DateTimeUtc)));
        assert!(matches!(s.fields[7].field_type, IRFieldType::Primitive(IRPrimitive::DateTimeTz)));
        assert!(matches!(s.fields[8].field_type, IRFieldType::Primitive(IRPrimitive::Date)));
        assert!(matches!(s.fields[9].field_type, IRFieldType::Primitive(IRPrimitive::Time)));
        assert!(matches!(s.fields[10].field_type, IRFieldType::Primitive(IRPrimitive::Duration)));
    } else {
        panic!("Expected struct type");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package fluorite_codegen test_new_primitive_types`
Expected: FAIL with "no variant named `UUID` found"

**Step 3: Add new variants to IRPrimitive**

In `codegen/src/code_gen/ir/types.rs`, update the `IRPrimitive` enum:

```rust
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
    // New time/date types
    Timestamp,
    TimestampMillis,
    DateTimeUtc,
    DateTimeTz,
    Date,
    Time,
    Duration,
    // Other new primitives
    UUID,
    Decimal,
    Bytes,
    Url,
}
```

**Step 4: Update IRBuilder to parse new primitives**

In `codegen/src/code_gen/ir/builder.rs`, update `parse_primitive`:

```rust
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
        // New time/date types
        "Timestamp" => Some(IRPrimitive::Timestamp),
        "TimestampMillis" => Some(IRPrimitive::TimestampMillis),
        "DateTimeUtc" => Some(IRPrimitive::DateTimeUtc),
        "DateTimeTz" => Some(IRPrimitive::DateTimeTz),
        "Date" => Some(IRPrimitive::Date),
        "Time" => Some(IRPrimitive::Time),
        "Duration" => Some(IRPrimitive::Duration),
        // Other new primitives
        "UUID" => Some(IRPrimitive::UUID),
        "Decimal" => Some(IRPrimitive::Decimal),
        "Bytes" => Some(IRPrimitive::Bytes),
        "Url" => Some(IRPrimitive::Url),
        _ => None,
    }
}
```

**Step 5: Run test to verify it passes**

Run: `cargo test --package fluorite_codegen test_new_primitive_types`
Expected: PASS

**Step 6: Commit**

```bash
git add codegen/src/code_gen/ir/types.rs codegen/src/code_gen/ir/builder.rs codegen/tests/rust_code_gen.rs
git commit -m "feat: add new primitive type variants to IR

Add UUID, Decimal, Bytes, Url, Timestamp, TimestampMillis,
DateTimeUtc, DateTimeTz, Date, Time, Duration primitives"
```

---

### Task 1.2: Add Rust Type Mappings for New Primitives

**Files:**
- Modify: `codegen/src/code_gen/rust/template_generator.rs:288-300`
- Test: `codegen/tests/rust_code_gen.rs`

**Step 1: Write the failing test**

Add to `codegen/tests/rust_code_gen.rs` in the `template_generator_tests` module:

```rust
#[test]
fn test_new_primitive_type_mappings() -> anyhow::Result<()> {
    use fluorite_codegen::definitions::CustomType;

    let def = Definition {
        configs: DefinitionConfig {
            rust_package: Some("test.primitives".to_string()),
        },
        types: vec![CustomType::Object {
            name: "NewPrimitives".to_string(),
            fields: vec![
                Field {
                    name: "uuid_field".to_string(),
                    field_type: "UUID".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "decimal_field".to_string(),
                    field_type: "Decimal".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "bytes_field".to_string(),
                    field_type: "Bytes".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "url_field".to_string(),
                    field_type: "Url".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "timestamp_field".to_string(),
                    field_type: "Timestamp".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "datetime_utc_field".to_string(),
                    field_type: "DateTimeUtc".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "date_field".to_string(),
                    field_type: "Date".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "time_field".to_string(),
                    field_type: "Time".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "duration_field".to_string(),
                    field_type: "Duration".to_string(),
                    optional: None,
                    configs: None,
                },
            ],
        }],
    };

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[def])?;

    let content = fs.get_string("/output/test/primitives/mod.rs").unwrap();

    assert!(content.contains("uuid::Uuid"), "Should have uuid::Uuid");
    assert!(content.contains("rust_decimal::Decimal"), "Should have rust_decimal::Decimal");
    assert!(content.contains("Vec<u8>"), "Should have Vec<u8> for Bytes");
    assert!(content.contains("url::Url"), "Should have url::Url");
    assert!(content.contains("i64"), "Should have i64 for Timestamp");
    assert!(content.contains("chrono::DateTime<chrono::Utc>"), "Should have DateTime<Utc>");
    assert!(content.contains("chrono::NaiveDate"), "Should have NaiveDate");
    assert!(content.contains("chrono::NaiveTime"), "Should have NaiveTime");
    assert!(content.contains("chrono::Duration"), "Should have chrono::Duration");

    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package fluorite_codegen test_new_primitive_type_mappings`
Expected: FAIL - missing match arms for new primitives

**Step 3: Update format_primitive in template_generator.rs**

In `codegen/src/code_gen/rust/template_generator.rs`, update `format_primitive`:

```rust
fn format_primitive(&self, p: IRPrimitive) -> String {
    match p {
        IRPrimitive::String => "String".to_string(),
        IRPrimitive::Bool => "bool".to_string(),
        IRPrimitive::DateTime => "chrono::NaiveDateTime".to_string(),
        IRPrimitive::UInt32 => "u32".to_string(),
        IRPrimitive::UInt64 => "u64".to_string(),
        IRPrimitive::Int32 => "i32".to_string(),
        IRPrimitive::Int64 => "i64".to_string(),
        IRPrimitive::Float32 => "f32".to_string(),
        IRPrimitive::Float64 => "f64".to_string(),
        // New time/date types
        IRPrimitive::Timestamp => "i64".to_string(),
        IRPrimitive::TimestampMillis => "i64".to_string(),
        IRPrimitive::DateTimeUtc => "chrono::DateTime<chrono::Utc>".to_string(),
        IRPrimitive::DateTimeTz => "chrono::DateTime<chrono::FixedOffset>".to_string(),
        IRPrimitive::Date => "chrono::NaiveDate".to_string(),
        IRPrimitive::Time => "chrono::NaiveTime".to_string(),
        IRPrimitive::Duration => "chrono::Duration".to_string(),
        // Other new primitives
        IRPrimitive::UUID => "uuid::Uuid".to_string(),
        IRPrimitive::Decimal => "rust_decimal::Decimal".to_string(),
        IRPrimitive::Bytes => "Vec<u8>".to_string(),
        IRPrimitive::Url => "url::Url".to_string(),
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test --package fluorite_codegen test_new_primitive_type_mappings`
Expected: PASS

**Step 5: Commit**

```bash
git add codegen/src/code_gen/rust/template_generator.rs codegen/tests/rust_code_gen.rs
git commit -m "feat: add Rust type mappings for new primitives

Map UUID→uuid::Uuid, Decimal→rust_decimal::Decimal,
Bytes→Vec<u8>, Url→url::Url, and chrono types"
```

---

### Task 1.3: Update Validator to Recognize New Primitives

**Files:**
- Modify: `codegen/src/code_gen/validation/mod.rs`
- Test: `codegen/tests/rust_code_gen.rs`

**Step 1: Write the failing test**

Add to `codegen/tests/rust_code_gen.rs` in the `validator_tests` module:

```rust
#[test]
fn test_new_primitives_recognized_by_validator() {
    let schema = create_schema(vec![(
        "test".to_string(),
        vec![IRType::Struct(IRStruct {
            name: "TestNewPrimitives".to_string(),
            fields: vec![
                IRField {
                    name: "uuid".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::UUID),
                    is_optional: false,
                    is_boxed: false,
                    rename: None,
                    doc: None,
                },
                IRField {
                    name: "decimal".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Decimal),
                    is_optional: false,
                    is_boxed: false,
                    rename: None,
                    doc: None,
                },
                IRField {
                    name: "bytes".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Bytes),
                    is_optional: false,
                    is_boxed: false,
                    rename: None,
                    doc: None,
                },
                IRField {
                    name: "url".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Url),
                    is_optional: false,
                    is_boxed: false,
                    rename: None,
                    doc: None,
                },
                IRField {
                    name: "timestamp".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Timestamp),
                    is_optional: false,
                    is_boxed: false,
                    rename: None,
                    doc: None,
                },
                IRField {
                    name: "date".to_string(),
                    field_type: IRFieldType::Primitive(IRPrimitive::Date),
                    is_optional: false,
                    is_boxed: false,
                    rename: None,
                    doc: None,
                },
            ],
            is_union_variant: false,
            doc: None,
        })],
    )]);

    let errors = Validator::new().validate(&schema);
    assert!(errors.is_empty(), "New primitives should be valid: {:?}", errors);
}
```

**Step 2: Run test to verify it passes (validator doesn't check primitive types directly)**

Run: `cargo test --package fluorite_codegen test_new_primitives_recognized_by_validator`
Expected: PASS (primitives in IRFieldType::Primitive are always valid)

**Step 3: Commit (test only, no code changes needed)**

```bash
git add codegen/tests/rust_code_gen.rs
git commit -m "test: verify validator accepts new primitive types"
```

---

## Phase 2: Portable Serde Features

### Task 2.1: Add Field-Level `alias` Config

**Files:**
- Modify: `codegen/src/definitions/mod.rs:12-15`
- Modify: `codegen/src/code_gen/ir/types.rs:57-66`
- Modify: `codegen/src/code_gen/ir/builder.rs:191-207`
- Modify: `codegen/src/code_gen/rust/templates.rs:14-22`
- Modify: `codegen/templates/rust/struct.rs.j2`
- Test: `codegen/tests/rust_code_gen.rs`

**Step 1: Write the failing test**

Add to `codegen/tests/rust_code_gen.rs`:

```rust
#[test]
fn test_field_alias() -> anyhow::Result<()> {
    let def = Definition {
        configs: DefinitionConfig {
            rust_package: Some("test.alias".to_string()),
        },
        types: vec![CustomType::Object {
            name: "AliasTest".to_string(),
            fields: vec![Field {
                name: "source".to_string(),
                field_type: "String".to_string(),
                optional: None,
                configs: Some(FieldConfig {
                    rename: None,
                    rust_type_wrapper: None,
                    alias: Some(vec!["origin".to_string(), "src".to_string()]),
                    default: None,
                }),
            }],
        }],
    };

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[def])?;

    let content = fs.get_string("/output/test/alias/mod.rs").unwrap();
    assert!(content.contains(r#"#[serde(alias = "origin")]"#), "Should have alias origin");
    assert!(content.contains(r#"#[serde(alias = "src")]"#), "Should have alias src");

    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package fluorite_codegen test_field_alias`
Expected: FAIL with "no field `alias`"

**Step 3: Add `alias` to FieldConfig**

In `codegen/src/definitions/mod.rs`:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldConfig {
    pub rename: Option<String>,
    pub rust_type_wrapper: Option<crate::definitions::RustTypeWrapper>,
    #[serde(default)]
    pub alias: Option<Vec<String>>,
    pub default: Option<String>,
}
```

**Step 4: Add `aliases` to IRField**

In `codegen/src/code_gen/ir/types.rs`:

```rust
/// A field within a struct
#[derive(Debug, Clone)]
pub struct IRField {
    pub name: String,
    pub field_type: IRFieldType,
    pub is_optional: bool,
    pub is_boxed: bool,
    pub rename: Option<String>,
    pub aliases: Vec<String>,
    pub default_value: Option<String>,
    pub doc: Option<String>,
}
```

**Step 5: Update IRBuilder to set aliases**

In `codegen/src/code_gen/ir/builder.rs`, update `convert_field`:

```rust
fn convert_field(&self, field: &Field) -> IRField {
    let field_type = self.convert_field_type(&field.field_type);
    let is_boxed = field
        .configs
        .as_ref()
        .and_then(|c| c.rust_type_wrapper.as_ref())
        .is_some();
    let rename = field.configs.as_ref().and_then(|c| c.rename.clone());
    let aliases = field
        .configs
        .as_ref()
        .and_then(|c| c.alias.clone())
        .unwrap_or_default();
    let default_value = field.configs.as_ref().and_then(|c| c.default.clone());

    IRField {
        name: field.name.clone(),
        field_type,
        is_optional: field.optional.unwrap_or(false),
        is_boxed,
        rename,
        aliases,
        default_value,
        doc: None,
    }
}
```

**Step 6: Update FieldTemplate**

In `codegen/src/code_gen/rust/templates.rs`:

```rust
/// Field information for templates
#[derive(Clone)]
pub struct FieldTemplate {
    pub code_name: String,
    pub original_name: String,
    pub type_str: String,
    pub is_optional: bool,
    pub needs_rename: bool,
    pub aliases: Vec<String>,
    pub default_value: Option<String>,
}
```

**Step 7: Update template_generator to pass aliases**

In `codegen/src/code_gen/rust/template_generator.rs`, update `convert_field`:

```rust
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
        aliases: field.aliases.clone(),
        default_value: field.default_value.clone(),
    })
}
```

**Step 8: Update struct.rs.j2 template**

```jinja2
{{ derives }}
pub struct {{ name }} {
{%- for field in fields %}
{%- if field.needs_rename %}
    #[serde(rename = "{{ field.original_name }}")]
{%- endif %}
{%- for alias in field.aliases %}
    #[serde(alias = "{{ alias }}")]
{%- endfor %}
{%- if field.default_value.is_some() %}
    #[serde(default = "{{ field.default_value.as_ref().unwrap() }}")]
{%- endif %}
{%- if field.is_optional %}
    #[serde(skip_serializing_if = "Option::is_none")]
{%- endif %}
    pub {{ field.code_name }}: {{ field.type_str|safe }},
{%- endfor %}
}

```

**Step 9: Fix all test IRField instantiations to include new fields**

Update all existing IRField constructions in tests to include `aliases: vec![]` and `default_value: None`.

**Step 10: Run test to verify it passes**

Run: `cargo test --package fluorite_codegen test_field_alias`
Expected: PASS

**Step 11: Run all tests**

Run: `cargo test --package fluorite_codegen`
Expected: PASS

**Step 12: Commit**

```bash
git add codegen/src/definitions/mod.rs codegen/src/code_gen/ir/types.rs codegen/src/code_gen/ir/builder.rs codegen/src/code_gen/rust/templates.rs codegen/src/code_gen/rust/template_generator.rs codegen/templates/rust/struct.rs.j2 codegen/tests/rust_code_gen.rs
git commit -m "feat: add field-level alias and default serde attributes

- Add alias field to FieldConfig for alternative JSON names
- Add default field to FieldConfig for default values
- Generate #[serde(alias = ...)] attributes in Rust output"
```

---

### Task 2.2: Add Type-Level `rename_all` Config

**Files:**
- Modify: `codegen/src/definitions/mod.rs`
- Modify: `codegen/src/code_gen/ir/types.rs`
- Modify: `codegen/src/code_gen/ir/builder.rs`
- Modify: `codegen/src/code_gen/rust/templates.rs`
- Modify: `codegen/templates/rust/struct.rs.j2`
- Test: `codegen/tests/rust_code_gen.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_type_rename_all() -> anyhow::Result<()> {
    let def = Definition {
        configs: DefinitionConfig {
            rust_package: Some("test.renameall".to_string()),
        },
        types: vec![CustomType::Object {
            name: "RenameAllTest".to_string(),
            fields: vec![
                Field {
                    name: "first_name".to_string(),
                    field_type: "String".to_string(),
                    optional: None,
                    configs: None,
                },
                Field {
                    name: "last_name".to_string(),
                    field_type: "String".to_string(),
                    optional: None,
                    configs: None,
                },
            ],
            configs: Some(TypeConfig {
                union_style: None,
                rename_all: Some("camelCase".to_string()),
                deny_unknown_fields: None,
            }),
        }],
    };

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[def])?;

    let content = fs.get_string("/output/test/renameall/mod.rs").unwrap();
    assert!(content.contains(r#"#[serde(rename_all = "camelCase")]"#), "Should have rename_all");

    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package fluorite_codegen test_type_rename_all`
Expected: FAIL

**Step 3: Update TypeConfig and CustomType**

In `codegen/src/definitions/mod.rs`:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TypeConfig {
    pub union_style: Option<crate::definitions::UnionStyle>,
    pub rename_all: Option<String>,
    #[serde(rename = "rust")]
    pub rust_config: Option<RustTypeConfig>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RustTypeConfig {
    pub deny_unknown_fields: Option<bool>,
}

// Update CustomType::Object to include configs
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum CustomType {
    Object {
        name: String,
        fields: crate::definitions::FieldList,
        #[serde(default)]
        configs: Option<TypeConfig>,
    },
    // ... other variants unchanged
}
```

**Step 4: Update IRStruct to include rename_all**

In `codegen/src/code_gen/ir/types.rs`:

```rust
/// A struct type
#[derive(Debug, Clone)]
pub struct IRStruct {
    pub name: String,
    pub fields: Vec<IRField>,
    pub is_union_variant: bool,
    pub rename_all: Option<String>,
    pub deny_unknown_fields: bool,
    pub doc: Option<String>,
}
```

**Step 5: Update IRBuilder convert_type for Object**

In `codegen/src/code_gen/ir/builder.rs`:

```rust
CustomType::Object { name, fields, configs } => {
    let is_union_variant = self.union_variant_names.contains(name);
    let ir_fields = fields.iter().map(|f| self.convert_field(f)).collect();
    let rename_all = configs.as_ref().and_then(|c| c.rename_all.clone());
    let deny_unknown_fields = configs
        .as_ref()
        .and_then(|c| c.rust_config.as_ref())
        .and_then(|r| r.deny_unknown_fields)
        .unwrap_or(false);

    Ok(IRType::Struct(IRStruct {
        name: name.clone(),
        fields: ir_fields,
        is_union_variant,
        rename_all,
        deny_unknown_fields,
        doc: None,
    }))
}
```

**Step 6: Update StructTemplate**

In `codegen/src/code_gen/rust/templates.rs`:

```rust
/// Template for rendering a struct
#[derive(Template)]
#[template(path = "rust/struct.rs.j2")]
pub struct StructTemplate {
    pub derives: String,
    pub name: String,
    pub fields: Vec<FieldTemplate>,
    pub rename_all: Option<String>,
    pub deny_unknown_fields: bool,
}
```

**Step 7: Update render_struct in template_generator.rs**

```rust
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
        rename_all: s.rename_all.clone(),
        deny_unknown_fields: s.deny_unknown_fields,
    };

    Ok(template.render()?)
}
```

**Step 8: Update struct.rs.j2 template**

```jinja2
{{ derives }}
{%- if rename_all.is_some() or deny_unknown_fields %}
#[serde(
{%- if rename_all.is_some() %}rename_all = "{{ rename_all.as_ref().unwrap() }}"{% if deny_unknown_fields %}, {% endif %}{% endif %}
{%- if deny_unknown_fields %}deny_unknown_fields{% endif %}
)]
{%- endif %}
pub struct {{ name }} {
{%- for field in fields %}
{%- if field.needs_rename %}
    #[serde(rename = "{{ field.original_name }}")]
{%- endif %}
{%- for alias in field.aliases %}
    #[serde(alias = "{{ alias }}")]
{%- endfor %}
{%- if field.default_value.is_some() %}
    #[serde(default = "{{ field.default_value.as_ref().unwrap() }}")]
{%- endif %}
{%- if field.is_optional %}
    #[serde(skip_serializing_if = "Option::is_none")]
{%- endif %}
    pub {{ field.code_name }}: {{ field.type_str|safe }},
{%- endfor %}
}

```

**Step 9: Fix all existing IRStruct instantiations in tests**

Add `rename_all: None, deny_unknown_fields: false` to all existing IRStruct constructions in tests.

**Step 10: Run test to verify it passes**

Run: `cargo test --package fluorite_codegen test_type_rename_all`
Expected: PASS

**Step 11: Run all tests**

Run: `cargo test --package fluorite_codegen`
Expected: PASS

**Step 12: Commit**

```bash
git add codegen/src/definitions/mod.rs codegen/src/code_gen/ir/types.rs codegen/src/code_gen/ir/builder.rs codegen/src/code_gen/rust/templates.rs codegen/src/code_gen/rust/template_generator.rs codegen/templates/rust/struct.rs.j2 codegen/tests/rust_code_gen.rs
git commit -m "feat: add type-level rename_all and deny_unknown_fields

- Add rename_all to TypeConfig for camelCase/snake_case etc
- Add RustTypeConfig with deny_unknown_fields
- Generate #[serde(rename_all = ...)] attributes"
```

---

## Phase 3: Rust-Specific Serde Features

### Task 3.1: Add Field-Level `skip_if_none`, `skip_if_default`, `flatten`

**Files:**
- Modify: `codegen/src/definitions/mod.rs`
- Modify: `codegen/src/code_gen/ir/types.rs`
- Modify: `codegen/src/code_gen/ir/builder.rs`
- Modify: `codegen/src/code_gen/rust/templates.rs`
- Modify: `codegen/templates/rust/struct.rs.j2`
- Test: `codegen/tests/rust_code_gen.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_rust_field_serde_features() -> anyhow::Result<()> {
    let def = Definition {
        configs: DefinitionConfig {
            rust_package: Some("test.rustfeatures".to_string()),
        },
        types: vec![CustomType::Object {
            name: "RustFeaturesTest".to_string(),
            fields: vec![
                Field {
                    name: "skip_none_field".to_string(),
                    field_type: "String".to_string(),
                    optional: true,
                    configs: Some(FieldConfig {
                        rename: None,
                        rust_type_wrapper: None,
                        alias: None,
                        default: None,
                        rust: Some(RustFieldConfig {
                            skip_if_none: Some(true),
                            skip_if_default: None,
                            flatten: None,
                        }),
                    }),
                },
                Field {
                    name: "skip_default_field".to_string(),
                    field_type: "String".to_string(),
                    optional: None,
                    configs: Some(FieldConfig {
                        rename: None,
                        rust_type_wrapper: None,
                        alias: None,
                        default: Some("default_value".to_string()),
                        rust: Some(RustFieldConfig {
                            skip_if_none: None,
                            skip_if_default: Some(true),
                            flatten: None,
                        }),
                    }),
                },
                Field {
                    name: "flattened_field".to_string(),
                    field_type: "Metadata".to_string(),
                    optional: None,
                    configs: Some(FieldConfig {
                        rename: None,
                        rust_type_wrapper: None,
                        alias: None,
                        default: None,
                        rust: Some(RustFieldConfig {
                            skip_if_none: None,
                            skip_if_default: None,
                            flatten: Some(true),
                        }),
                    }),
                },
            ],
            configs: None,
        }],
    };

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[def])?;

    let content = fs.get_string("/output/test/rustfeatures/mod.rs").unwrap();
    assert!(content.contains(r#"#[serde(skip_serializing_if = "Option::is_none")]"#), "Should have skip_if_none");
    assert!(content.contains(r#"#[serde(skip_serializing_if = "is_default")]"#), "Should have skip_if_default");
    assert!(content.contains("#[serde(flatten)]"), "Should have flatten");

    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package fluorite_codegen test_rust_field_serde_features`
Expected: FAIL

**Step 3: Add RustFieldConfig to definitions**

In `codegen/src/definitions/mod.rs`:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RustFieldConfig {
    pub skip_if_none: Option<bool>,
    pub skip_if_default: Option<bool>,
    pub flatten: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldConfig {
    pub rename: Option<String>,
    pub rust_type_wrapper: Option<crate::definitions::RustTypeWrapper>,
    #[serde(default)]
    pub alias: Option<Vec<String>>,
    pub default: Option<String>,
    #[serde(rename = "rust")]
    pub rust: Option<RustFieldConfig>,
}
```

**Step 4: Add Rust serde fields to IRField**

In `codegen/src/code_gen/ir/types.rs`:

```rust
/// A field within a struct
#[derive(Debug, Clone)]
pub struct IRField {
    pub name: String,
    pub field_type: IRFieldType,
    pub is_optional: bool,
    pub is_boxed: bool,
    pub rename: Option<String>,
    pub aliases: Vec<String>,
    pub default_value: Option<String>,
    pub skip_if_none: bool,
    pub skip_if_default: bool,
    pub flatten: bool,
    pub doc: Option<String>,
}
```

**Step 5: Update IRBuilder to set new fields**

In `codegen/src/code_gen/ir/builder.rs`, update `convert_field`:

```rust
fn convert_field(&self, field: &Field) -> IRField {
    let field_type = self.convert_field_type(&field.field_type);
    let is_boxed = field
        .configs
        .as_ref()
        .and_then(|c| c.rust_type_wrapper.as_ref())
        .is_some();
    let rename = field.configs.as_ref().and_then(|c| c.rename.clone());
    let aliases = field
        .configs
        .as_ref()
        .and_then(|c| c.alias.clone())
        .unwrap_or_default();
    let default_value = field.configs.as_ref().and_then(|c| c.default.clone());

    let rust_config = field.configs.as_ref().and_then(|c| c.rust.as_ref());
    let skip_if_none = rust_config.and_then(|r| r.skip_if_none).unwrap_or(false);
    let skip_if_default = rust_config.and_then(|r| r.skip_if_default).unwrap_or(false);
    let flatten = rust_config.and_then(|r| r.flatten).unwrap_or(false);

    IRField {
        name: field.name.clone(),
        field_type,
        is_optional: field.optional.unwrap_or(false),
        is_boxed,
        rename,
        aliases,
        default_value,
        skip_if_none,
        skip_if_default,
        flatten,
        doc: None,
    }
}
```

**Step 6: Update FieldTemplate**

In `codegen/src/code_gen/rust/templates.rs`:

```rust
/// Field information for templates
#[derive(Clone)]
pub struct FieldTemplate {
    pub code_name: String,
    pub original_name: String,
    pub type_str: String,
    pub is_optional: bool,
    pub needs_rename: bool,
    pub aliases: Vec<String>,
    pub default_value: Option<String>,
    pub skip_if_none: bool,
    pub skip_if_default: bool,
    pub flatten: bool,
}
```

**Step 7: Update convert_field in template_generator.rs**

```rust
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
        aliases: field.aliases.clone(),
        default_value: field.default_value.clone(),
        skip_if_none: field.skip_if_none || field.is_optional, // auto skip_if_none for optional
        skip_if_default: field.skip_if_default,
        flatten: field.flatten,
    })
}
```

**Step 8: Update struct.rs.j2 template**

```jinja2
{{ derives }}
{%- if rename_all.is_some() or deny_unknown_fields %}
#[serde(
{%- if rename_all.is_some() %}rename_all = "{{ rename_all.as_ref().unwrap() }}"{% if deny_unknown_fields %}, {% endif %}{% endif %}
{%- if deny_unknown_fields %}deny_unknown_fields{% endif %}
)]
{%- endif %}
pub struct {{ name }} {
{%- for field in fields %}
{%- if field.needs_rename %}
    #[serde(rename = "{{ field.original_name }}")]
{%- endif %}
{%- for alias in field.aliases %}
    #[serde(alias = "{{ alias }}")]
{%- endfor %}
{%- if field.default_value.is_some() %}
    #[serde(default = "{{ field.default_value.as_ref().unwrap() }}")]
{%- endif %}
{%- if field.flatten %}
    #[serde(flatten)]
{%- endif %}
{%- if field.skip_if_none %}
    #[serde(skip_serializing_if = "Option::is_none")]
{%- elif field.skip_if_default %}
    #[serde(skip_serializing_if = "is_default")]
{%- endif %}
    pub {{ field.code_name }}: {{ field.type_str|safe }},
{%- endfor %}
}

```

**Step 9: Fix all existing IRField instantiations in tests**

Add `skip_if_none: false, skip_if_default: false, flatten: false` to all existing IRField constructions.

**Step 10: Run test to verify it passes**

Run: `cargo test --package fluorite_codegen test_rust_field_serde_features`
Expected: PASS

**Step 11: Run all tests**

Run: `cargo test --package fluorite_codegen`
Expected: PASS

**Step 12: Commit**

```bash
git add codegen/src/definitions/mod.rs codegen/src/code_gen/ir/types.rs codegen/src/code_gen/ir/builder.rs codegen/src/code_gen/rust/templates.rs codegen/src/code_gen/rust/template_generator.rs codegen/templates/rust/struct.rs.j2 codegen/tests/rust_code_gen.rs
git commit -m "feat: add Rust-specific field serde features

- Add skip_if_none for optional field serialization
- Add skip_if_default for default value skipping
- Add flatten for nested struct flattening"
```

---

## Phase 4: Documentation Support

### Task 4.1: Add `description` Field Support

**Files:**
- Modify: `codegen/src/definitions/mod.rs`
- Modify: `codegen/src/code_gen/ir/builder.rs`
- Modify: `codegen/src/code_gen/rust/templates.rs`
- Modify: `codegen/templates/rust/struct.rs.j2`
- Modify: `codegen/templates/rust/enum.rs.j2`
- Test: `codegen/tests/rust_code_gen.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn test_description_generates_doc_comments() -> anyhow::Result<()> {
    let def = Definition {
        configs: DefinitionConfig {
            rust_package: Some("test.docs".to_string()),
        },
        types: vec![CustomType::Object {
            name: "DocumentedType".to_string(),
            description: Some("This is a documented type".to_string()),
            fields: vec![Field {
                name: "documented_field".to_string(),
                field_type: "String".to_string(),
                optional: None,
                description: Some("This field has documentation".to_string()),
                configs: None,
            }],
            configs: None,
        }],
    };

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[def])?;

    let content = fs.get_string("/output/test/docs/mod.rs").unwrap();
    assert!(content.contains("/// This is a documented type"), "Should have type doc comment");
    assert!(content.contains("/// This field has documentation"), "Should have field doc comment");

    Ok(())
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test --package fluorite_codegen test_description_generates_doc_comments`
Expected: FAIL

**Step 3: Add description to definitions**

In `codegen/src/definitions/mod.rs`:

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub optional: Option<bool>,
    pub description: Option<String>,
    pub deprecated: Option<bool>,
    pub configs: Option<crate::definitions::FieldConfig>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum CustomType {
    Object {
        name: String,
        description: Option<String>,
        fields: crate::definitions::FieldList,
        #[serde(default)]
        configs: Option<TypeConfig>,
    },
    Enum {
        name: String,
        description: Option<String>,
        values: crate::definitions::EnumValueList,
    },
    Union {
        name: String,
        description: Option<String>,
        type_tag: String,
        values: crate::definitions::EnumValueList,
        configs: Option<crate::definitions::TypeConfig>,
    },
    List {
        name: String,
        description: Option<String>,
        item_type: String,
    },
    Map {
        name: String,
        description: Option<String>,
        key_type: String,
        value_type: String,
    },
}
```

**Step 4: Update IRBuilder to populate doc fields**

In `codegen/src/code_gen/ir/builder.rs`, update `convert_type` and `convert_field`:

```rust
// In convert_type for Object:
CustomType::Object { name, description, fields, configs } => {
    // ...
    Ok(IRType::Struct(IRStruct {
        name: name.clone(),
        fields: ir_fields,
        is_union_variant,
        rename_all,
        deny_unknown_fields,
        doc: description.clone(),
    }))
}

// In convert_field:
fn convert_field(&self, field: &Field) -> IRField {
    // ... existing code ...
    IRField {
        name: field.name.clone(),
        field_type,
        is_optional: field.optional.unwrap_or(false),
        is_boxed,
        rename,
        aliases,
        default_value,
        skip_if_none,
        skip_if_default,
        flatten,
        doc: field.description.clone(),
        deprecated: field.deprecated.unwrap_or(false),
    }
}
```

**Step 5: Add doc and deprecated to IRField**

In `codegen/src/code_gen/ir/types.rs`:

```rust
/// A field within a struct
#[derive(Debug, Clone)]
pub struct IRField {
    pub name: String,
    pub field_type: IRFieldType,
    pub is_optional: bool,
    pub is_boxed: bool,
    pub rename: Option<String>,
    pub aliases: Vec<String>,
    pub default_value: Option<String>,
    pub skip_if_none: bool,
    pub skip_if_default: bool,
    pub flatten: bool,
    pub doc: Option<String>,
    pub deprecated: bool,
}
```

**Step 6: Update FieldTemplate and StructTemplate**

In `codegen/src/code_gen/rust/templates.rs`:

```rust
/// Field information for templates
#[derive(Clone)]
pub struct FieldTemplate {
    pub code_name: String,
    pub original_name: String,
    pub type_str: String,
    pub is_optional: bool,
    pub needs_rename: bool,
    pub aliases: Vec<String>,
    pub default_value: Option<String>,
    pub skip_if_none: bool,
    pub skip_if_default: bool,
    pub flatten: bool,
    pub doc: Option<String>,
    pub deprecated: bool,
}

#[derive(Template)]
#[template(path = "rust/struct.rs.j2")]
pub struct StructTemplate {
    pub derives: String,
    pub name: String,
    pub fields: Vec<FieldTemplate>,
    pub rename_all: Option<String>,
    pub deny_unknown_fields: bool,
    pub doc: Option<String>,
}
```

**Step 7: Update convert_field to pass doc**

In `codegen/src/code_gen/rust/template_generator.rs`:

```rust
fn convert_field(&self, field: &IRField, schema: &IRSchema) -> Result<FieldTemplate> {
    // ... existing code ...
    Ok(FieldTemplate {
        // ... existing fields ...
        doc: field.doc.clone(),
        deprecated: field.deprecated,
    })
}
```

**Step 8: Update struct.rs.j2 template**

```jinja2
{%- if doc.is_some() %}
/// {{ doc.as_ref().unwrap() }}
{%- endif %}
{{ derives }}
{%- if rename_all.is_some() or deny_unknown_fields %}
#[serde(
{%- if rename_all.is_some() %}rename_all = "{{ rename_all.as_ref().unwrap() }}"{% if deny_unknown_fields %}, {% endif %}{% endif %}
{%- if deny_unknown_fields %}deny_unknown_fields{% endif %}
)]
{%- endif %}
pub struct {{ name }} {
{%- for field in fields %}
{%- if field.doc.is_some() %}
    /// {{ field.doc.as_ref().unwrap() }}
{%- endif %}
{%- if field.deprecated %}
    #[deprecated{%- if field.doc.is_some() %}(note = "{{ field.doc.as_ref().unwrap() }}"){%- endif %}]
{%- endif %}
{%- if field.needs_rename %}
    #[serde(rename = "{{ field.original_name }}")]
{%- endif %}
{%- for alias in field.aliases %}
    #[serde(alias = "{{ alias }}")]
{%- endfor %}
{%- if field.default_value.is_some() %}
    #[serde(default = "{{ field.default_value.as_ref().unwrap() }}")]
{%- endif %}
{%- if field.flatten %}
    #[serde(flatten)]
{%- endif %}
{%- if field.skip_if_none %}
    #[serde(skip_serializing_if = "Option::is_none")]
{%- elif field.skip_if_default %}
    #[serde(skip_serializing_if = "is_default")]
{%- endif %}
    pub {{ field.code_name }}: {{ field.type_str|safe }},
{%- endfor %}
}

```

**Step 9: Update enum.rs.j2 template similarly for enum docs**

```jinja2
{%- if doc.is_some() %}
/// {{ doc.as_ref().unwrap() }}
{%- endif %}
{{ derives }}
pub enum {{ name }} {
{%- for variant in variants %}
    {{ variant }},
{%- endfor %}
}

```

**Step 10: Fix all existing test instantiations**

Add `deprecated: false` to all existing IRField constructions, and `doc: None` where not already present.

**Step 11: Run test to verify it passes**

Run: `cargo test --package fluorite_codegen test_description_generates_doc_comments`
Expected: PASS

**Step 12: Run all tests**

Run: `cargo test --package fluorite_codegen`
Expected: PASS

**Step 13: Commit**

```bash
git add codegen/src/definitions/mod.rs codegen/src/code_gen/ir/types.rs codegen/src/code_gen/ir/builder.rs codegen/src/code_gen/rust/templates.rs codegen/src/code_gen/rust/template_generator.rs codegen/templates/rust/struct.rs.j2 codegen/templates/rust/enum.rs.j2 codegen/tests/rust_code_gen.rs
git commit -m "feat: add documentation and deprecation support

- Add description field to types and fields in YAML schema
- Generate /// doc comments in Rust output
- Add deprecated field with #[deprecated] attribute generation"
```

---

## Phase 5: End-to-End Testing

### Task 5.1: Create Comprehensive E2E Test Schema

**Files:**
- Create: `examples/production_features.yml`
- Test: `codegen/tests/rust_code_gen.rs`

**Step 1: Create test schema file**

Create `examples/production_features.yml`:

```yaml
---
configs:
  rust_package: "test.production"

types:
  - name: CreateOrderRequest
    type: Object
    description: Request payload for creating a new order
    configs:
      rename_all: camelCase
      rust:
        deny_unknown_fields: true
    fields:
      - name: order_id
        type: UUID
        description: Client-generated order ID for idempotency
      - name: amount
        type: Decimal
        description: Order total in customer's currency
      - name: currency
        type: String
        default: "default_currency"
      - name: created_at
        type: DateTimeUtc
      - name: ttl
        type: Duration
        optional: true
        configs:
          rust:
            skip_if_none: true
      - name: metadata
        type: OrderMetadata
        optional: true
        configs:
          rust:
            flatten: true

  - name: OrderMetadata
    type: Object
    fields:
      - name: source
        type: String
        configs:
          alias:
            - origin
            - src
      - name: correlation_id
        type: UUID
        optional: true

  - name: OrderEvent
    type: Object
    description: Event emitted when order state changes
    fields:
      - name: event_id
        type: UUID
      - name: timestamp
        type: Timestamp
        description: Unix epoch seconds when event occurred
      - name: payload
        type: Bytes
        optional: true
        description: Optional binary payload

  - name: LegacyOrder
    type: Object
    fields:
      - name: id
        type: String
      - name: old_field
        type: String
        deprecated: true
        description: Use new_field instead
```

**Step 2: Write comprehensive E2E test**

Add to `codegen/tests/rust_code_gen.rs`:

```rust
#[test]
fn test_production_features_e2e() -> anyhow::Result<()> {
    let def = deserialize_definition_file("../examples/production_features.yml")?;

    let fs = Arc::new(MemoryFileSystem::new());
    let options = RustOptions::new("/output".to_owned());
    let generator = RustTemplateGenerator::new(options, fs.clone());
    generator.generate(&[def])?;

    let content = fs.get_string("/output/test/production/mod.rs").unwrap();

    // Test new primitives
    assert!(content.contains("uuid::Uuid"), "Should have UUID type");
    assert!(content.contains("rust_decimal::Decimal"), "Should have Decimal type");
    assert!(content.contains("chrono::DateTime<chrono::Utc>"), "Should have DateTimeUtc");
    assert!(content.contains("chrono::Duration"), "Should have Duration");
    assert!(content.contains("Vec<u8>"), "Should have Bytes as Vec<u8>");
    assert!(content.contains("i64"), "Should have Timestamp as i64");

    // Test documentation
    assert!(content.contains("/// Request payload for creating a new order"), "Should have type doc");
    assert!(content.contains("/// Client-generated order ID for idempotency"), "Should have field doc");
    assert!(content.contains("/// Event emitted when order state changes"), "Should have event doc");

    // Test type-level serde features
    assert!(content.contains(r#"rename_all = "camelCase""#), "Should have rename_all");
    assert!(content.contains("deny_unknown_fields"), "Should have deny_unknown_fields");

    // Test field-level serde features
    assert!(content.contains(r#"alias = "origin""#), "Should have alias origin");
    assert!(content.contains(r#"alias = "src""#), "Should have alias src");
    assert!(content.contains("#[serde(flatten)]"), "Should have flatten");

    // Test deprecation
    assert!(content.contains("#[deprecated"), "Should have deprecated attribute");
    assert!(content.contains("Use new_field instead"), "Should have deprecation note");

    Ok(())
}
```

**Step 3: Run test to verify it passes**

Run: `cargo test --package fluorite_codegen test_production_features_e2e`
Expected: PASS

**Step 4: Commit**

```bash
git add examples/production_features.yml codegen/tests/rust_code_gen.rs
git commit -m "test: add comprehensive E2E test for production features

Tests new primitives, documentation, serde features, and deprecation"
```

---

### Task 5.2: Update Documentation

**Files:**
- Modify: `CLAUDE.md`
- Modify: `README.md` (if exists, or note in CLAUDE.md)

**Step 1: Update CLAUDE.md with new features**

Add a new section documenting the production-ready features.

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with production-ready features"
```

---

### Task 5.3: Final Integration Test - Build Full Project

**Step 1: Run full test suite**

Run: `cargo test`
Expected: All tests PASS

**Step 2: Run clippy**

Run: `cargo clippy --all-targets`
Expected: No warnings

**Step 3: Build release**

Run: `cargo build --release`
Expected: Build succeeds

**Step 4: Final commit if any fixes needed**

---

## Summary

This plan implements the production-ready features in 5 phases:

1. **Phase 1:** New primitive types (UUID, Decimal, Bytes, Url, Timestamp, DateTimeUtc, etc.)
2. **Phase 2:** Portable serde features (alias, default, rename_all)
3. **Phase 3:** Rust-specific serde features (skip_if_none, skip_if_default, flatten, deny_unknown_fields)
4. **Phase 4:** Documentation support (description, deprecated)
5. **Phase 5:** E2E testing and documentation

Each task follows TDD with explicit steps for writing failing tests, implementing, verifying, and committing.
