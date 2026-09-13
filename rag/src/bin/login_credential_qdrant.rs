// Qdrant Login Credential Query Examples
// Demonstrates how to query user credentials from Qdrant for login validation

use qdrant_client::Qdrant;
use qdrant_client::qdrant::{Filter, FieldCondition, Match};
use qdrant_client::qdrant::{Condition, PointId, ScrollPoints, WithPayloadSelector};
use std::env;
use serde::{Deserialize, Serialize};
use qdrant_client::qdrant::GetPointsBuilder;
use serde_json::Value;
use anyhow::Context;
use serde_json::{to_value};
use dotenv::dotenv;
use log::{error, info, log_enabled, warn, Level, LevelFilter};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();
    log::set_max_level(LevelFilter::Debug);
    if log_enabled!(Level::Debug) {
        info!("Tokio!");
    }

    println!("=== Qdrant Login Credential Query Examples ===\n");

    // Get Qdrant URL from environment variable or use default
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
    println!("Connecting to Qdrant at: {}", qdrant_url);

    // Connect to Qdrant
    let client = Qdrant::from_url(&qdrant_url).build()?;
    println!("Connected to Qdrant successfully\n");

    // Example 1: Query all user credentials
    println!("=== Example 1: Query All User Credentials ===");
    query_all_users(&client).await;
    //
    // // Example 2: Query user by email (for login validation)
    // println!("\n=== Example 2: Query User by Email (Login Validation) ===");
    // query_user_by_email(&client, "istamar.rozid@gmail.com").await?;
    //
    // // Example 3: Query users by role
    // println!("\n=== Example 3: Query Users by Role ===");
    // query_users_by_role(&client, "DEV").await?;
    //
    // // Example 4: Check if user exists (for login check)
    println!("\n=== Example 4: Check if User Exists ===");
    check_user_exists(&client).await?;

    Ok(())
}

/// Query all users from user_credentials collection
async fn query_all_users(client: &Qdrant) -> Result<(), Box<dyn std::error::Error>> {
    // let mut headers = header::HeaderMap::new();
    // headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(APPLICATION_JSON.as_ref()));
    // let client = Client::builder().default_headers(headers).build()?;
    // let url = format!(
    //     "{}/collections/{}/points?wait=true",
    //     qdrant_url,
    //     collection
    // );

    // let QDRANT_URL = env::var("QDRANT_URL")
    //     .unwrap_or_else(|_| "http://localhost:6333".to_string());
    // let client = Qdrant::from_url(&*QDRANT_URL).build()?;
    //
    // let mut filter: Filter = Filter::default();
    // if(!payload.data.clone().is_empty()){
    //     filter = Filter::must([
    //         // Condition::matches("title", payload.title.clone()),
    //         // Condition::matches("tags", payload.tags.clone()),
    //         Condition::matches("data", payload.data.clone()),
    //         // Condition::matches("source", payload.source.clone()),
    //     ]);
    // }
    //
    // let result = client
    //     .scroll(&ScrollPoints {
    //         collection_name: "user_credentials".to_string(),
    //         filter: Some(filter),
    //         limit: Some(1),
    //         with_payload: Some(WithPayloadSelector {
    //             selector_options: Some(
    //                 qdrant_client::qdrant::with_payload_selector::SelectorOptions::Enable(true),
    //             ),
    //         }),
    //         ..Default::default()
    //     })
    //     .await?;

        // let client = Qdrant::from_url(QDRANT_URL).build()?;
        //
        // let result = client
        //     .get_points(GetPointsBuilder::new(
        //         "user_credentials",
        //         // vec![0.into(), 30.into(), 100.into()],
        //         vec![0.641136,0.5309056,0.5541514],
        //     ))
        //     .await?;


    // let result = client
        // .scroll(&"user_credentials".to_string())
        // .with_payload(WithPayloadSelector::enable(true))
        // .limit(1000)
        // .await?;

    // if let Some(points) = result.result.points {
    //     println!("Found {} user(s):", points.len());
    //     for point in points {
    //         if let Some(payload) = point.payload {
    //             let email = payload.get("email").and_then(|v| v.as_str()).unwrap_or("N/A");
    //             let role = payload.get("role").and_then(|v| v.as_str()).unwrap_or("N/A");
    //             println!("  - Email: {}, Role: {}", email, role);
    //         }
    //     }
    // } else {
    //     println!("No users found");
    // }

    Ok(())
}

// /// Query user by email (for login validation)
// async fn query_user_by_email(client: &Qdrant, email: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let filter = Filter::must([FieldCondition::new_match("email", Match::text(email))]);
//
//     let result = client
//         .scroll(&"user_credentials".to_string())
//         .with_payload(WithPayloadSelector::enable(true))
//         .with_filter(filter)
//         .limit(1)
//         .await?;
//
//     if let Some(points) = result.result.points {
//         if let Some(point) = points.first() {
//             if let Some(payload) = &point.payload {
//                 println!("User found:");
//                 println!("  Email: {:?}", payload.get("email"));
//                 println!("  Password: {:?}", payload.get("password"));
//                 println!("  Role: {:?}", payload.get("role"));
//                 println!("  Key: {:?}", payload.get("key"));
//             }
//         } else {
//             println!("User not found with email: {}", email);
//         }
//     } else {
//         println!("User not found with email: {}", email);
//     }
//
//     Ok(())
// }
//
// /// Query users by role
// async fn query_users_by_role(client: &Qdrant, role: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let filter = Filter::must([FieldCondition::new_match("role", Match::text(role))]);
//
//     let result = client
//         .scroll(&"user_credentials".to_string())
//         .with_payload(WithPayloadSelector::enable(true))
//         .with_filter(filter)
//         .limit(1000)
//         .await?;
//
//     if let Some(points) = result.result.points {
//         println!("Found {} user(s) with role '{}':", points.len(), role);
//         for point in points {
//             if let Some(payload) = point.payload {
//                 let email = payload.get("email").and_then(|v| v.as_str()).unwrap_or("N/A");
//                 println!("  - Email: {}", email);
//             }
//         }
//     } else {
//         println!("No users found with role: {}", role);
//     }
//
//     Ok(())
// }
//


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PointsWrapperCredential {
    pub points: Vec<PointsData>,
}

#[derive(Clone, Deserialize, Serialize, Debug)]
pub struct PointsDataCredential {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QdDetailCredential {
    pub id : String,
    pub title : String,
    pub content : String
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PayloadCredential {
    pub email: String,
    pub password: String,
    // pub tags: Vec<String>,
    // pub data: String,
    // pub source: String,
    // pub date: String,//created_date
    // pub similiarity_score: String,
    // pub precision_score: String,
    // pub troubleshot: String,            //analize
    // pub instruction_hint: String,       //describe solver
    // pub response_struct_result: String  //suggestion todo solver
}


// /// Check if user exists (simplified login check)
async fn check_user_exists(client: &Qdrant) -> Result<(), Box<dyn std::error::Error>> {



    // let filter = Filter::must([FieldCondition::new_match("email", Match::text(email))]);
    //
    // let result = client
    //     .scroll(&"user_credentials".to_string())
    //     .with_payload(WithPayloadSelector::enable(true))
    //     .with_filter(filter)
    //     .limit(1)
    //     .await?;
    //
    // let exists = if let Some(points) = result.result.points {
    //     !points.is_empty()
    // } else {
    //     false
    // };
    //
    // if exists {
    //     println!("User exists: {}", email);
    // } else {
    //     println!("User does not exist: {}", email);
    // }


        //Using ID
        // let client = reqwest::Client::builder()
        //     .build()?;
        //
        // let mut headers = reqwest::header::HeaderMap::new();
        // headers.insert("Content-Type", "application/json".parse()?);
        //
        // let data = "";
        //
        // let request = client.request(reqwest::Method::GET, "http://localhost:6333/collections/user_credentials/points/1")
        //     .headers(headers)
        //     .body(data);
        //
        // let response = request.send().await?;
        // let body = response.text().await?;
        //
        // println!("{}", body);


        //Using Email
        // let qdrant_url
        let mut last_id: i32 = 0;
        let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL Not Set")?;
        let collection_name = env::var("ASIST_CREDENTIAL").context("ASIST_CREDENTIAL Not Set")?;
        let client = Qdrant::from_url(&*qdrant_url).build()?;
        // let collection_name = "user_credentials";
        let payload = PayloadCredential {
            email: "istamar.rozid@gmail.com".to_string(),
            password: "".to_string(),
        };
        let payload_str : Value = to_value(&payload)?;
        println!("payload {:?}",payload.clone());

        if let Some(existing_id) = check_existing_point(&client, &*collection_name, &payload).await? {
            println!("embedding {:?} (ID: {:?}) ",
                "Point with payload exists",
                existing_id);
        } else {
            // if let Err(e) = upsert_to_qdrant(payload_str, embedding, &*qdrant_url, &*collection_name).await {
                // error!("Error Processing Row Upsert Vec To Qdrant (ID: {}): {:?}", id, e);
            // } else {
                // last_id = id.parse()?;
            // }
            println!("User isnt eksisting, create upsert data user credential first");
        }



    Ok(())
}

async fn check_existing_point(
    client: &Qdrant,
    collection_name: &str,
    payload: &PayloadCredential,
) -> Result<Option<PointId>, anyhow::Error> {
    let log = "Log Detail Knowledge - Check Existing Point ".to_string();
    println!("{}", log.clone());

    //Option 1:
    let QDRANT_URL_PORT_6334 = env::var("QDRANT_URL_PORT_6334")
        .unwrap_or_else(|_| "http://localhost:6334".to_string());
    let client = Qdrant::from_url(&*QDRANT_URL_PORT_6334).build()?;
    //Option 2:
    // let client = Qdrant::from_url("http://localhost:6334").build()?;
    let mut filter: Filter = Filter::default();
    if(!payload.email.clone().is_empty()){
        filter = Filter::must([
            // Condition::matches("title", payload.title.clone()),
            // Condition::matches("tags", payload.tags.clone()),
            Condition::matches("email", payload.email.clone()),//payload.data.clone()
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

        if payload.email.len() > 0 {
            println!("{} - Found existing point with email: {} and password: {:?}", log.clone(), payload.email, point.payload.get("password"));
            return Ok(point.id.clone());
        }
    }

    Ok(None)
}



use chrono::{DateTime, Utc};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Payload {
    pub key: String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub create_date: DateTime<Utc>,
    pub update_date: DateTime<Utc>,
}

use rag::domain::dbs_rag_LM::{PointsData, PointsWrapper, QdDetail};
// Example: Insert user credential into Qdrant
// This shows how to insert a user with encrypted password
// #[allow(dead_code)]
// async fn insert_user_credential(
//     client: &Qdrant,
//     key: &str,
//     email: &str,
//     password: &str, // Should be AES-256-GCM encrypted Base64
//     role: &str,
// ) -> Result<(), Box<dyn std::error::Error>> {
//     use qdrant_client::qdrant::{PointStruct};
//
//     let datasource = QdDetail {
//         id : id.to_string(),
//         title : title.to_string(),
//         content : "content".to_string()
//     };//content.clone()
//
//     let payload_datasource_json = serde_json::to_string(&datasource.clone())?;
//     let point = Payload {
//         key: key.to_string(),
//         email: email.to_string(),
//         password: password.to_string(),
//         role: role.to_string(),
//         create_date: chrono::Utc::now().to_rfc3339(),
//         update_date: chrono::Utc::now().to_rfc3339(),
//     };
//     let payload_str : Value = to_value(&payload)?;
//     info!("{:?}: payload {:?}",log.clone(),payload_str.clone());
//
//     // let point = PointStruct {
//     //     id: Some(qdrant_client::qdrant::PointId::from(1)),
//     //     vector: vec![], // Empty vector for user credentials
//     //     payload: Some({
//     //         let mut payload = Payload::new();
//     //         payload.insert("key".to_string(), key.to_string());
//     //         payload.insert("email".to_string(), email.to_string());
//     //         payload.insert("password".to_string(), password.to_string());
//     //         payload.insert("role".to_string(), role.to_string());
//     //         payload.insert("create_date".to_string(), chrono::Utc::now().to_rfc3339());
//     //         payload.insert("update_date".to_string(), chrono::Utc::now().to_rfc3339());
//     //         payload
//     //     }),
//     // };
//
//     client
//         .upsert_points_blocking(&"user_credentials".to_string(), None, vec![point], None)
//         .await?;
//
//     println!("User credential inserted successfully");
//     Ok(())
// }
//
//
// // cargo run --bin login_credential_qdrant
// //user_credentials collection
//
//
// // #[tokio::main]
// // async fn main() -> Result<(), Box<dyn std::error::Error>> {
// //     let client = reqwest::Client::builder()
// //         .build()?;
// //
// //     let mut headers = reqwest::header::HeaderMap::new();
// //     headers.insert("Content-Type", "application/json".parse()?);
// //
// //     let data = "";
// //
// //     let request = client.request(reqwest::Method::GET, "http://localhost:6333/collections/asist_collection_detail/points/b559ee3f-4a97-4618-be61-5b7f2f6b6995")
// //         .headers(headers)
// //         .body(data);
// //
// //     let response = request.send().await?;
// //     let body = response.text().await?;
// //
// //     println!("{}", body);
// //
// //     Ok(())
// // }
//
//
// // #[tokio::main]
// // async fn main() -> Result<(), Box<dyn std::error::Error>> {
// //     let client = reqwest::Client::builder()
// //         .build()?;
// //     let mut headers = reqwest::header::HeaderMap::new();
// //     headers.insert("Content-Type", "application/json".parse()?);
// //     let data = "";
// //     let request = client.request(reqwest::Method::GET, "http://localhost:6333/collections/user_credentials/points/1")
// //         .headers(headers)
// //         .body(data);
// //     let response = request.send().await?;
// //     let body = response.text().await?;
// //     println!("{}", body);
// //
// //     println!("body.result.payload.password {}", body.result.payload.password);
// //
// //     Ok(())
// // }
