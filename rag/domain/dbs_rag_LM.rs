use serde_derive::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointsWrapper {
    pub points: Vec<PointsData>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PointsData {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QdDetail {
    pub id : String,
    pub title : String,
    pub content : String
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Payload {
    pub title: String,
    pub tags: Vec<String>,
    pub data: String,
    pub source: String,
    pub date: String,//created_date
    pub similiarity_score: String,
    pub precision_score: String,
    pub troubleshot: String,            //analize
    pub instruction_hint: String,       //describe solver
    pub response_struct_result: String  //suggestion todo solver
}
