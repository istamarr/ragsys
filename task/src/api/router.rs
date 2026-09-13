use std::collections::HashMap;
use std::sync::Arc;
use axum::{async_trait, routing::{get, post}, Router};
use axum::http::{HeaderValue, Method};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use log::debug;
use warp::{Filter};
// use crate::task::handler::key_handler::{reqKey, validateKey};
// use crate::task::handler::llm_handler::{generate_analysis_tag_handler, generate_llm};
// use crate::task::handler::user_handler::{admin_handler, user_handler};
// use crate::task::handler::embedd_handler::{quickThink, deepThink, quickThinkVoice, quickThinkAnimation, deepBlockChainThink, quickBlockChainThink};
use crate::domain::models::user::User;
// use crate::task::handler::command_helper::command_handler::call_to_run_script_execute_deploy;
// use crate::task::handler::query_handler::queryVec;
use crate::secure::auth::{with_auth, Role};
use crate::secure::error;
use crate::shared::helperUtils::current_time;
use super::health_checker_handler;
use crate::shared::secureUtils::*;
use crate::shared::sharedUtils::UrlConnect;
pub fn create_router(users: Arc<HashMap<String, User>>) -> warp::filters::BoxedFilter<(impl warp::Reply,)> {

    debug!("Create Router - Start {}", current_time());

    /**
    * Role && Credentials
    */

    let login_routes = warp::path!("login")
        .and(warp::post())
        .and(with_users(users.clone()))
        .and(warp::body::json())
        .and_then(login_handler);

    // let user_route = warp::path!("user")
    //     .and(with_auth(Role::User))
    //     .and_then(user_handler);

    // let admin_route = warp::path!("admin")
    //     .and(with_auth(Role::Admin))
    //     .and_then(admin_handler);

    /**
    * Utils
    */

    let healthchecker_route = warp::path!("api"/"healthchecker")
        .and_then(health_checker_handler);

    /**
    * LLm server
    */

    // let req_key = warp::path!("reqkey")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(reqKey);
    //
    // let req_key_validate = warp::path!("reqkey"/"validate")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(validateKey);
    //
    // let generate_llm = warp::path!("llm"/"createbuild")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(generate_llm);
    //
    // let llm_route = warp::path!("llm"/"llmpgd-1")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(generate_analysis_tag_handler);

    /**
    * Vectorizer
    */

    /**
    * Bert Server
    */

    /**
    * Req RAG MoE
    */
    // let llm_route = warp::path!("rag"/"quick")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(quickThink);
    //
    // let llm_route = warp::path!("rag"/"deep")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(deepThink);
    //
    // let llm_route = warp::path!("rag"/"quickThinkVoice")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(quickThinkVoice);
    //
    // let llm_route = warp::path!("rag"/"quickThinkAnimation")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(quickThinkAnimation);
    //
    // let llm_route = warp::path!("rag"/"deepBlockChainThink")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(deepBlockChainThink);
    //
    // let llm_route = warp::path!("rag"/"quickBlockChainThink")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(quickBlockChainThink);
    //
    // let llm_route = warp::path!("rag"/"queryVec")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(queryVec);

    /**
    * Scripts
    */
    // call_to_run_script
    // let lib_deploy = warp::path!("scripts_modules"/"lib"/"deploy")
    //     .and(warp::post())
    //     .and(with_auth(Role::User))
    //     .and(warp::body::json())
    //     .and_then(call_to_run_script_execute_deploy);


    let libServerOrigin: &str = &String::from(UrlConnect::LIB_SERVER.url.clone());
    debug!("Open Resource Lib Server {}",libServerOrigin.clone());
    let cors = warp::cors()
        .allow_origin(libServerOrigin.clone())
        .allow_methods(vec![Method::GET.as_str(), Method::POST.as_str(), Method::PATCH.as_str(), Method::DELETE.as_str()])
        .allow_credentials(true)
        .allow_headers(vec![AUTHORIZATION.to_string(), ACCEPT.to_string(), CONTENT_TYPE.to_string()]);
        // .allow_any_origin()
        // .allow_methods(vec!["GET", "POST", "DELETE"])
        // .allow_headers(vec!["Authorization", "Content-Type"]);

    debug!("Create Router - Succeed {}", current_time());

    login_routes
        .or(healthchecker_route)
        // .or(user_route)
        // .or(admin_route)
        // .or(llm_route)
        // .or(req_key)
        // .or(req_key_validate)
        // .or(generate_llm)
        .recover(error::handle_rejection)
        .with(cors)
        .boxed()

}

