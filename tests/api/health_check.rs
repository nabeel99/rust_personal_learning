use crate::helpers::spawn_app;
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