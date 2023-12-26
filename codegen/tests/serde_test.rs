use fluorite_codegen::definitions::*;

#[test]
fn can_serialize_and_deserialize() -> anyhow::Result<()> {
    let field = Field {
        name: "name".to_string(),
        field_type: "String".to_string(),
        optional: Some(true),
        configs: None,
    };
    let fields = vec![field];
    let user_type = CustomType::Object {
        name: "User".to_string(),
        fields,
    };
    let definition = Definition {
        types: vec![user_type],
        configs: DefinitionConfig { rust_package: None },
    };

    let serialized = serde_yaml::to_string(&definition)?;
    println!("serialized: {}", serialized);
    let _deserialized = serde_yaml::from_str(serialized.as_str())?;
    // assert_eq!(definition, deserialized);
    Ok(())
}
