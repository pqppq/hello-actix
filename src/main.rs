use std::sync::Mutex;

use actix_web::{get, guard, post, web, App, HttpResponse, HttpServer, Responder};

struct AppState {
    app_name: String,
}

struct AppStateWithCounter {
    counter: Mutex<i32>,
}

#[get("/hello")]
async fn hello(data: web::Data<AppState>) -> String {
    // HttpResponse::Ok().body("Hello world!")
    let app_name = &data.app_name;
    format!("Hello {app_name}!")
}

async fn hello_world() -> impl Responder {
    "Hello world!"
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}

#[get("/")]
async fn index(data: web::Data<AppStateWithCounter>) -> String {
    let mut counter = data.counter.lock().unwrap();
    *counter += 1;
    format!("Request number: {counter}")
}

fn scoped_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/test")
            .route(web::get().to(|| async { HttpResponse::Ok().body("test") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/app2")
            .route(web::get().to(|| async { HttpResponse::Ok().body("app2") }))
            .route(web::head().to(HttpResponse::MethodNotAllowed)),
    );
}

// marks async main function as the Actix Web system entry-point.
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let counter = web::Data::new(AppStateWithCounter {
        counter: Mutex::new(0),
    });
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                app_name: String::from("Actix web"),
            }))
            .app_data(counter.clone())
            // /app2
            .configure(config)
            // /api/test
            .service(web::scope("/api").configure(scoped_config))
            .service(index)
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            .service(
                // in /app scope(same url path prefix)
                web::scope("/app")
                    // create route /app/index.html
                    .route("/index.html", web::get().to(hello_world)),
            )
            .service(
                web::scope("/foo")
                    .guard(guard::Host("users.rust-lang.org"))
                    .route("/", web::get().to(hello_world)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
