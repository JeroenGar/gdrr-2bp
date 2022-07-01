use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct JsonInstance {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Objects")]
    pub sheettypes: Vec<JsonSheetType>,
    #[serde(rename = "Items")]
    pub parttypes: Vec<JsonPartType>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct JsonSolution {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Objects")]
    pub sheettypes: Vec<JsonSheetType>,
    #[serde(rename = "Items")]
    pub parttypes: Vec<JsonPartType>,
    #[serde(rename = "CuttingPatterns")]
    pub cutting_patterns: Vec<JsonCP>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct JsonSheetType {
    pub length: u64,
    pub height: u64,
    pub stock: Option<usize>,
    pub cost: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct JsonPartType {
    pub length: u64,
    pub height: u64,
    pub demand: usize,
    pub value: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference: Option<usize>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct JsonCP {
    pub object: usize,
    pub root : JsonCPNode
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct JsonCPNode {
    pub length: u64,
    pub height: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<JsonOrientation>,
    #[serde(rename = "Type")]
    pub node_type: JsonCPNodeType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item : Option<usize>,
    pub children: Vec<JsonCPNode>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum JsonOrientation {
    H,
    V,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub enum JsonCPNodeType {
    Structure,
    Item,
    Leftover,
}

