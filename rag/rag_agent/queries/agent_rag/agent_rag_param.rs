use anyhow::Context;
use log::error;
use reqwest::{header, Client, Method, ClientBuilder};
use reqwest::header::HeaderValue;
use tokio_postgres::{NoTls, Row};
use crate::secure::error::Error;

pub async fn param_db_get_master_dataset(
    db_conn_str: &str,
    qdrant_url: &str,
    vector_url: &str,
    collection: &str,
    mut name_data: &str,
) -> anyhow::Result<(Vec<Row>)> {
    let (client, connection)
        = tokio_postgres::connect(db_conn_str, NoTls)
        .await
        .context("Query Master Dataset: Error Connect PostgresSQL For CDC")?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Query Master Dataset: Error Connect PostgresSQL: {}", e);
        }
    });

    let rows = client.
        query("SELECT id_data, name \
        FROM t_dataset_master \
        WHERE name > $1 ORDER BY id ASC"
              ,&[&name_data],).await
        .context("Query Master Dataset: Get New Dataset")?;
    Ok(rows)
}