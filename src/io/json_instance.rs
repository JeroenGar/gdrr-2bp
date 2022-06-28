use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct JsonInstance{
    #[serde(rename = "Name")]
    pub name : String,
    #[serde(rename = "Objects")]
    pub sheettypes : Vec<JsonSheetType>,
    #[serde(rename = "Items")]
    pub parttypes : Vec<JsonPartType>
}


#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct JsonSheetType{
    pub length: u64,
    pub height: u64,
    pub stock: Option<usize>,
    pub cost : u64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct JsonPartType {
    pub length : u64,
    pub height : u64,
    pub demand : usize,
    pub value : u64,
}