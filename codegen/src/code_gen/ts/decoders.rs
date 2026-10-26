//! Opt-in runtime wire validation. Types remain owned by the ordinary generator.
use crate::code_gen::ir::*;
use crate::code_gen::utils::to_camel_case;
use anyhow::Result;
use std::collections::BTreeSet;

fn quoted(s: &str) -> String {
    serde_json::Value::String(s.to_owned()).to_string()
}

// Encode every UTF-8 byte so distinct package paths cannot share an identifier.
// Replacing separators alone would conflate e.g. a.b and a_b.
fn package_alias(package: &str) -> String {
    let mut alias = String::from("p_");
    for byte in package.bytes() {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        alias.push(char::from(HEX[usize::from(byte >> 4)]));
        alias.push(char::from(HEX[usize::from(byte & 15)]));
    }
    alias
}

struct Context<'a> {
    package: &'a str,
    imports: BTreeSet<String>,
}
impl Context<'_> {
    fn check(&mut self, ty: &IRFieldType, value: &str, path: &str) -> String {
        match ty {
            IRFieldType::Any => String::new(),
            IRFieldType::Primitive(p) => {
                let predicate = match p {
                    IRPrimitive::Bool => format!("typeof {value} === 'boolean'"),
                    IRPrimitive::UInt32 => format!("typeof {value} === 'number' && Number.isInteger({value}) && {value} >= 0 && {value} <= 4294967295"),
                    IRPrimitive::Int32 => format!("typeof {value} === 'number' && Number.isInteger({value}) && {value} >= -2147483648 && {value} <= 2147483647"),
                    IRPrimitive::UInt64 => format!("typeof {value} === 'number' && Number.isSafeInteger({value}) && {value} >= 0"),
                    IRPrimitive::Int64 | IRPrimitive::Timestamp | IRPrimitive::TimestampMillis => format!("typeof {value} === 'number' && Number.isSafeInteger({value})"),
                    IRPrimitive::Float32 | IRPrimitive::Float64 => format!("typeof {value} === 'number' && Number.isFinite({value})"),
                    IRPrimitive::String | IRPrimitive::DateTime | IRPrimitive::UUID | IRPrimitive::Decimal | IRPrimitive::Bytes | IRPrimitive::Url | IRPrimitive::DateTimeUtc | IRPrimitive::DateTimeTz | IRPrimitive::Date | IRPrimitive::Time | IRPrimitive::Duration => format!("typeof {value} === 'string'"),
                };
                format!(
                    "Wire.expect({predicate}, {path}, {});",
                    quoted(&format!("{:?}", p))
                )
            }
            IRFieldType::Custom(r) => {
                let prefix = if r.package == self.package {
                    String::new()
                } else {
                    self.imports.insert(r.package.clone());
                    format!("{}.", package_alias(&r.package))
                };
                format!("{prefix}decode{}({value}, {path});", r.name)
            }
            IRFieldType::List(item) => {
                let inner = self.check(item, "item", "itemPath");
                if inner.is_empty() {
                    format!("Wire.array({value}, {path});")
                } else {
                    format!("Wire.visitArray({value}, {path}, (item, itemPath) => {{ {inner} }});")
                }
            }
            IRFieldType::Map(key, item) => {
                let numeric_key = matches!(
                    key.as_ref(),
                    IRFieldType::Primitive(
                        IRPrimitive::UInt32
                            | IRPrimitive::UInt64
                            | IRPrimitive::Int32
                            | IRPrimitive::Int64
                            | IRPrimitive::Float32
                            | IRPrimitive::Float64
                            | IRPrimitive::Timestamp
                            | IRPrimitive::TimestampMillis
                    )
                );
                let keycheck = if numeric_key {
                    format!(
                        "Wire.expect(key.trim() !== '', itemPath, 'numeric key'); {}",
                        self.check(key, "Number(key)", "itemPath")
                    )
                } else {
                    self.check(key, "key", "itemPath")
                };
                let inner = self.check(item, "item", "itemPath");
                if keycheck.is_empty() && inner.is_empty() {
                    format!("Wire.object({value}, {path});")
                } else {
                    let key_name = if keycheck.is_empty() { "_key" } else { "key" };
                    let item_name = if inner.is_empty() { "_item" } else { "item" };
                    format!("Wire.visitMap({value}, {path}, ({key_name}, {item_name}, itemPath) => {{ {keycheck} {inner} }});")
                }
            }
        }
    }
}

pub(super) fn render(package: &str, types: &[IRType], strict: bool) -> Result<String> {
    let mut ctx = Context {
        package,
        imports: BTreeSet::new(),
    };
    let mut body = String::new();
    for ty in types {
        let name = ty.name();
        body.push_str(&format!(
            "\nexport function decode{name}(value: unknown, path = {}): models.{name} {{\n",
            quoted(name)
        ));
        match ty {
            IRType::Struct(s) => {
                if s.fields.is_empty() && !strict {
                    body.push_str("  Wire.object(value, path);\n");
                } else {
                    body.push_str("  const record = Wire.object(value, path);\n");
                }
                if strict {
                    let fields = s
                        .fields
                        .iter()
                        .map(|f| quoted(&to_camel_case(f.rename.as_ref().unwrap_or(&f.name))))
                        .collect::<Vec<_>>()
                        .join(", ");
                    body.push_str(&format!("  Wire.keys(record, [{fields}], path);\n"));
                }
                for f in &s.fields {
                    let field = quoted(&to_camel_case(f.rename.as_ref().unwrap_or(&f.name)));
                    let value = format!("record[{field}]");
                    let path = format!("path + '.' + {field}");
                    let check = ctx.check(&f.field_type, &value, &path);
                    if f.is_optional {
                        body.push_str(&format!("  if ({value} !== undefined) {{ {check} }}\n"));
                    } else {
                        body.push_str(&format!("  Wire.expect(Object.hasOwn(record, {field}), {path}, 'required field'); {check}\n"));
                    }
                }
            }
            IRType::Enum(e) => body.push_str(&format!(
                "  Wire.expect([{}].some(item => item === value), path, 'enum variant');\n",
                e.variants
                    .iter()
                    .map(|v| quoted(&v.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            )),
            IRType::Union(u) => {
                body.push_str(&format!(
                    "  const record = Wire.object(value, path);\n  switch (record[{}]) {{\n",
                    quoted(&u.tag_field)
                ));
                for v in &u.variants {
                    match v {
                        IRUnionVariant::Unit { name, .. } => {
                            let keys = if strict {
                                format!("Wire.keys(record, [{}], path); ", quoted(&u.tag_field))
                            } else {
                                String::new()
                            };
                            body.push_str(&format!("    case {}: {keys}break;\n", quoted(name)))
                        }
                        IRUnionVariant::Newtype { name, ty, .. } => {
                            let field = quoted(&u.content_field);
                            let check = ctx.check(
                                ty,
                                &format!("record[{field}]"),
                                &format!("path + '.' + {field}"),
                            );
                            let keys = if strict {
                                format!(
                                    "Wire.keys(record, [{}, {field}], path); ",
                                    quoted(&u.tag_field)
                                )
                            } else {
                                String::new()
                            };
                            body.push_str(&format!(
                                "    case {}: {{ {keys}Wire.expect(Object.hasOwn(record, {field}), path + '.' + {field}, 'required payload'); {check} break; }}\n",
                                quoted(name)
                            ));
                        }
                    }
                }
                body.push_str(
                    "    default: throw new TypeError(`${path}: unknown union variant`);\n  }\n",
                );
            }
            IRType::TypeAlias(a) => {
                let field = match &a.target {
                    IRTypeAliasTarget::List(t) => IRFieldType::List(Box::new(t.clone())),
                    IRTypeAliasTarget::Map(k, v) => {
                        IRFieldType::Map(Box::new(k.clone()), Box::new(v.clone()))
                    }
                };
                body.push_str(&ctx.check(&field, "value", "path"));
            }
        }
        body.push_str(&format!("  return value as models.{name};\n}}\n"));
    }
    let mut header="// Generated runtime wire decoders. Do not edit.\nimport type * as models from './index.js';\n".to_string();
    for target in ctx.imports {
        let from: Vec<_> = package.split('.').collect();
        let to: Vec<_> = target.split('.').collect();
        let shared = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
        let mut parts = vec![".."; from.len() - shared];
        parts.extend_from_slice(&to[shared..]);
        header.push_str(&format!(
            "import * as {} from '{}/decoders.js';\n",
            package_alias(&target),
            parts.join("/")
        ));
    }
    header.push_str(include_str!("../../../templates/ts/decoder_runtime.ts"));
    header.push_str(&body);
    Ok(header)
}
