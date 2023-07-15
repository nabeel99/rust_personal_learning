use zero2prod::{
    configuration::{get_configuration, DatabaseSettings},
    telemetry::{get_subscriber, init_global_logger},
};
// use secrecy::ExposeSecret;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
pub const LOCAL_HOST_WITH_RANDOM_PORT: &str = "127.0.0.1";

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
    address: String,
    pool_conn: PgPool,
}

#[tokio::test]
#[should_panic]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(&dbg!(format!("{}/subscriptions", app.address)))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("failed to execute request");
        assert_eq!(
            200,
            response.status().as_u16(),
            "The API did not return a 200 OK when the payload was {}.",
            description
        );
        // let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        //     .fetch_one(&app.pool_conn)
        //     .await
        //     .expect("Failed to fetch saved subscription.");
        // assert_eq!(saved.email, "ac3r_nabeel@live.com");
        // assert_eq!(saved.name, "Nabeel Naveed");
    }
}
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    dotenvy::dotenv().expect("failed to load env parameters");
    //no need for this now as pool conn is part of testapp struct
    // let configuration = get_configuration().expect("failed to get db settings");
    // let connection_string = dbg!(configuration.db_settings.connection_string());
    // let mut connection = PgConnection::connect(&connection_string)
    //     .await
    //     .expect("connection failed");
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_body = "name=Nabeel%20Naveed&email=ac3r_nabeel%40live.com";
    let response = client
        .post(&dbg!(format!("{}/subscriptions", app.address)))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(test_body)
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.pool_conn)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ac3r_nabeel@live.com");
    assert_eq!(saved.name, "Nabeel Naveed");
}

#[tokio::test]
async fn test_health_api() -> Result<(), std::io::Error> {
    // tokio::spawn(zero2prod::run().expect("failed"));
    //     tokio::spawn(async {
    //         spawn_app().await.await

    // });

    let app = spawn_app().await;
    dbg!("Hello");

    let client = reqwest::Client::new();
    let response = client
        .get(&dbg!(format!("{}/health_check", { &app.address })))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
    Ok(())
}

#[tokio::test]
async fn test_html_form_subscribe_data_api_200_response() -> Result<(), std::io::Error> {
    let app = spawn_app().await;
    //% encoded for non-alphanumeric, urlencoded encoding algorithm used to encode html data
    //no need for {}
    let test_body = "name=Nabeel%20Naveed&email=ac3r_nabeel%40live.com";
    let client = reqwest::Client::new();
    let response = client
        .post(&dbg!(format!("{}/subscriptions", app.address)))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(test_body)
        .send()
        .await
        .expect("failed to execute request");
    assert_eq!(200, response.status().as_u16());
    Ok(())
}
#[tokio::test]
async fn test_html_form_subscribe_data_api_400_response() -> Result<(), std::io::Error> {
    let app = spawn_app().await;
    //% encoded for non-alphanumeric, urlencoded encoding algorithm used to encode html data
    let test_body = vec![
        ("email=ac3r_nabeel%40live.com", "missing name"),
        ("name=Nabeel%20Naveed", "missing email"),
        ("", "no data"),
    ];
    let client = reqwest::Client::new();
    for (payload, err) in test_body {
        let response = client
            .post(&dbg!(format!("{}/subscriptions", &app.address)))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(payload)
            .send()
            .await
            .expect("failed to execute request");
        assert_eq!(
            400,
            response.status().as_u16(),
            "The Api did not fail with 400 Bad Request , when the invalid payoad was
         {}",
            err
        );
    }

    Ok(())
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
async fn spawn_app() -> TestApp {
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
    let listner =
        TcpListener::bind(format!("{LOCAL_HOST_WITH_RANDOM_PORT}:0")).expect("bind failed");
    let port_num = listner.local_addr().expect("socket addr failed").port();
    let mut settings = get_configuration().expect("failed to retrieve settings");
    //randomize new_test_db_name
    settings.db_settings.database_name = uuid::Uuid::new_v4().to_string();
    //spawning server on another future
    //so as to not block the main future as server future will never return

    //configure new database
    //process : create a random db name -> connect to an instance and create a database with the random name
    //-> connect to that random database-> run migrations on that random database
    //create a new test db-> run sqlx migrate
    let pool_conn = configure_test_db(&settings.db_settings).await;
    //named future
    let server =
        zero2prod::startup::run(listner, pool_conn.clone()).expect(" failed to bind a listener");
    let _ = tokio::spawn(server);

    let address = format!("http://{LOCAL_HOST_WITH_RANDOM_PORT}:{}", port_num);
    TestApp {
        address: address,
        pool_conn: pool_conn,
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
    sqlx::migrate!("./migrations")
        .run(&pg_pool_conn)
        .await
        .expect("migration failed");

    pg_pool_conn
}
