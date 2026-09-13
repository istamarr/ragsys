use std::path::Path;
use anyhow::Result;

pub use crate::handler::flow_code::templates::rust::*;
pub use crate::handler::flow_code::templates::python::*;
pub use crate::handler::flow_code::templates::node::*;
pub use crate::handler::flow_code::templates::react::*;
pub use crate::handler::flow_code::templates::fullstack::*;

pub async fn generate_boilerplate(template: &str, name: &str, output_dir: &Path) -> Result<()> {
    match template {
        "rust-axum" => generate_rust_axum_boilerplate(output_dir, name).await,
        "rust-actix" => generate_rust_actix_boilerplate(output_dir, name).await,
        "rust-advanced" => generate_rust_advanced_boilerplate(output_dir, name).await,
        "python-fastapi" => generate_python_fastapi_boilerplate(output_dir, name).await,
        "python-advanced" => generate_python_advanced_boilerplate(output_dir, name).await,
        "node-express" => generate_node_express_boilerplate(output_dir, name).await,
        "node-advanced" => generate_node_advanced_boilerplate(output_dir, name).await,
        "react-app" => generate_react_app_boilerplate(output_dir, name).await,
        // "fullstack-rust-react" => generate_fullstack_rust_react_boilerplate(output_dir, name).await,
        _ => Err(anyhow::anyhow!("Unknown template: {}", template)),
    }
}

pub fn get_available_templates() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        // (template_name, description, category)
        ("rust-axum", "Rust web server with Axum framework", "basic"),
        ("rust-actix", "Rust web server with Actix-web framework", "basic"),
        ("rust-advanced", "Modular Rust project with nested structure", "advanced"),
        ("python-fastapi", "Basic Python API with FastAPI framework", "basic"),
        ("python-advanced", "Python package with organization", "advanced"),
        ("node-express", "Node.js server with Express framework", "basic"),
        ("node-advanced", "Node.js with full structure", "advanced"),
        ("react-app", "React application with basic setup", "basic"),
        // ("fullstack-rust-react", "Complete Rust + React", "fullstack"),trial
        //go
        //java
    ]
}
