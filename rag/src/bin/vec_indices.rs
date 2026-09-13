use std::env;
use anyhow::Context;
use dotenv::dotenv;
use log::{info, log_enabled, Level, LevelFilter};
use qdrant_client::{
    prelude::*,
    qdrant::{
        ScoredPoint,
        SearchPoints,
        value::Kind,
        QueryResponse
    },
    Qdrant
};
use qdrant_client::qdrant::QueryPointsBuilder;
use rag::shared::helperUtils::utils_embedding_vec_dim_with_indices;
use rag::shared::sharedUtils::VECTOR_SIZE;
use qdrant_client::qdrant::qdrant_client::QdrantClient;

// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     dotenv().ok();
//
//     env_logger::init();
//     log::set_max_level(LevelFilter::Debug);
//     if log_enabled!(Level::Debug) {
//         info!("vec_indices");
//     }
//
//     // AppName::App1("1");
//     // AppName::App2("2");
//
//     // let mut vec_embedding_indices = vec![];
//     // for(i,v) in query_embedding_indices {
//     //     vec_embedding_indices.push((i,v));
//     // }
//     // let query_embedding : Vec<(i32, f32)> = vec![];
//     // if vec_embedding_indices.len() > 0 {
//     //     query_embedding = utils_embedding_vec_dim_with_indices("hai apa kabar kamu?",VECTOR_SIZE);
//     // }
//
//     let mut query_embedding = vec![(0,0.1)];
//     if query_embedding.len() > 0 {
//         query_embedding.remove(0);
//         query_embedding = utils_embedding_vec_dim_with_indices("hai apa kabar kamu?", VECTOR_SIZE);
//     }
//
//     //https://qdrant.tech/documentation/concepts/search/
//     println!("query embedding {:?}",query_embedding.to_vec());
//     // println!("query embedding rev {:?}",query_embedding);
//
//     let qdrant_url = env::var("QDRANT_URL").context("QDRANT_URL belum diatur")?;
//     let mut collection_name = env::var("SCHEMA_DB").context("SCHEMA_DB belum diatur")?;
//     let mut QDRANT_URL_PORT_6334=  env::var("QDRANT_URL_PORT_6334").context("QDRANT_URL_PORT_6334 belum diatur")?;
//     let client = Qdrant::from_url(&*QDRANT_URL_PORT_6334).build()?;
//
//     let mut vec_index = vec![(0, 0.0)];
//     if (query_embedding.len() > 1){
//         vec_index.remove(0);
//         for (i,v ) in query_embedding {
//             vec_index.push((i,v));
//         }
//     }
//     // let mut vec_index = vec![];
//     // vec_index.push((1, 0.1));
//
//     // For dense vectors
//     let dense_vector: Vec<f32> = query_embedding.iter()
//         .is_sorted_by_key(|(i, _)| *i) // sort by index
//         .map(|(_, v)| *v)
//         .collect();
//     // `bool` is not an iterator [E0599]
//     // Note: the following trait bounds were not satisfied:
//     // `bool: Iterator`
//     // which is required by `&mut bool: Iterator`
//
//     let search_result = client
//         .query(
//             QueryPointsBuilder::new(collection_name.to_string())
//                 .query(Query::new_nearest(dense_vector)) // Use Query::Nearest for dense vectors
//                 .limit(5)
//                 .using("text") // or your vector field name
//         )
//         .await?;
//
    // let search_result = client
    //     .query(
    //         QueryPointsBuilder::new(collection_name.to_string())
    //             .query(vec_index)//Trait `From<Vec<(i32, f32), Global>>` is not implemented for `Query` [E0277] because of  vec_index.push((i,v));
    //             .limit(5)
    //             .using("text"),
    //     )
    //     .await?;//VECTOR_SIZE as u64
//
//     Ok(())
//
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    env_logger::init();
    log::set_max_level(LevelFilter::Debug);

    if log_enabled!(Level::Debug) {
        info!("Starting query");
    }

    //1
    println!("****************************** 1 *********************************");
    let query_embedding_tuples = utils_embedding_vec_dim_with_indices("hai apa kabar kamu?", VECTOR_SIZE);
    println!("Query embedding tuples: {:?}", query_embedding_tuples);

    let collection_name = env::var("ASIST_MASTER").context("ASIST_MASTER belum diatur")?;
    let qdrant_url = env::var("QDRANT_URL_PORT_6334").context("QDRANT_URL_PORT_6334 belum diatur")?;

    // Create client
    let config = QdrantClient::from(&qdrant_url);
    let client = Qdrant::from_url(Some(config))?;

    // Convert (i32, f32) tuples to dense vector
    let mut sorted_embedding: Vec<(i32, f32)> = query_embedding_tuples;
    sorted_embedding.sort_by_key(|(i, _)| *i);

    let dense_vector: Vec<f32> = sorted_embedding
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    println!("Dense vector length: {}", dense_vector.len());

    // Perform search using SearchPoints
    let search_result = client
        .search_points(&SearchPoints {
            collection_name: collection_name.clone(),
            vector: dense_vector,
            limit: 5,
            with_payload: Some(true.into()),
            with_vectors: None,
            filter: None,
            ..Default::default()
        })
        .await?;

    // Print results for first approach - FIXED
    println!("\nSearch Results (QdrantClient approach):");
    println!("Found {} results", search_result.result.len());

    for (i, scored_point) in search_result.result.iter().enumerate() {
        println!("\nResult #{}:", i + 1);
        println!("  ID: {:?}", scored_point.id);
        println!("  Score: {:.4}", scored_point.score);

        // FIXED: payload is not an Option, it's a HashMap directly
        let payload = &scored_point.payload;
        if !payload.is_empty() {
            println!("  Payload:");
            for (key, value) in payload {
                match &value.kind {
                    Some(Kind::StringValue(s)) => println!("    {}: {}", key, s),
                    Some(Kind::IntegerValue(n)) => println!("    {}: {}", key, n),
                    Some(Kind::BoolValue(b)) => println!("    {}: {}", key, b),
                    Some(Kind::DoubleValue(d)) => println!("    {}: {}", key, d),
                    Some(Kind::ListValue(list)) => println!("    {}: {:?}", key, list),
                    Some(Kind::StructValue(st)) => println!("    {}: {:?}", key, st),
                    _ => println!("    {}: {:?}", key, value),
                }
            }
        } else {
            println!("  Payload: Empty");
        }

        if let Some(vectors) = &scored_point.vectors {
            println!("  Has vectors: Yes");
        } else {
            println!("  Has vectors: No");
        }
    }

    //2
    // println!("******************************  2 *********************************");
    // // Get environment variables again (or reuse)
    // let collection_name2 = env::var("ASIST_MASTER").context("ASIST_MASTER belum diatur")?;
    // let qdrant_url2 = env::var("QDRANT_URL_PORT_6334").context("QDRANT_URL_PORT_6334 belum diatur")?;
    //
    // let client2 = Qdrant::from_url(&qdrant_url2).build()?;
    //
    // let query_embedding_tuples2 = utils_embedding_vec_dim_with_indices("hai apa kabar kamu?", VECTOR_SIZE);
    // println!("Query embedding tuples: {:?}", query_embedding_tuples2);
    //
    // let mut sorted_embedding2: Vec<(i32, f32)> = query_embedding_tuples2;
    // sorted_embedding2.sort_by_key(|(i, _)| *i);
    //
    // let dense_vector2: Vec<f32> = sorted_embedding2
    //     .into_iter()
    //     .map(|(_, value)| value)
    //     .collect();
    //
    // println!("Dense vector length: {}", dense_vector2.len());
    //
    // let search_result2 = client2
    //     .query(
    //         QueryPointsBuilder::new(collection_name2.to_string())
    //             .query(dense_vector2)
    //             .limit(5)
    //             .using("text"), // text, vectorm embedding
    //     )
    //     .await?;

    //2
    println!("******************************  2 *********************************");
    // Get environment variables again (or reuse)
    let collection_name2 = env::var("ASIST_MASTER").context("ASIST_MASTER belum diatur")?;
    let qdrant_url2 = env::var("QDRANT_URL_PORT_6334").context("QDRANT_URL_PORT_6334 belum diatur")?;

    let client2 = Qdrant::from_url(&qdrant_url2).build()?;

    let query_embedding_tuples2 = utils_embedding_vec_dim_with_indices("hai apa kabar kamu?", VECTOR_SIZE);
    println!("Query embedding tuples: {:?}", query_embedding_tuples2);

    let mut sorted_embedding2: Vec<(i32, f32)> = query_embedding_tuples2;
    sorted_embedding2.sort_by_key(|(i, _)| *i);

    let dense_vector2: Vec<f32> = sorted_embedding2
        .into_iter()
        .map(|(_, value)| value)
        .collect();

    println!("Dense vector length: {}", dense_vector2.len());

    // Option 1: Remove .using("text") to use default vector field
    let search_result2 = client2
        .query(
            QueryPointsBuilder::new(collection_name2.to_string())
                .query(dense_vector2)
                .limit(5)  // Changed from 384 to 5 to match first approach
                // .using("vector")
        )
        .await?;

    //using can work only in:
    //curl -X PUT http://localhost:6334/collections/test_collection \
    //   -H 'Content-Type: application/json' \
    //   -d '{
    //     "vectors": {
    //       "text": {
    //         "size": 384,
    //         "distance": "Cosine"
    //       }
    //     }
    //   }'
    //atau mulai dari collection
    //cara 1:
    //curl -X PUT http://localhost:6334/collections/ASIST_MASTER \
    //   -H 'Content-Type: application/json' \
    //   -d '{
    //     "vectors": {
    //       "size": 384,
    //       "distance": "Cosine"
    //     }
    //   }'
    //
    //upsert (notice "vector" not "vectors"):
    //curl -X PUT http://localhost:6334/collections/ASIST_MASTER/points \
    //   -H 'Content-Type: application/json' \
    //   -d '{
    //     "points": [
    //       {
    //         "id": 1,
    //         "vector": [0.1, 0.2, 0.3, ...],  # 384 values
    //         "payload": {"text": "Hello world"}
    //       }
    //     ]
    //   }'
    //
    //cara 2:
    //curl -X PUT http://localhost:6334/collections/ASIST_MASTER \
    //   -H 'Content-Type: application/json' \
    //   -d '{
    //     "vectors": {
    //       "text": {
    //         "size": 384,
    //         "distance": "Cosine"
    //       }
    //     }
    //   }'
    //
    //vector not vectors in upsert
    //curl -X PUT http://localhost:6334/collections/ASIST_MASTER/points \
    //   -H 'Content-Type: application/json' \
    //   -d '{
    //     "points": [
    //       {
    //         "id": 1,
    //         "vectors": {
    //           "text": [0.1, 0.2, 0.3, ...]  # Named vector field
    //         },
    //         "payload": {"text": "Hello world"}
    //       }
    //     ]
    //   }'

    //check eksisting:
    //curl -X POST http://localhost:6334/collections/ASIST_MASTER/points \
    //   -H 'Content-Type: application/json' \
    //   -d '{
    //     "ids": [1],
    //     "with_vectors": true
    //   }'
    //· "vector": [...] → Default vector field
    // · "vectors": {"text": [...]} → Named vector field "text" ---> using harus pakai ini
    //
    // check eksisting data store:
    //curl -X POST http://localhost:6334/collections/ASIST_MASTER/points/scroll \
    //   -H 'Content-Type: application/json' \
    //   -d '{
    //     "limit": 1,
    //     "with_vectors": true
    //   }'

    println!("\nSearch Results (Qdrant struct approach):");
    println!("Found {} results", search_result2.result.len());

    for (i, scored_point) in search_result2.result.iter().enumerate() {
        println!("\nResult #{}:", i + 1);
        println!("  ID: {:?}", scored_point.id);
        println!("  Score: {:.4}", scored_point.score);

        // FIXED: In QueryResponse, payload is also a HashMap directly
        let payload = &scored_point.payload;
        if !payload.is_empty() {
            println!("  Payload:");
            for (key, value) in payload {
                match &value.kind {
                    Some(Kind::StringValue(s)) => println!("    {}: {}", key, s),
                    Some(Kind::IntegerValue(n)) => println!("    {}: {}", key, n),
                    Some(Kind::BoolValue(b)) => println!("    {}: {}", key, b),
                    Some(Kind::DoubleValue(d)) => println!("    {}: {}", key, d),
                    Some(Kind::ListValue(list)) => println!("    {}: {:?}", key, list),
                    Some(Kind::StructValue(st)) => println!("    {}: {:?}", key, st),
                    _ => println!("    {}: {:?}", key, value),
                }
            }
        } else {
            println!("  Payload: Empty");
        }

        if let Some(vectors) = &scored_point.vectors {
            println!("  Has vectors: Yes");
        } else {
            println!("  Has vectors: No");
        }
    }

    Ok(())
}
