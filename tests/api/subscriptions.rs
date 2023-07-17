use crate::helpers::spawn_app;
use linkify::LinkFinder;
use wiremock::{
    matchers::{header, method, path},
    Match, Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    // Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    // Sabotage the database
    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&app.pool_conn)
        .await
        .unwrap();
    // Act
    let response = app.post_subscriptions(body.into()).await;
    // Assert
    assert_eq!(response.status().as_u16(), 500);
}

#[tokio::test]
async fn subscribe_sends_a_valid_link() {
    dotenvy::dotenv().expect("failed to load env parameters");
    //no need for this now as pool conn is part of testapp struct
    // let configuration = get_configuration().expect("failed to get db settings");
    // let connection_string = dbg!(configuration.db_settings.connection_string());
    // let mut connection = PgConnection::connect(&connection_string)
    //     .await
    //     .expect("connection failed");
    let app = spawn_app().await;
    //mock settings to mount on top of a wiremock server

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        //not adding expectation any more
        .mount(&app.mock_server)
        .await;
    let client = reqwest::Client::new();
    let test_body = "name=Nabeel%20Naveed&email=ac3r_nabeel%40live.com";
    //ACT
    let response = app.post_subscriptions(test_body.into()).await;
    let email_request = &app
        .mock_server
        .received_requests()
        .await
        .expect("failed to get received requests")[0];
    let confirmation_links = app.get_confirmation_links(&email_request);
    assert_eq!(confirmation_links.html, confirmation_links.plain_text);

    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.pool_conn)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ac3r_nabeel@live.com");
    assert_eq!(saved.name, "Nabeel Naveed");
}

#[tokio::test]
// #[should_panic]
async fn subscribe_returns_a_200_when_fields_are_present_but_empty() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=ursula_le_guin%40gmail.com", "empty name"),
        ("name=Ursula&email=", "empty email"),
        ("name=Ursula&email=definitely-not-an-email", "invalid email"),
    ];
    //mock settings to mount on top of a wiremock server

    for (body, description) in test_cases {
        //ACT
        let response = app.post_subscriptions(body.into()).await;
        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 Bad Request, when the payload was {}.",
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
    //mock settings to mount on top of a wiremock server

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.mock_server)
        .await;
    let client = reqwest::Client::new();
    let test_body = "name=Nabeel%20Naveed&email=ac3r_nabeel%40live.com";
    let response = app.post_subscriptions(test_body.into()).await;
    assert_eq!(200, response.status().as_u16());
}
#[tokio::test]
async fn subscribe_check_persistence() {
    dotenvy::dotenv().expect("failed to load env parameters");
    //no need for this now as pool conn is part of testapp struct
    // let configuration = get_configuration().expect("failed to get db settings");
    // let connection_string = dbg!(configuration.db_settings.connection_string());
    // let mut connection = PgConnection::connect(&connection_string)
    //     .await
    //     .expect("connection failed");
    let app = spawn_app().await;
    //mock settings to mount on top of a wiremock server

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.mock_server)
        .await;
    let client = reqwest::Client::new();
    let test_body = "name=Nabeel%20Naveed&email=ac3r_nabeel%40live.com";
    let response = app.post_subscriptions(test_body.into()).await;
    assert_eq!(200, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name,status FROM subscriptions",)
        .fetch_one(&app.pool_conn)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ac3r_nabeel@live.com");
    assert_eq!(saved.name, "Nabeel Naveed");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn test_html_form_subscribe_data_api_200_response() -> Result<(), std::io::Error> {
    let app = spawn_app().await;
    //% encoded for non-alphanumeric, urlencoded encoding algorithm used to encode html data
    //no need for {}
    let test_body = "name=Nabeel%20Naveed&email=ac3r_nabeel%40live.com";
    let client = reqwest::Client::new();
    //mock settings to mount on top of a wiremock server

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.mock_server)
        .await;
    let response = app.post_subscriptions(test_body.into()).await;
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
    for (payload, err) in test_body {
        let response = app.post_subscriptions(payload.into()).await;
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
