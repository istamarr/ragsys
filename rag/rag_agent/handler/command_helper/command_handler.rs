use warp::Reply;
use crate::domain::models::native::RequestBody;
use crate::scripts_modules::command::shellexec::cmd_shell_deploy_execute;
use crate::WebResult;

fn main() {
    println!("command handler");
}
//munculin semua helper command di scripts_modules ini!!!

pub async fn call_to_run_script_execute_deploy(uid : String, body : RequestBody) -> WebResult<impl Reply>{
    println!("command handler run deploy lib to artifactory");
    cmd_shell_deploy_execute();
    Ok(format!("Call:\n{}\n - Run Script Result:\n{}\n - Deploy At:\n{}\n ", "".to_string(), "".to_string(), "".to_string()))
}