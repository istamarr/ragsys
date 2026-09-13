use std::path::Path;
use crate::rag_agent::handler::flow_code::resolve::resolve::resolve_gen_flow_code;

pub async fn dependency_resolve(template: &str, name: &str, output_dir: &Path) -> anyhow::Result<()> {
    match template {
        "flow-code" => resolve_gen_flow_code(output_dir, name, ""),
        // "fullstack-rust-react" => resolve_fullstack_rust_react_boilerplate(output_dir, name).await,
        _ => Err(anyhow::anyhow!("Unknown template: {}", template)),
    }
}

// # Add String...Example...
// $newContent += ""
// $newContent += "[profile.release]"
// $newContent += "opt-level = 3"
// $newContent += "lto = true"
// $newContent += "codegen-units = 1"
// Call Pin's LM

