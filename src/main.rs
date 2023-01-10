use actix_web::{
    guard,
    body::BoxBody, 
    http::{header::ContentType, StatusCode}, 
    get, post, 
    web, App, 
    HttpRequest,
    HttpResponse, 
    HttpServer, 
    Responder, 
    Result, 
    error,
    middleware::Logger, 
};


#[get("/")]
async fn hello() -> impl Responder{
    HttpResponse::Ok().body("Hello World!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder{
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder{
    HttpResponse::Ok().body("Hey there!")
}


async fn index( ) -> impl Responder {
    "Hello app/index.html"
}
async fn index2( ) -> impl Responder {
    "Hello app/index"
}

#[get("/show/{id}")]
async fn show_id(path: web::Path<(u32,)>) -> HttpResponse {
    HttpResponse::Ok().body(format!("show_id: {}", path.into_inner().0))
}


#[get("/test/{v1}/{v2}/")]
async fn index3(req: HttpRequest) -> Result<String> {
    let v1: String = req.match_info().get("v1").unwrap().parse().unwrap();
    let v2: String = req.match_info().query("v2").parse().unwrap();
    let (v3, v4):( String, String) = req.match_info().load().unwrap();
    Ok(format!("Test values {} {} {} {}", v1, v2, v3, v4))
}


#[get("/{username}/{id}/index.html")] // <- define path parameters
async fn path_info_ext(info: web::Path<(String, u32)>) -> Result<String> {
    let info = info.into_inner();
    Ok(format!("Welcome {}! id: {}", info.0, info.1))
}


// This struct represents state
struct AppState {
    app_name: String,
}

#[get("/app_name")]
async fn app_name(data: web::Data<AppState>) -> String {
    let app_name = &data.app_name; // <- get app_name
    format!("Hello {app_name}!") // <- response with app_name
}

use std::sync::Mutex;
struct AppStateWithCounter {
    counter: Mutex<i32>, // <- Mutex is necessary to mutate safely across threads
}

async fn mutable_state(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap(); // <- get counter's MutexGuard
    *counter += 1; // <- access counter inside MutexGuard

    format!("Request number: {counter}") // <- response with count
}

// this function could be located in a different module
fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/testtest")
            .route(web::get().to(|| async { HttpResponse::Ok().body("testtest") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

// this function could be located in a different module
fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/appapp")
            .route(web::get().to(|| async { HttpResponse::Ok().body("appapp") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}


//===== read JSON 
use serde::Deserialize;

#[derive(Deserialize)]
struct Info {
    username: String,
}

/// deserialize `Info` from request's body, max payload size is 4kb
async fn json_info(info: web::Json<Info>) -> impl Responder {
    format!("Welcome {}!", info.username)
}


//===== return JSON
use serde::Serialize;

#[derive(Serialize)]
struct MyObj {
    name: &'static str,
}

// Responder
impl Responder for MyObj {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(body)
    }
}

async fn return_json() -> impl Responder {
    MyObj { name: "user" }
}


//===== ERROR =====
use derive_more::{Display, Error};
#[derive(Debug, Display, Error)]
#[display(fmt = "my error: {}", name)]
struct MyErrorStruct {
    name: &'static str,
}

// Use default implementation for `error_response()` method
impl error::ResponseError for MyErrorStruct {}

async fn index_error() -> Result<&'static str, MyErrorStruct> {
    Err(MyErrorStruct { name: "test" })
}

#[derive(Debug, Display, Error)]
enum MyError {
    #[display(fmt = "internal error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "timeout")]
    Timeout,
}

impl error::ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            MyError::BadClientData => StatusCode::BAD_REQUEST,
            MyError::Timeout => StatusCode::GATEWAY_TIMEOUT,
        }
    }
}

#[get("/400")]
async fn index_error_400() -> Result<&'static str, MyError> {
    Err(MyError::BadClientData)
}

#[get("/408")]
async fn index_error_408() -> Result<&'static str, MyError> {
    Err(MyError::Timeout)
}

#[get("/500")]
async fn index_error_500() -> Result<&'static str, MyError> {
    Err(MyError::InternalError)
}


//====== LOG REPORT ======
use log::info;

#[derive(Debug, Display, Error)]
#[display(fmt = "my error: {}", name)]
pub struct MyErrorLog {
    name: &'static str,
}

// Use default implementation for `error_response()` method
impl error::ResponseError for MyErrorLog {}

#[get("/")]
async fn index_err_log() -> Result<&'static str, MyErrorLog> {
    let err = MyErrorLog { name: "test error" };
    info!("{}", err);
    Err(err)
}

#[rustfmt::skip]
#[actix_web::main]
async fn main() -> std::io::Result<()> {

    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();
    
    
    // Note: web::Data created _outside_ HttpServer::new closure
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    let json_config = web::JsonConfig::default()
    .limit(4096)
    .error_handler(|err, _req| {
        // create custom error response
        error::InternalError::from_response(err, HttpResponse::Conflict().finish())
            .into()
    });



    HttpServer::new(move || {
        let logger = Logger::default();

        App::new()
        .wrap(logger)
        .configure(config)
        .service(web::scope("/api").configure(scoped_config))
        .route(
            "/root",
            web::get().to(|| async { HttpResponse::Ok().body("/root") }),
        )

        .app_data(web::Data::new(AppState {
            app_name: String::from("Actix Web"),
        }))
        .service(app_name)

        .service(
            web::resource("/mutable_state")
                .app_data(counter.clone()) // <- register the created data
                .route( web::get().to(mutable_state)),
        )

        .service(
            web::resource("/json_info")
                // change json extractor configuration
                .app_data(json_config.clone())
                .route(web::post().to(json_info)),
        )

        .service(
            web::resource("/return_json")
                .route(web::get().to(return_json)),            
        )

        .service(
            web::resource("/index_error")
            .route(web::get().to(index_error)),            
        )
        .service(index_error_400)
        .service(index_error_408)
        .service(index_error_500)

        .service(
            web::scope("/app")
            .route("index.html", web::get().to(index))
            .route("/", web::get().to(index2))
            .route("",web::get().to(index2)),
        )
        .service(hello)
        .service(echo)
        .service(index3)
        .service(show_id)
        .service(path_info_ext)
        .route("/hey", web::get().to(manual_hello))
        .default_service(
            web::route()
                .guard(guard::Not(guard::Get()))
                .to(HttpResponse::MethodNotAllowed),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}


#[cfg(test)]
mod tests {
    use actix_web::{http::header::ContentType, test, web, App};

    use super::*;

    #[actix_web::test]
    async fn test_index_get() {
        let app = test::init_service(App::new().route("/", web::get().to(index))).await;
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_index_post() {
        let app = test::init_service(App::new().route("/", web::get().to(index))).await;
        let req = test::TestRequest::post().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }
}