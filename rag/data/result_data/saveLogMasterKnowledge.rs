//menyimpan per knowledge base
use std::env;
use anyhow::Context;
use crate::domain::models::llm::{CmdBody, RequestBody};
use log::{debug, error, info};
// use qdrant_client::client::QdrantClient;
use qdrant_client;
use qdrant_client::Qdrant;
use qdrant_client::qdrant::{Condition, Filter, PointId, ScrollPoints, SearchPointsBuilder, WithPayloadSelector};
use regex::Regex;
use serde_json::{json, to_value, Value};
use uuid::Uuid;
use crate::domain::dbs_rag_LM;
use crate::domain::dbs_rag_LM::QdDetail;
use crate::data::result_data::saveLogDetailKnowledge::upsert_to_qdrant;
use crate::shared::helperUtils::{current_time, utils_embedding_vec, utils_embedding_vec_dim};
use crate::shared::secureUtils::generate_api_key;

pub async fn save_data_master(//master information and from agent protocol
    response: String,
    body: CmdBody,
) -> anyhow::Result<String> {
    let log = "Log Detail Knowledge - Save Data".to_string();
    info!("{}", log.clone());

    //1. verified user atau id

    //2. save to qdrant vec and chunk data
    let _ = upsert_result_master(body.prompt.clone(), body, generate_api_key(), &response.as_str())
        .await;
    //todo: .expect("Log Detail Knowledge * Save Data * Info");

    info!("{} - {}", log.clone(), "Done");
    let context: Vec<String> = vec![String::from("done")];
    Ok(context.join("\n\n"))
}

/// VEC AND EMBEDDING TO QDRANT DB
async fn upsert_result_master (title_data: String, cmd_body: CmdBody, uuid: String, resp_text:&str) -> anyhow::Result<()> {
// async fn upsert_result_master(
//     // responseLog: String,
//     // response: String,
//     // body: CmdBody
//     title_data: String, cmd_body: CmdBody, uuid: String, resp_text:&str
// )  ->
    // anyhow::Result<String> {
    //
    // //1. verified user atau id dan sumber
    //
    // //2. save to qdrant vec and chunk data
    //
    // info!("retrieve_context start");
    // // const LOCAL_LLM_MODEL: &str = "llama3.2:latest"; // Default local LLM cari dari ollama local
    // let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL belum diatur")?;
    // info!("qdrant_url start {} ",qdrant_url);
    // let mut COLLECTION_NAME = env::var("PINQRDB_DETAIL").context("PINQRDB_DETAIL belum diatur")?;
    // info!("COLLECTION_NAME start {} ",COLLECTION_NAME);
    //
    // // let query_embedding = generate_embedding_local(query).await?;
    // let query_embedding = utils_embedding_vec_dim(cmd_body.prompt.as_str(), 3);
    // info!("query_embedding {:?}",query_embedding);
    //
    //
    // info!(" Search Qdrant for relevant chunks - START");
    // // Search Qdrant for relevant chunks
    // // http://localhost:6333/collections/:collection_name/points/:id
    // // let search_result = qdrant_client
    // //     .search_points(&SearchPoints {
    // //         collection_name: COLLECTION_NAME.to_string(),
    // //         vector: query_embedding,
    // //         limit: 3, // Get top 3 relevant chunks
    // //         with_payload: Some(true.into()),
    // //         ..Default::default()
    // //     }).await?;
    // //
    // let QDRANT_URL_PORT_6334 = env::var("QDRANT_URL_PORT_6334")
    //     .unwrap_or_else(|_| "http://localhost:6334".to_string());
    // // let client = Qdrant::from_url(&*QDRANT_URL_PORT_6334).build()?;
    // let client = Qdrant::from_url(&*QDRANT_URL_PORT_6334).build()?;
    // // let search_request = self
    // //     .client
    // //     .search_points(
    // //         SearchPointsBuilder::new("embeddings".to_string(), query_vector, 5)
    // //             .with_payload(true)
    // //             .params(SearchParamsBuilder::default().exact(true)),
    // //     );
    //
    // let mut search_request = SearchPointsBuilder::new(
    //     COLLECTION_NAME.to_string(),    // Collection name
    //     query_embedding, // Search vector
    //     3,     // Search limit, number of results to return
    // ).with_payload(true);
    // // let &_request = &search_request;
    // // println!(" search_request {:?} ", &search_request);
    // let search_result = client.search_points(search_request).await?;
    // info!(" Search Qdrant for relevant chunks - DONE");
    // info!(" Search Result: {:?}", search_result);
    //
    // info!("Extract text from results - START");
    // // Extract text from results
    // let context: Vec<String> = search_result
    //     .result
    //     .into_iter()
    //     .filter_map(|point| {
    //         point
    //             .payload
    //             .get("text")
    //             .and_then(|v| v.as_str())
    //             .map(|s| s.to_string())
    //     })
    //     .collect();
    // info!("Extract text from results - DONE");
    //
    // Ok(context.join("\n\n"))

    let log = "Log Master Knowledge - Save Data - Upsert Result ".to_string();
    info!("{}", log.clone());

    // define db connection
    let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL Not Set")?;
    let collection_name = env::var("ASIST_MASTER").context("ASIST_MASTER Not Set")?;
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
        source: format!("{} * {}","asist_collection_master".to_string(),"agent_protocol_sources"),//di master sourcenya dari srv-apis
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
    // let client = Qdrant::from_url(&*qdrant_url).build()?;
    let client = Qdrant::from_url(&*qdrant_url).build()?;
    if let Some(existing_id) = check_existing_point_master(&client, &*collection_name, &payload).await? {
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


async fn check_existing_point_master(
    client: &Qdrant,
    collection_name: &str,
    payload: &dbs_rag_LM::Payload,
) -> anyhow::Result<Option<PointId>> {
    let log = "Log Master Knowledge - Upsert Result - Check Existing Point ".to_string();
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


//note: rag---agentprotocol---sources
//       |         |
//     detail   master
