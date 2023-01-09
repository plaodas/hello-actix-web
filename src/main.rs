use actix_web::{get, post, web, App, HttpRequest,HttpResponse, HttpServer, Responder, Result};


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


async fn index() -> impl Responder {
    "Hello app/index.html"
}
async fn index2() -> impl Responder {
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


#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Note: web::Data created _outside_ HttpServer::new closure
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });

    HttpServer::new(move || {
        App::new()

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

        .app_data(counter.clone()) // <- register the created data
        .route("/mutable_state", web::get().to(mutable_state))

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
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}