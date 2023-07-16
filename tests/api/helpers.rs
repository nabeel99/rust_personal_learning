use linkify::LinkFinder;
use reqwest::Url;
use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    email_client::EmailClient,
    startup::get_pool_conn,
    telemetry::{get_subscriber, init_global_logger},
};
// use secrecy::ExposeSecret;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;

use wiremock::{matchers::header, Match, MockServer};
pub const LOCAL_HOST_WITH_RANDOM_PORT: &str = "127.0.0.1";

/// Confirmation links embedded in the request to the email API.
pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

//init logging
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info";
    //attached to each test
    let subscriber_name = "test";
    if let Ok(_) = std::env::var("TEST_LOG") {
        let subscriber = get_subscriber(&default_filter_level, &subscriber_name, std::io::stdout);
        init_global_logger(subscriber);
    } else {
        let subscriber = get_subscriber(&default_filter_level, &subscriber_name, std::io::sink);
        init_global_logger(subscriber);
    };
});
pub struct TestApp {
    pub address: String,
    pub pool_conn: PgPool,
    pub mock_server: MockServer,
    pub port_num: u16,
}
impl TestApp {
    pub async fn post_subscriptions(&self, test_body: String) -> reqwest::Response {
        let resp = reqwest::Client::new()
            .post(&dbg!(format!("{}/subscriptions", self.address)))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(test_body)
            .send()
            .await
            .expect("failed to execute request");
        resp
    }
    pub fn get_confirmation_links(&self, email_request: &wiremock::Request) -> ConfirmationLinks {
        let body: serde_json::Value = serde_json::from_slice(&email_request.body)
            .expect("deserialization of req body failed");
        // Extract the link from one of the request fields.
        let get_link = |s: &str| {
            let find_link = LinkFinder::new();
            let links: Vec<_> = find_link
                .links(s)
                .filter(|link_item| *link_item.kind() == linkify::LinkKind::Url)
                .collect();
            assert!(links.len() == 1);
            let raw_link = links[0].as_str().to_owned();

            let mut confirmation_link = Url::parse(&raw_link).expect("failed to parse url");
            // Let's make sure we don't call random APIs on the web
            assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
            // Let's rewrite the URL to include the port
            confirmation_link
                .set_port(Some(self.port_num))
                .expect("failed to set port");
            confirmation_link
        };
        let html = get_link(&body["HtmlBody"].as_str().expect("failed to deserialize"));
        let plain_text = get_link(&body["TextBody"].as_str().expect("failed to deserialize"));
        ConfirmationLinks{
            html,
            plain_text
        }
    
    }
}
#[test]
#[should_panic]
fn dummy_fail() {
    let result: Result<&str, &str> = Err("The app crashed due to an IO error");
    claims::assert_ok!(result);
}
//not needed but here for convenience
// async fn spawn_app().await {
//     // tokio::spawn(zero2prod::run().expect("failed"));
//     // todo!()
//     let x = zero2prod::run()
//         .expect("test")
//         .await
//         .expect("test2");
// }
//0 is a special port, os scans for available port and returns that port
//spawns the server on a background thread, so that server runs in parallel to the client handler thread
// The function is asynchronous now!
pub async fn spawn_app() -> TestApp {
    //evaluate the tracing function
    //1st way
    // *TRACING;
    //Forces the evaluation of this lazy value and returns a reference to the result. This is equivalent to the Deref impl, but is explicit.

    //2nd way
    // The first time `initialize` is invoked the code in `TRACING` is executed.
    // All other invocations will instead skip execution.
    Lazy::force(&TRACING);

    //reason to not have it here is logger should be set only once
    // but it will panic as all tests will call it , it is better to move it to lazy static
    // let subscriber = get_subscriber("debug", "testing_zero2prod");
    // init_global_logger(subscriber);

    //start mock server
    let email_server = MockServer::start().await;

    let settings = {
        let mut c = get_configuration().expect("failed to retrieve settings");
        c.db_settings.database_name = uuid::Uuid::new_v4().to_string();
        //use a random os port
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };
    //build does this onw
    //randomize new_test_db_name
    //instantiate email instance
    // let sender_email = settings
    //     .email_client
    //     .sender()
    //     .expect("invalid sender email");
    // let timeout = settings.email_client.timeout();
    // let email_client = EmailClient::new(
    //     settings.email_client.base_url,
    //     sender_email,
    //     settings.email_client.authorization_token,
    //     timeout
    // );

    //configure new database
    //process : create a random db name -> connect to an instance and create a database with the random name
    //-> connect to that random database-> run migrations on that random database
    //create a new test db-> run sqlx migrate
    // let pool_conn = configure_test_db(&settings.db_settings).await;
    configure_test_db(&settings.db_settings).await;
    //named future
    let server =
        zero2prod::startup::Application::build(settings.clone()).expect("failed to  bind listener");
    //spawning server on another future
    //so as to not block the main future as server future will never return
    let port_num = server.port();
    let _ = tokio::spawn(server.run_until_stopped());
    // let listner =
    //     TcpListener::bind(format!("{LOCAL_HOST_WITH_RANDOM_PORT}:0")).expect("bind failed");
    // let port_num = listner.local_addr().expect("socket addr failed").port();
    let address = format!("http://localhost:{}", port_num);

    //adding mock server
    TestApp {
        address: address,
        pool_conn: get_pool_conn(&settings.db_settings),
        mock_server: email_server,
        port_num,
    }
}
//process : create a random db name -> connect to an instance and create a database with the random name
//-> connect to that random database-> run migrations on that random database
//create a new test db-> run sqlx migrate
pub async fn configure_test_db(db_s: &DatabaseSettings) -> PgPool {
    //connection string with random db name to the postgres instance
    let conn_string_instance = db_s.connection_string_without_db();
    //create a new test db with name : random
    let mut conn = PgConnection::connect_with(&conn_string_instance)
        .await
        .expect("connection failed");
    conn.execute(format!(r#"CREATE DATABASE "{}";"#, db_s.database_name).as_str())
        .await
        .expect("failed to create db");
    //connect to that new db
    let new_db_conn_string = dbg!(db_s.connection_string());
    //run sqlx migration
    let pg_pool_conn = PgPool::connect_with(new_db_conn_string)
        .await
        .expect("connection failed");
    //run migrations in code
    sqlx::migrate!("./migrations")
        .run(&pg_pool_conn)
        .await
        .expect("migration failed");

    pg_pool_conn
}
