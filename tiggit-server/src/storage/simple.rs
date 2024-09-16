use axum::{
    extract::{Path, Query},
    routing::{get, post, method_routing::MethodRouter},
    Router,
    response::IntoResponse,
    body::Bytes,
    http::{header, HeaderMap, StatusCode}
};
use tokio::{
    process::Command,
    io::AsyncWriteExt
};
use std::{
    process::Stdio,
    env,
};
use serde::Deserialize;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use super::utils::get_repo_name;

// Structs
#[derive(Deserialize)]
struct InfoRefQueryParam {
    service: String,
}

// Handler implementations
async fn _git_service_handler(Path((user, repo, service_name)): Path<(String, String, String)>, body: Bytes) -> impl IntoResponse {
    match service_name.as_str() {
        "git-receive-pack" | "git-upload-pack" => (),
        // implement a 404
        _ => return (StatusCode::NOT_FOUND, HeaderMap::new(), b"Not found".to_vec()),
    };

    let repository = get_repo_name(repo).unwrap();
    let storage_dir = env::var("SIMPLE_STORAGE_DIR").expect("SIMPLE_STORAGE_DIR not defined");

    let mut command = Command::new("git")
        .arg(&service_name[4..])
        .arg("--stateless-rpc")
        .arg(format!("./{}/{}/{}", storage_dir, user, repository))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .kill_on_drop(true)
        .spawn().unwrap();

    command.stdin.as_mut().unwrap().write_all(&body).await.unwrap();

    let output = command.wait_with_output().await.unwrap();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        format!("application/x-git-{}-result",&service_name.as_str()[4..]).parse().unwrap(),
    );

    (StatusCode::OK ,headers, output.stdout)

}

async fn _git_info_refs_handler(
        Path((user, repo)): Path<(String, String)>,
        headers: HeaderMap,
        q: Query<InfoRefQueryParam>
    ) -> impl IntoResponse {
    let InfoRefQueryParam { service } = q.0;
    let service_name = &service.to_string()[4..];  // strip the 'git-' prefix
    let mut hex_length = String::new();
    match service_name {
        "receive-pack" => hex_length.push_str("001f"),
        "upload-pack" => hex_length.push_str("001e"),
        // implement a 404
        _ => return (StatusCode::NOT_FOUND, HeaderMap::new(), "Not found".to_string()),
    };

    if !headers.contains_key("authorization") {
        let mut challenge_headers = HeaderMap::new();
        challenge_headers.insert(header::WWW_AUTHENTICATE, "Basic realm=\"Git Server\"".parse().unwrap());
        return (StatusCode::UNAUTHORIZED, challenge_headers, "Authentication required".to_string());
    }
    let encoded_token = headers["authorization"].clone().to_str().unwrap().to_string();

    if !encoded_token.starts_with("Basic") {
        // Making sure that Basic auth is followed
        let mut challenge_headers = HeaderMap::new();
        challenge_headers.insert(header::WWW_AUTHENTICATE, "Basic realm=\"Git Server\"".parse().unwrap());
        return (StatusCode::UNAUTHORIZED, challenge_headers, "Authentication required".to_string());
    }
    let encoded_part = &encoded_token[6..];
    let decoded_token_vector = &STANDARD.decode(encoded_part).unwrap();
    let token = String::from_utf8_lossy(decoded_token_vector);
    let mut split_creds = token.split(':');
    let username = split_creds.next().unwrap();
    let password = split_creds.next().unwrap();
    let repository = get_repo_name(repo).unwrap();
    let storage_dir = env::var("SIMPLE_STORAGE_DIR").expect("SIMPLE_STORAGE_DIR not defined");
    println!("{} {} {} {}", username, password, user, repository);

    let command = Command::new("git")
        .arg(service_name)
        .arg("--stateless-rpc")
        .arg("--advertise-refs")
        .arg(format!("./{}/{}/{}", storage_dir, user, repository))
        .output()
        .await
        .unwrap();

    let stdout = String::from_utf8_lossy(&command.stdout);

    let response_content = format!("# service=git-{}\n0000", service_name);

    let response = format!("{}{}{}", hex_length, response_content, stdout);

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        format!("application/x-git-{}-advertisement", service_name).parse().unwrap(),
    );

    (StatusCode::OK, headers, response)
}

// Public Handlers
pub fn git_service_handler() -> MethodRouter {
    return post(_git_service_handler);
}

pub fn git_info_refs_handler() -> MethodRouter {
    return get(_git_info_refs_handler)
}

// Router
pub fn git_storage_router() -> Router {
    return Router::new()
        .route("/:user/:repo/:service_name", git_service_handler())
        .route("/:user/:repo/info/refs", git_info_refs_handler())
    ;
}