// // src/script/img.rs
// use tokio_postgres::NoTls;
// use cron::Schedule;
// use std::str::FromStr;
// use std::sync::Arc;
// use tokio::sync::Mutex;
// use std::time::Duration;
// use dotenv::dotenv;
// use std::env;
// use crate::scripts_modules::db::*;
// use rocket::tokio::spawn;
//
// pub async fn run_scheduler() {
//     // Define a cron schedule (e.g., every minute)
//     let schedule = Schedule::from_str("* * * * * *").expect("Failed to parse schedule");
//
//     println!("Start DB image validation");
//     dotenv().ok();
//
//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     let (client, connection) =
//         tokio_postgres::connect(&database_url, NoTls).await.expect("Failed to connect to database");
//
//     // Create a task to run the connection processing in the background
//     let connection_task = spawn(async move {
//         if let Err(e) = connection.await {
//             eprintln!("Connection error: {}", e);
//         }
//     });
//
//     // Create a shared client for the scheduled tasks
//     let shared_client = Arc::new(Mutex::new(client));
//
//     loop {
//         let now = chrono::Utc::now();
//         let next = schedule.upcoming(chrono::Utc).next().unwrap();
//         let duration = next - now;
//
//         // Wait until the next scheduled time
//         tokio::time::sleep(Duration::from_millis(duration.num_milliseconds() as u64)).await;
//
//         // Clone the shared client for the task
//         let client = Arc::clone(&shared_client);
//         spawn(async move {
//             let client = client.lock().await;
//             if let Err(e) = select_from_db(&client).await {
//                 eprintln!("Error selecting data: {}", e);
//             }
//         });
//     }
// }
