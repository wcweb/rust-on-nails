mod config;
mod errors;
mod authentification;

use crate::api_service::UsersService;
use crate::errors::CustomError;
use crate::authentification::Authentication;

use axum::extract::{Extension, Path};
use axum::{response::Html, response::IntoResponse, response::Json, routing::get, Router};
// use deadpool_postgres::Pool;
use std::net::SocketAddr;
use axum::body::{self, Body, Empty};
use axum::http::{header, HeaderValue, Response, StatusCode};
use assets::templates::statics::StaticFile;

use self::multiplex_service::MultiplexService;
use db::User;
// use tower::{make::Shared, steer::Steer, BoxError, ServiceExt};
// use tonic::transport::Server;
use grpc_api::api::users_server::UsersServer;
// use http::{header::CONTENT_TYPE, Request};
mod multiplex_service;
mod api_service;

#[macro_use]
mod axum_ructe;

#[tokio::main]
async fn main() {
    let config = config::Config::new();

    let pool = db::create_pool(&config.database_url);

    // build our application with a route
    let app = Router::new()
        .route("/", get(users))
        .route("/home", get(home_page))
        .route("/static/*path", get(static_path))
        .layer(Extension(config))
        .layer(Extension(pool.clone()));
        // .boxed_clone();

    // run it
    // let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    // println!("listening on {}", addr);
    // axum::Server::bind(&addr)
    //     .serve(app.into_make_service())
    //     .await.unwrap();

    // build the grpc service
    //  let reflection_service = tonic_reflection::server::Builder::configure()
    //     .register_encoded_file_descriptor_set(grpc_api::api)
    //     .build()
    //     .unwrap();

    let grpc = tonic::transport::Server::builder()
        // .add_service(reflection_service)
        .add_service(tonic_web::enable(UsersServer::new(UsersService { pool })))
        .into_service();

    // combine them into one service
    let service = MultiplexService::new(app, grpc);

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)        
        .serve(tower::make::Shared::new(service))
        .await.unwrap();
}

// async fn users(Extension(pool): Extension<db::Pool>) -> Result<Json<Vec<User>>, CustomError> {
//     let client = pool.get().await?;

//     let users = db::queries::users::get_users()
//         .bind(&client)
//         .all()
//         .await?;

//     Ok(Json(users))
// }

async fn users(Extension(pool): Extension<db::Pool>, current_user: Authentication) -> Result<Html<String>, CustomError> {
    let client = pool.get().await?;
    dbg!(current_user.user_id);
    let users = db::queries::users::get_users()
        .bind(&client)
        .all()
        .await?;

     // We now return HTML
     Ok(Html(ui_components::users::users(
        users,
    )))
}

async fn home_page() -> impl IntoResponse {
    render!(
        assets::templates::page_html,
        &[("first", 3), ("second", 7), ("third", 2)]
    )
}

async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    if let Some(data) = StaticFile::get(path) {
        Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(data.mime.as_ref()).unwrap(),
            )
            .body(body::boxed(Body::from(data.content)))
            .unwrap()
    } else {
        Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::boxed(Empty::new()))
            .unwrap()
    }
}