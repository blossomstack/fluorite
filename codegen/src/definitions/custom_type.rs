use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CustomType {
    Object {
        name: String,
        fields: crate::definitions::FieldList,
    },
    Enum {
        name: String,
        values: crate::definitions::EnumValueList,
    },
    ObjectEnum {
        name: String,
        type_tag: String,
        values: crate::definitions::EnumValueList,
    },
    List {
        name: String,
        item_type: String,
    },
    Map {
        name: String,
        key_type: String,
        value_type: String,
    },
}
