use axum::extract::{Path, Query};
use axum::http::{Method, Uri};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{middleware, Json, Router};
use ctx::Ctx;
use log::log_request;
use model::ModelController;
use serde::Deserialize;
use serde_json::json;
use tower_cookies::CookieManagerLayer;
use tower_http::services::ServeDir;
use uuid::Uuid;

pub use self::error::{Error, Result};

mod error;
mod web;
mod model;
mod ctx;
mod log;

#[tokio::main]
async fn main() -> Result<()> {

  //init model controller
  let mc = ModelController::new().await?;

  // here we apply the middleware to this routes so that 
  let routes_apis = web::routes_tickets::routes(mc.clone()).route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

  // build our application with a single route
  // cargo watch -q -c -w src/ -x run
  let routes_all = Router::new()
    .merge(routes_hello())
    .merge(web::routes_login::routes())
    .nest("/api", routes_apis)
    .layer(middleware::map_response(main_response_mapper))
    .layer(middleware::from_fn_with_state(
      mc.clone(),
      web::mw_auth::mw_ctx_resolver,
    ))
    .layer(CookieManagerLayer::new())
    .fallback_service(routes_static());
  // server
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  axum::serve(listener, routes_all).await.unwrap();
  // end of server
  Ok(())
}

// to map our responses in the right way
async fn main_response_mapper(
  ctx: Option<Ctx>,
  uri: Uri,
  req_method: Method,
  res: Response
) -> Response {
  println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
  let uuid = Uuid::new_v4();

  // -- Get the eventual error response
  let service_error = res.extensions().get::<Error>();
  let client_status_error = service_error.map(|se| se.client_status_and_error());

  // -- if we have a client error we build a new res
  let error_response = client_status_error
    .as_ref()
    .map(|(status_code, client_error)| {
    let client_error_body = json!({
      "error": {
        "type": client_error.as_ref(),
        "req_uuid": uuid.to_string(),
      }
    });

  println!("  ->> client_error_body: {client_error_body}");

    // Build the new response from the client_error_body
    // deref the status code
  (*status_code, Json(client_error_body)).into_response()
  });
  
  //server log implementation
  let client_error = client_status_error.unzip().1;
  let _ = 
    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

  println!(); //an empty line to separate our responses
  error_response.unwrap_or(res)
}
// static file routing and remember they are done
// as fallbacks
fn routes_static() -> Router {
  Router::new().nest_service("/", ServeDir::new("./"))
}
//merging the routes
fn routes_hello() -> Router {
  Router::new()
    .route("/hello", get(handler_hello))
    .route("/hello2/:name", get(handler_hello2)) 
}

// handler hello
#[derive(Debug, Deserialize)]
struct  HelloParams {
  name: Option<String>,
}

//e.g `/hello?name=frexie` ie params
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
  println!("->> {:<12} -handler_hello - {params:?}", "HANDLER");
  let name = params.name.as_deref().unwrap_or("World!");
  Html(format!("Hello <strong>{name}</strong>"))
}

// e.g `/hello2/Vee` i.e path
async fn handler_hello2(Path(name): Path<String>) -> impl IntoResponse {
  println!("->> {:<12} -handler_hello - {name:?}", "HANDLER");
  
  Html(format!("Hello <strong>{name}</strong>"))
}