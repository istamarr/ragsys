use log::info;

/**
* Lib Internal Test (api and abstract code) And Uploader to Artifactory
* Changes In Lib based on
* 1. llm update -> done internal tested
* 2. llm if any changes -> done internal tested
* 3. upload -> notify by email
*/

pub fn upload_wasm_server() -> String {
    let log = "Uploading wasm server artifactory";
    // let my_logger = "";
    // info!(logger: my_logger,"{} # Done In Ai Model Server",log.clone());

    info!("#######################################");
    info!("{} # 'Ai Model Server' to Upload ",log.clone());
    info!("#######################################");
    info!("   Flow WASM      # WASM ----> Srv RAG ----> Pipeline RAG");
    info!("   Test And Devel # 'Ai Model Server'");

    "Uploaded".to_string()
}