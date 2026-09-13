use warp::{Filter, Rejection, Reply};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use chrono::Local;
use std::fs;
use std::io;

#[derive(Debug, Deserialize)]
pub(crate) struct QueryParams {
    tags: String,
    prompt: String,
}

#[derive(Debug, Clone, Serialize)]
enum Role {
    User,
}

#[derive(Debug)]
struct Auth {
    role: Role,
}

#[derive(Debug, Serialize)]
struct ApiError {
    message: String,
    code: u16,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "API Error {}: {}", self.code, self.message)
    }
}

impl std::error::Error for ApiError {}

impl warp::reject::Reject for ApiError {}

pub async fn quickThink_helper_handler(
    username: String,
    query: QueryParams,
    body: Bytes,
) -> Result<impl Reply, Rejection> {
    println!("User: {}", username);
    println!("Received query params: tags={}, prompt={}", query.tags, query.prompt);
    println!("Received {} bytes of data", body.len());

    let _auth = Auth { role: Role::User };
    let now = Local::now();
    // let path = "c:\\Users\\nameLocation\\";
    let path = "c:\\\\Users\\\\istamar.nugraha\\\\DEV_ISTAMAR\\\\WORKSPACE_ISTA\\\\src_code_rag_check\\\\";
    let name_folder = now.format("%Y%m%d_%H%M%S").to_string();
    let zip_path = format!("{}{}.zip", path, name_folder);

    println!("Zip file path: {}", zip_path);

    // Save the zip file
    tokio::fs::write(PathBuf::from(zip_path.clone()), &body)
        .await
        .map_err(|e| {
            eprintln!("Failed to save file: {}", e);
            warp::reject::custom(ApiError {
                message: format!("File save failed: {}", e),
                code: 500,
            })
        })?;

    // Extract the zip file to a folder
    let extract_dir = format!("{}{}", path, name_folder);
    extract_zip(&zip_path, &extract_dir).map_err(|e| {
        warp::reject::custom(ApiError {
            message: format!("Failed to extract zip: {}", e),
            code: 500,
        })
    })?;

    println!("File uploaded and extracted successfully to: {}", extract_dir);

    Ok(warp::reply::json(&format!(
        "File uploaded and extracted successfully to: {}",
        extract_dir
    )))
}

fn extract_zip(zip_path: &str, extract_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create the extraction directory if it doesn't exist
    let extract_path = PathBuf::from(extract_dir);
    if !extract_path.exists() {
        fs::create_dir_all(&extract_path)?;
        println!("Created extraction directory: {}", extract_dir);
    }

    // Open the zip file
    let file = fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    println!("Extracting {} files to: {}", archive.len(), extract_dir);

    // Extract each file in the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        // Get the file path within the zip
        let file_path = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        // Create the full output path
        let out_path = extract_path.join(&file_path);

        // Create parent directories if needed
        if let Some(parent) = out_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Handle directories
        if file.is_dir() {
            println!("Creating directory: {}", out_path.display());
            fs::create_dir_all(&out_path)?;
        } else {
            println!("Extracting file: {} ({} bytes)",
                     out_path.display(),
                     file.size());

            // Create and write the file
            let mut outfile = fs::File::create(&out_path)?;
            io::copy(&mut file, &mut outfile)?;

            // Set file permissions on Unix-like systems
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }

    println!("Extraction completed successfully!");
    Ok(())
}

// async fn extract_zip_async(zip_path: &str, extract_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let zip_path = zip_path.to_string();
//     let extract_dir = extract_dir.to_string();
//
//     tokio::task::spawn_blocking(move || {
//         extract_zip(&zip_path, &extract_dir)
//     }).await?
// }