// // src/script/db.rs
// use std::ptr::null;
// use tokio_postgres::Client;
// use tokio_postgres::Error;
// use chrono::{DateTime, Utc, NaiveDateTime, Local};
// use crate::detect_objects_on_image;
// use std::fs::File;
// use std::io::Write;
// use std::path::Path;
// use crate::scripts_modules::client::{handleQsAsr, load_gguf_file};
//
//
// pub async fn select_from_db(client: &Client) -> Result<(), Error> {
//     let rows = client.query("SELECT * FROM t_temp_file WHERE content_treshold IS NULL AND content_label IS NULL AND file_id='10' ", &[]).await?;
//     if(!rows.is_empty()){
//         println!("log selecting data: found row");
//         for row in rows {
//             let id: String = row.get(0);
//             println!("log selecting data: found id {} ",id);
//             let typeFile: String = row.get(1);
//             let now: DateTime<Utc> = Utc::now();
//             println!("log selecting data: Found row: id = {}, type = {}, At : {}", id, typeFile, now);
//             let bytea_data: Option<Vec<u8>> = row.get(2);
//             if let Some(data) = bytea_data {
//                 let bytea_data: Vec<u8> = data;
//                 println!("log selecting data: Data length: {}", &bytea_data.len());
//
//                 let mut file = File::create("output.".to_string()+typeFile.as_str()).expect("Failed to create file");
//                 file.write_all(&*bytea_data).expect("Failed to write data");
//                 println!("log file object: File Created ");
//                 // let buf = std::fs::read(file.path().unwrap_or(Path::new(""))).unwrap_or(vec![]);
//                 let boxes = detect_objects_on_image(bytea_data.to_vec());
//                 println!("{}", serde_json::to_string(&boxes).unwrap_or_default());
//                 println!("log file object: Finish Detect Object ");
//
//                 println!("update data: Start ");
//                 let mut new_treshold_value = "1";
//                 println!("update data: Check LLM PGD ");
//                 let answer = "confirm";
//                 let tag = "";
//                 let mut gguf = "".to_string();
//                 gguf = handleQsAsr(tag);
//                 if(answer==gguf){
//                     new_treshold_value = "2";
//                 }
//
//                 println!("update data: Label ");
//                 let new_label_value = "[image]";
//                 let id_value: String = id.clone().to_string();
//                 println!("update data: start update data table ");
//                 info!( "Executing query: UPDATE t_temp_file SET content_treshold = '{}', content_label = '{}' WHERE file_id = '{}' ", new_treshold_value, new_label_value, id_value );
//
//                 let updated_rows = client
//                     .execute(
//                         "UPDATE t_temp_file SET content_treshold = $1, content_label = $2 WHERE file_id = $3 ",
//                         &[&new_treshold_value, &new_label_value, &id_value],
//                     )
//                     .await?;
//                 println!("update data: success update data table ");
//                 println!("Number of rows updated: {}", updated_rows);
//                 /***
//                  ** Sent To MN IO
//                  */
//                 //in progres
//             }
//             println!("Save And Update row: id = {}, type = {}, At : {}", id, typeFile, now);
//         }
//     }else{
//         let now = Local::now();
//         let timestamp: i64 = now.timestamp();
//         let naive_datetime = NaiveDateTime::from_timestamp(timestamp, 0);
//         let local_datetime: DateTime<Local> = DateTime::from_utc(naive_datetime, *now.offset());
//         println!(" log: llm image running at {} ",local_datetime);
//     }
//
//     Ok(())
// }
