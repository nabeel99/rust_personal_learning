use crate::helpers::spawn_app;
use linkify::LinkFinder;
use reqwest::Url;
use wiremock::{ResponseTemplate, Mock}; 
use wiremock::matchers::{path, method};

#[tokio::test]
async fn confirmations_without_tokens_are_rejected_with_a_400() {
    //Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    //email server mock server test
    Mock::given(path("/email"))
    .and(method("POST"))
    .respond_with(ResponseTemplate::new(200))
    .expect(1)
    .mount(&app.mock_server)
    .await;
    //Act
    let _ = app.post_subscriptions(body.into()).await;
    let response = reqwest::get(&format!("{}/subscriptions/confirm",app.address)).await.expect("req failed");
    assert_eq!(response.status().as_u16(),400);

}

#[tokio::test]
async fn the_link_returned_by_subscriber_returns_a_200_if_called() {
    //Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    //email server mock server test
    Mock::given(path("/email"))
    .and(method("POST"))
    .respond_with(ResponseTemplate::new(200))
    .expect(1)
    .mount(&app.mock_server)
    .await;
    //Act
    let _ = app.post_subscriptions(body.into()).await;
    //fetch the request from the mock email server
    let email_request = &app.mock_server.received_requests().await.expect("failed to get a request")[0];
    //extract html and plain text link from mock  email serve request body
    let confirmation_links = app.get_confirmation_links(&email_request);
    assert_eq!(confirmation_links.html,confirmation_links.plain_text);
// Act
//send to /confirm with token
let response = reqwest::get(confirmation_links.html)
.await .unwrap();
// Assert
assert_eq!(response.status().as_u16(), 200);

}


#[tokio::test]
async fn clicking_on_the_confirmation_link_confirms_a_subscriber() {

//Arrange
    let app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    //email server mock server test
    Mock::given(path("/email"))
    .and(method("POST"))
    .respond_with(ResponseTemplate::new(200))
    .expect(1)
    .mount(&app.mock_server)
    .await;
    //Act
    let _ = app.post_subscriptions(body.into()).await;
    //fetch the request from the mock email server
    let email_request = &app.mock_server.received_requests().await.expect("failed to get a request")[0];
    //extract html and plain text link from mock  email serve request body
    let confirmation_links = app.get_confirmation_links(&email_request);
    assert_eq!(confirmation_links.html,confirmation_links.plain_text);
// Act
//send to /confirm with token
//Assert
let response = reqwest::get(confirmation_links.html)
.await .unwrap().error_for_status() .unwrap();
let saved = sqlx::query!("SELECT email,name,status from subscriptions").fetch_one(&app.pool_conn).await.expect("failed to get a record");
assert_eq!(saved.email, "ursula_le_guin@gmail.com"); assert_eq!(saved.name, "le guin"); assert_eq!(saved.status, "confirmed");




}