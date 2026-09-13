use crate::domain::models::llm::{CmdBody, RequestBody};
use crate::shared::helperUtils::{current_time, utils_embedding_vec};
use anyhow::{Context, Result};
use log::{debug, error, info, warn};
use serde_json::{json, Value};
use std::env;
// use qdrant_client::client::QdrantClient;
use qdrant_client;
use qdrant_client::qdrant::{Condition, Filter, PointId, ScrollPoints, WithPayloadSelector};
use serde_json::{to_value};
use crate::domain::dbs_rag_LM;
use crate::domain::dbs_rag_LM::{PointsData, PointsWrapper, QdDetail};
use std::hash::{Hash, Hasher};
use std::io::{BufReader};
use mime::APPLICATION_JSON;
use regex::Regex;
use reqwest::{header, Client, Method};
use reqwest::header::HeaderValue;
use crate::shared::secureUtils::generate_api_key;
use qdrant_client::Qdrant;

pub async fn log_hist_pipeline(
    // cag_pipeline
    response: String,
    body: CmdBody,
) -> anyhow::Result<String> {
    let log = "CAG Pipeline".to_string();
    info!("{}", log.clone());


    info!("{} - {}", log.clone(), "Done");
    let context: Vec<String> = vec![String::from("done")];
    Ok(context.join("\n\n"))
}

pub async fn save_data_detail(//cag and by user
    response: String,
    body: CmdBody,
) -> anyhow::Result<String> {
    let log = "Log Detail Knowledge - Save Data".to_string();
    info!("{}", log.clone());

    //1. verified user atau id

    //2. save to qdrant vec and chunk data
    let _ = upsert_result(body.prompt.clone(), body, generate_api_key(), &response.as_str())
        .await;
        //todo: .expect("Log Detail Knowledge * Save Data * Info");

    info!("{} - {}", log.clone(), "Done");
    let context: Vec<String> = vec![String::from("done")];
    Ok(context.join("\n\n"))
}

async fn upsert_result(title_data: String, cmd_body: CmdBody, uuid: String, resp_text:&str) -> Result<()> {
    let log = "Log Detail Knowledge - Save Data - Upsert Result ".to_string();
    info!("{}", log.clone());

    // define db connection
    let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL Not Set")?;
    let collection_name = env::var("PINQRDB_DETAIL").context("PINQRDB_DETAIL Not Set")?;
    // let point_id = Uuid::new_v4().to_string();
    // let embedding_vector = utils_embedding_vec_dim(resp_text, 3);
    // let string_resp_text: String = resp_text.parse().unwrap();

    // setup data
    let mut id : String = uuid;
    debug!("{:?}: id {:?}",log.clone(),id.clone());
    let mut title : String = title_data;
    debug!("{:?}: title {:?}",log.clone(),title.clone());
    // let mut content : String = resp_text.to_string();
    debug!("{:?}: content {:?}",log.clone(),"content".to_string());//content.clone()
    let datasource = QdDetail {
        id : id.to_string(),
        title : title.to_string(),
        content : "content".to_string()
    };//content.clone()

    let mut collected_tags: Vec<String> = Vec::new();
    collected_tags.push("qdrant".to_string());
    collected_tags.push("embedding".to_string());
    collected_tags.push("pipeline".to_string());
    collected_tags.push("datatable".to_string());
    // collected_tags.push("fixing".to_string());
    // collected_tags.push("flow".to_string());
    // collected_tags.push("code".to_string());
    collected_tags.push("detail".to_string());
    if !cmd_body.tags.trim().is_empty() || cmd_body.tags.trim() != "" {
        let re = Regex::new(r"^\[([^\]]+)\]\[([^\]]+)\]\[([^\]]*)\]\[([^\]]*)\](.*)$")?;
        if let Some(caps) = re.captures(cmd_body.tags.trim()) {
            info!("{} ~ Capture Tags", log.clone());
            collected_tags.push(caps.get(1).unwrap().as_str().to_string());
            collected_tags.push(caps.get(2).unwrap().as_str().to_string());
            collected_tags.push(caps.get(3).unwrap().as_str().to_string());
            collected_tags.push(caps.get(4).unwrap().as_str().to_string());
            collected_tags.push(caps.get(5).unwrap().as_str().to_string().trim().parse()?);
        }
    }

    let payload_datasource_json = serde_json::to_string(&datasource.clone())?;
    let payload = dbs_rag_LM::Payload {
        title: title,
        // tags: vec!["qdrant".to_string(), "embedding".to_string(),
        //            "pipeline".to_string(), "datatable".to_string(),
        //            "fixing".to_string(), "flow".to_string(),
        //            "detail".to_string(),],//correction, etc, add from
        tags: collected_tags,
        data: payload_datasource_json,
        source: "asist_collection_master".to_string(),//di master sourcenya dari srv-apis
        date: "current_time".to_string(),
        similiarity_score: "".to_string(),
        precision_score: "".to_string(),
        troubleshot: "".to_string(),
        instruction_hint: "".to_string(),
        response_struct_result: "".to_string(),
    };
    let payload_str : Value = to_value(&payload)?;
    info!("{:?}: payload {:?}",log.clone(),payload_str.clone());

    let full_text = format!("{:?}", &payload_str.clone());
    let embedding = utils_embedding_vec(&*full_text);
    info!("{:?}: embedding {:?}",log.clone(),embedding.clone());

    /***
    * Validate Eksisting Data
    **/
    let mut last_id: i32 = 0;
    let client = Qdrant::from_url(&*qdrant_url).build()?;
    if let Some(existing_id) = check_existing_point(&client, &*collection_name, &payload).await? {
        info!("{:?}: embedding {:?} (ID: {:?}) ",
            log.clone(),
            "Point with payload exists",
            existing_id);
    } else {
        if let Err(e) = upsert_to_qdrant(payload_str, embedding, &*qdrant_url, &*collection_name).await {
            error!("Error Processing Row Upsert Vec To Qdrant (ID: {}): {:?}", id, e);
        } else {
            last_id = id.parse()?;
        }
        info!("{:?}: Last ID {:?} Succeed {:?}",log.clone(), last_id.clone(), current_time());
    }

    info!("{} - {}", log.clone(), "Done");
    Ok(())
}

async fn check_existing_point(
    client: &Qdrant,
    collection_name: &str,
    payload: &dbs_rag_LM::Payload,
) -> Result<Option<PointId>> {
    let log = "Log Detail Knowledge - Upsert Result - Check Existing Point ".to_string();
    info!("{}", log.clone());

    //Option 1:
    let QDRANT_URL_PORT_6334 = env::var("QDRANT_URL_PORT_6334")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    let client = Qdrant::from_url(&*QDRANT_URL_PORT_6334).build()?;
    //Option 2:
    // let client = Qdrant::from_url("http://localhost:6334").build()?;
    let mut filter: Filter = Filter::default();
    if(!payload.data.clone().is_empty()){
        filter = Filter::must([
            // Condition::matches("title", payload.title.clone()),
            // Condition::matches("tags", payload.tags.clone()),
            Condition::matches("data", payload.data.clone()),
            // Condition::matches("source", payload.source.clone()),
        ]);
    }
    // let filter = Filter::must([
    //     // Condition::matches("title", payload.title.clone()),
    //     // Condition::matches("tags", payload.tags.clone()),
    //     Condition::matches("data", payload.data.clone()),
    //     // Condition::matches("source", payload.source.clone()),
    // ]);

    // Tags
    // for tag in &payload.tags {
    //     conditions.push(Condition::matches("tags", tag.clone()));
    // }
    // // Also check that the tags array has the exact same count
    // conditions.push(Condition::values_count("tags", ValuesCount {
    //     gte: Some(payload.tags.len() as u64),
    //     lte: Some(payload.tags.len() as u64),
    //     ..Default::default()
    // }));
    // let filter = Filter::must(conditions);

    let search_result = client
        .scroll(ScrollPoints {
            collection_name: collection_name.to_string(),
            filter: Some(filter),
            limit: Some(1),
            with_payload: Some(WithPayloadSelector {
                selector_options: Some(
                    qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true),
                ),
            }),
            ..Default::default()
        })
        .await?;

    //Option 2:
    // let client = Qdrant::from_url("http://localhost:6334").build()?;
    // let search_result = client
    //     .scroll(
    //         ScrollPointsBuilder::new(collection_name)
    //             .filter(Filter::must([
    //             Condition::matches("data", payload.data.clone()),
    //             ]))
    //             .limit(1)
    //             .with_payload(true)
    //             .with_vectors(false),
    //     )
    //     .await?;


    if let Some(point) = search_result.result.first() {
        // Check if tags also match
        // if let Some(existing_tags_value) = point.payload.get("tags") {
        //     if let Some(existing_tags) = value_to_string_list(existing_tags_value) {
        //         if existing_tags == payload.tags {
        //             return Ok(point.id.clone());
        //     }
        // }
        // }

        if payload.title.len() > 0 {
            info!("{} - Found existing point with title: {} data {:?}", log.clone(), payload.title, payload.data);
            return Ok(point.id.clone());
        }
    }

    Ok(None)
}

pub async fn upsert_to_qdrant(
    payload: Value,
    embedding: Vec<f32>,
    qdrant_url: &str,
    collection: &str,
) -> Result<()> {
    let log = "Log Detail Knowledge - Check Existing Point - Upsert Data ".to_string();
    info!("{}", log.clone());

    let body = json!(
        PointsWrapper { points: vec![
            PointsData{
                id: generate_api_key(),
                vector: embedding,
                payload: to_value(&payload)?,
            }
        ] }
    );

    let mut headers = header::HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(APPLICATION_JSON.as_ref()));
    let client = Client::builder().default_headers(headers).build()?;
    let url = format!(
        "{}/collections/{}/points?wait=true",
        qdrant_url,
        collection
    );
    let resp = client
        .request(Method::PUT, &url)
        .json(&body)
        .send()
        .await?;
    let status = resp.status();
    let text = resp.text().await?;
    info!("{} - Status: {}\nBody: {}", log.clone(), status, text);
    info!("{} - Done ", log.clone());

    Ok(())
}
