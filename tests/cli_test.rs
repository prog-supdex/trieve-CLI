use assert_cmd::Command;
use rexpect::session::spawn_command;
use std::error::Error;

#[test]
fn test_no_arguments() {
    let mut cmd = Command::cargo_bin("trieve").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Usage"));
}

#[test]
fn test_login_with_mockito() -> Result<(), Box<dyn Error>> {
    let mut server = mockito::Server::new();
    let url = server.url();

    // let auth_mock = server.mock("GET", "/api/auth")
    //     .with_status(200)
    //     .with_header("content-type", "application/json")
    //     .with_body(r#"{"api_key":"mocked_api_key"}"#)
    //     .create();

    let user_info_mock = server
        .mock("GET", "/api/auth/me")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "email": "mocked_user@test.com",
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "Mocked User",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "deleted": 0,
            "orgs": [
                {
                    "name": "Some Organization",
                    "id": "123e4567-e89b-12d3-a456-426614174001",
                    "created_at": "2024-01-01T00:00:00Z",
                    "updated_at": "2024-01-01T00:00:00Z",
                    "deleted": 0
                }
            ],
            "user_orgs": [
                {
                    "created_at": "2024-01-01T00:00:00Z",
                    "id": "123e4567-e89b-12d3-a456-426614174002",
                    "organization_id": "123e4567-e89b-12d3-a456-426614174001",
                    "role": 1,
                    "updated_at": "2024-01-01T00:00:00Z",
                    "user_id": "123e4567-e89b-12d3-a456-426614174000",
                    "deleted": 0
                }
            ]
        }"#,
        )
        .create();

    let bin_path = assert_cmd::cargo::cargo_bin("trieve");
    let mut cmd = std::process::Command::new(bin_path);
    cmd.arg("login").arg("--api-key").arg("mocked_api_key");

    let mut process = spawn_command(cmd, Some(300))?;
    process.exp_string("Welcome back to the Trieve CLI! Let's update your configuration.")?;

    process.exp_regex("Would you like to use the production Trieve server")?;
    process.send("n\n")?;

    process.exp_regex("Trieve Server URL:")?;
    process.send_line(&url)?; // put the mock server url like a trieve server url

    process.exp_string("Select an organization to use: ")?;
    process.send("\n")?; // select the first selected organization

    process.exp_string("Enter a name for this profile: ")?;
    process.send("default\n")?;

    process.exp_string("Profile already exists. Overwrite? (y/N)")?;
    process.send("y\n")?;

    process.exp_eof()?;

    // auth_mock.assert();
    user_info_mock.assert();

    Ok(())
}

#[test]
fn test_dataset_list() {
    let mut server = mockito::Server::new();
    let url = server.url();

    let datasets_mock = server
        .mock(
            "GET",
            "/api/dataset/organization/123e4567-e89b-12d3-a456-426614174005",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"[
                {
                    "dataset": {
                        "client_configuration": {
                            "key": "value"
                        },
                        "created_at": "2024-11-12T00:00:00",
                        "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
                        "name": "Trieve",
                        "organization_id": "123e4567-e89b-12d3-a456-426614174005",
                        "updated_at": "2024-11-12T00:00:00",
                        "deleted": 0
                    },
                    "dataset_usage": {
                        "chunk_count": 100,
                        "dataset_id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
                        "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3"
                    }
                }
            ]"#,
        )
        .create();

    let output = Command::cargo_bin("trieve")
        .unwrap()
        .arg("dataset")
        .arg("list")
        .env("TRIEVE_NO_PROFILE", "true")
        .env("TRIEVE_API_KEY", "mocked_api_key")
        .env("TRIEVE_API_URL", &url)
        .env("TRIEVE_ORG_ID", "123e4567-e89b-12d3-a456-426614174005")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let output_str = String::from_utf8_lossy(&output);

    assert!(output_str.contains("Trieve"));
    assert!(output_str.contains("100"));
    assert!(output_str.contains("2024-11-12"));
    assert!(output_str.contains("e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3"));

    datasets_mock.assert();
}

#[test]
fn test_create_dataset() {
    let mut server = mockito::Server::new();
    let url = server.url();

    let create_dataset_mock = server
        .mock("POST", "/api/dataset")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "created_at": "2024-01-01T00:00:00.000",
            "id": "e3e3e3e3-e3e3-e3e3-e3e3-e3e3e3e3e3e3",
            "name": "Trieve",
            "organization_id": "123e4567-e89b-12d3-a456-426614174001",
            "server_configuration": {},
            "tracking_id": "foobar-dataset",
            "updated_at": "2024-01-01T00:00:00.000",
            "deleted": 0
        }"#,
        )
        .create();

    Command::cargo_bin("trieve")
        .unwrap()
        .arg("dataset")
        .arg("create")
        .arg("--name")
        .arg("Trieve")
        .env("TRIEVE_NO_PROFILE", "true")
        .env("TRIEVE_API_KEY", "mocked_api_key")
        .env("TRIEVE_API_URL", &url)
        .env("TRIEVE_ORG_ID", "123e4567-e89b-12d3-a456-426614174000")
        .assert()
        .success()
        .stdout(predicates::str::contains("Dataset created successfully!"));

    create_dataset_mock.assert();
}

#[test]
fn test_create_organization() {
    let mut server = mockito::Server::new();
    let url = server.url();

    let create_organization_mock = server
        .mock("POST", "/api/organization")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
            "id": "123e4567-e89b-12d3-a456-426614174000",
            "name": "Test Organization",
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z",
            "deleted": 0
        }"#,
        )
        .create();

    Command::cargo_bin("trieve")
        .unwrap()
        .arg("organization")
        .arg("create")
        .arg("Test Organization")
        .env("TRIEVE_API_KEY", "mocked_api_key")
        .env("TRIEVE_API_URL", &url)
        .env("TRIEVE_NO_PROFILE", "true")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Organization '123e4567-e89b-12d3-a456-426614174000' created.",
        ));

    create_organization_mock.assert();
}
