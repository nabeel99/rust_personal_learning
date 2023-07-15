use crate::{
    greet::greet,
    routes::{check_health, subscribe},
};
use actix_web::{middleware::Logger, guard::Trace};
use actix_web::{dev::Server, guard, web, App, HttpServer, Route};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

///server owns a dynamic owned boxFuture which when awiated returns a std::io::Result<()>
/// that is why its return value is acceptable
/// reason for future not being send and giving used across await error
/// weird compiler bug, although for HttpServer if type B or S is not send then
/// http server is not send, since S is of Type App which is impl !Send the future is not send
/// but since we are now using it afte rthe await call it should work fine
/// compiler in a previous iteration for some weird reason wanted to drop it after await which is what caused the error
pub fn run(
    listner: TcpListener,
    connection: PgPool,
) -> std::result::Result<Server, std::io::Error> {
    let wrapped_connection = web::Data::new(connection);
    let srv = HttpServer::new(move || {
        App::new()
        //logger is not tracing aware
            // .wrap(Logger::default())
            //solution use a tracing aware logger
            .wrap(TracingLogger::default())
            .route("/", web::get().to(greet))
            .route(
                "/health_check",
                //web::get is a macro for the below verbose syntx
                Route::new().guard(guard::Get()).to(check_health),
            )
            .route("/{name}", web::get().to(greet))
            .route(
                "/subscriptions",
                Route::new().guard(guard::Post()).to(subscribe),
            )
            .app_data(wrapped_connection.clone())
    })
    .listen(listner)?
    .run();
    Ok(srv)
}
