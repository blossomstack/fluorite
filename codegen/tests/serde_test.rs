use fluorite::definitions::*;

#[test]
fn can_serialize_and_deserialize() -> anyhow::Result<()> {
    let field = Field {
        name: "name".to_string(),
        field_type: "String".to_string(),
        optional: Some(true),
        config: None,
    };
    let fields = vec![field];
    let user_type = CustomType::Object {
        name: "User".to_string(),
        fields,
    };
    let definition = Definition {
        types: vec![user_type],
        configs: None,
    };

    let serialized = serde_yaml::to_string(&definition)?;
    let deserialized = serde_yaml::from_str(serialized.as_str())?;
    assert_eq!(definition, deserialized);
    Ok(())
}
