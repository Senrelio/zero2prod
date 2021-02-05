#[actix_rt::test]
async fn health_check_succeeds() {
    spawn_app();
    let client = reqwest::Client::new();
    let response = client
        .get("http://127.0.0.1:9001/health_check")
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() {
    let server = zero2prod::run().expect("failed to bind address");
    let _ = tokio::spawn(server);
}
