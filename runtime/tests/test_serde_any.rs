use std::collections::HashMap;

use fluorite::Any;

#[test]
fn can_serialize_deserialize() -> anyhow::Result<()> {
    let v1 = Any::String("test".to_owned());
    let v2 = Any::Bool(true);
    // let v3 = Any::UInt32(32);
    let v4 = Any::UInt64(64);
    // let v5 = Any::Int32(-32);
    let v6 = Any::Int64(-64);
    // let v7 = Any::Float32(32.32);
    let v8 = Any::Float64(64.64);
    let list1 = Any::List(vec![v1.clone(), Any::String("test2".to_owned())]);
    let mut map = HashMap::new();
    map.insert("k1".to_owned(), v1.clone());
    map.insert("k2".to_owned(), v2.clone());
    map.insert("k4".to_owned(), v4.clone());
    map.insert("list".to_owned(), list1.clone());
    let map1 = Any::Map(map);
    let mut map2 = HashMap::new();
    // map2.insert("k1".to_owned(), v5.clone());
    map2.insert("k2".to_owned(), list1.clone());
    map2.insert("k3".to_owned(), map1.clone());
    let map2 = Any::Map(map2);

    let values = vec![v1, v2, v4, v6, v8, list1, map1, map2];

    for v in values {
        let serialized = serde_json::to_string(&v)?;
        let deserialized: Any = serde_json::from_str(serialized.as_str())?;
        assert_eq!(v, deserialized);
    }
    Ok(())
}
