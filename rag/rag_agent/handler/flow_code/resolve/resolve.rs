use std::path::Path;
use crate::rag_agent::handler::flow_code::templates::template_utils::{format_project_name, write_file};

/**Dependency Resolve**/
pub fn resolve_gen_flow_code(output_dir: &Path, project_name: &str, lang: &str) -> anyhow::Result<()> {
    //If-Else using lang
    resolve_rust(output_dir, project_name).expect("resolve_gen_flow_code");
    Ok(())
}

pub fn resolve_rust(output_dir: &Path, project_name: &str) -> anyhow::Result<()> {
    let (snake_name, kebab_name, _) = format_project_name(project_name);
    let project_dir = output_dir.join(&kebab_name);

    //Call LLM

    // write_file(&project_dir.join("README.md"), &readme)?;

    Ok(())
}

// /**Fixing n Correcting**/