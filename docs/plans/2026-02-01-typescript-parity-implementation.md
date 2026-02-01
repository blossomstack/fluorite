# TypeScript Code Generation Parity Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Bring TypeScript code generation to parity with Rust's production-ready features including new primitive types, documentation support, and deprecation annotations.

**Architecture:** The TypeScript generator reuses the shared IR (Intermediate Representation) layer, validation, and FileSystem abstraction. Only the type formatting and templates are TypeScript-specific.

**Tech Stack:** Rust, askama templates, IR layer (`codegen/src/code_gen/ir/`)

**Status:** COMPLETED

---

## Summary of Changes

### Task 1: Add New Primitive Types to TypeScript Generator

**Files:**
- Modified: `codegen/src/code_gen/ts/template_generator.rs:336-357`

**Changes:**
Added support for all 11 new primitive types from the production-ready features:
- `UUID` → `string`
- `Decimal` → `string`
- `Bytes` → `string` (base64 encoded)
- `Url` → `string`
- `Timestamp` → `number` (Unix epoch seconds)
- `TimestampMillis` → `number` (Unix epoch milliseconds)
- `DateTimeUtc` → `string` (ISO 8601)
- `DateTimeTz` → `string` (ISO 8601 with timezone)
- `Date` → `string` (ISO 8601 date)
- `Time` → `string` (ISO 8601 time)
- `Duration` → `string` (ISO 8601 duration)

### Task 2: Add Documentation Support to TypeScript Templates

**Files:**
- Modified: `codegen/src/code_gen/ts/templates.rs`
- Modified: `codegen/src/code_gen/ts/template_generator.rs`
- Modified: `codegen/templates/ts/interface.ts.j2`
- Modified: `codegen/templates/ts/enum.ts.j2`
- Modified: `codegen/templates/ts/union.ts.j2`
- Modified: `codegen/templates/ts/type_alias.ts.j2`

**Changes:**
1. Added `doc` field to all template structs (`InterfaceTemplate`, `TsEnumTemplate`, `TsUnionTemplate`, `TsTypeAliasTemplate`)
2. Added `doc` and `deprecated` fields to `TsFieldTemplate`
3. Updated generator to pass doc strings from IR to templates
4. Updated all templates to generate JSDoc comments when `doc` is non-empty

### Task 3: Add @deprecated JSDoc Support

**Files:**
- Modified: `codegen/templates/ts/interface.ts.j2`

**Changes:**
Added `@deprecated` JSDoc annotation support for deprecated fields.

### Task 4: Update Documentation

**Files:**
- Modified: `CLAUDE.md`

**Changes:**
1. Updated project description to reflect TypeScript is fully supported (not "planned")
2. Updated supported types list to include extended primitives
3. Updated supported features to include serde options and documentation
4. Added TypeScript configuration options section
5. Updated type mapping table with all new primitive types

---

## Test Results

All 86 tests pass:
- 72 Rust code generation tests
- 12 TypeScript code generation tests
- 2 serde serialization tests

---

## Type Mapping Reference

| Fluorite Type | Rust | TypeScript |
|---------------|------|------------|
| String | `String` | `string` |
| Bool | `bool` | `boolean` |
| DateTime | `chrono::NaiveDateTime` | `string` |
| UInt32/UInt64 | `u32`/`u64` | `number` |
| Int32/Int64 | `i32`/`i64` | `number` |
| Float32/Float64 | `f32`/`f64` | `number` |
| UUID | `uuid::Uuid` | `string` |
| Decimal | `rust_decimal::Decimal` | `string` |
| Bytes | `Vec<u8>` | `string` (base64) |
| Url | `url::Url` | `string` |
| Timestamp | `i64` | `number` |
| TimestampMillis | `i64` | `number` |
| DateTimeUtc | `chrono::DateTime<Utc>` | `string` |
| DateTimeTz | `chrono::DateTime<FixedOffset>` | `string` |
| Date | `chrono::NaiveDate` | `string` |
| Time | `chrono::NaiveTime` | `string` |
| Duration | `chrono::Duration` | `string` |
| Any | configurable | `unknown` (default) |
| List<T> | `Vec<T>` | `T[]` |
| Map<K,V> | `HashMap<K,V>` | `Record<K,V>` |
