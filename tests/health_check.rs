use std::net::TcpListener;

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};

struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("failed to read configuration.");
    let db_name = Uuid::new_v4().to_string();
    configuration.database.database_name = db_name;
    let db_pool = configure_database(&configuration.database).await;
    let server = zero2prod::run(listener, db_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    TestApp { address, db_pool }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection: PgConnection =
        PgConnection::connect(&config.connection_string_no_db_name())
            .await
            .expect("failed to connect to Postgres");
    connection
        .execute(format!(r#"create database "{}";"#, config.database_name).as_str())
        .await
        .expect("failed to create random databases.");
    let pool = PgPool::connect(&config.connection_string())
        .await
        .expect("failed to connect to random Postgres.");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("failed to migrate random database");
    pool
}

#[actix_rt::test]
async fn health_check_succeeds() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("{}/health_check", &app.address))
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_rt::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("{}/subscriptions", &app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("msg");
    assert_eq!(200, response.status());

    let saved = sqlx::query!("select email, name from subscriptions",)
        .fetch_one(&app.db_pool)
        .await
        .expect("failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[actix_rt::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = client
            .post(&format!("{}/subscriptions", &app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        assert_eq!(
            400,
            response.status(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}
