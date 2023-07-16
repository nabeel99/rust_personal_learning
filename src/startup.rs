use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    greet::greet,
    routes::{check_health, subscribe,confirm},
};
// use actix_web::{middleware::Logger, guard::Trace};
use actix_web::{dev::Server, guard, web, App, HttpServer, Route};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

// We need to define a wrapper type in order to retrieve the URL
// in the `subscribe` handler.
// Retrieval from the context, in actix-web, is type-based: using
// a raw `String` would expose us to conflicts.
pub struct ApplicationBaseUrl(pub String);
///server owns a dynamic owned boxFuture which when awiated returns a std::io::Result<()>
/// that is why its return value is acceptable
/// reason for future not being send and giving used across await error
/// weird compiler bug, although for HttpServer if type B or S is not send then
/// http server is not send, since S is of Type App which is impl !Send the future is not send
/// but since we are now using it afte rthe await call it should work fine
/// compiler in a previous iteration for some weird reason wanted to drop it after await which is what caused the error
///
///
/// 
pub fn run(
    listner: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
    base_url : String
) -> std::result::Result<Server, std::io::Error> {
    let wrapped_connection = web::Data::new(connection);
    let wrapped_email_client = web::Data::new(email_client);
    let wrapped_base_url = web::Data::new(ApplicationBaseUrl(base_url));
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
            .route("/subscriptions/confirm",Route::new().guard(guard::Get()).to(confirm))
            .app_data(wrapped_connection.clone())
            .app_data(wrapped_email_client.clone())
            .app_data(wrapped_base_url.clone())
    })
    .listen(listner)?
    .run();
    Ok(srv)
}
impl Application {
    pub fn build(settings: Settings) -> Result<Application, std::io::Error> {
        let sender_email = settings
            .email_client
            .sender()
            .expect("invalid sender email");
        let timeout = settings.email_client.timeout();
        let email_client = EmailClient::new(
            settings.email_client.base_url,
            sender_email,
            settings.email_client.authorization_token,
            timeout,
        );
        let address = format!(
            "{}:{}",
            settings.application.host, settings.application.port
        );
        // dbg!("Listening on this Address : -> {} ",&address);
        println!("Listening on this Address : -> {}", address);
        dbg!(&address);
        let new_listener = TcpListener::bind(address)?;
        let port_num = new_listener.local_addr()?.port();
        let connection = get_pool_conn(&settings.db_settings);
        // .expect("failed to establish");
        //Build an email client using settings
        //fetch sender_email and parse it to SubScriber Email domain type (which encoded invariants aroudn email format in its name)

        let server = run(new_listener, connection, email_client,
            //confirmation email domain
            settings.application.base_url)?;
        Ok(Self {
            server,
            port: port_num,
        })
    }
    pub fn port(&self) -> u16 {
        self.port
    }
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_pool_conn(settings: &DatabaseSettings) -> PgPool {
    let conn_string = settings.connection_string();
    //set connection acquite to 2 seconds using PgOptions
    //connect_lazy_with isnt async so no need to await it
    let connection = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(conn_string);
    connection
}

pub struct Application {
    server: Server,
    port: u16,
}
