use assert_cmd::Command;
use rexpect::session::spawn_command;
use std::error::Error;
use lazy_static::lazy_static;
use stubr::Stubr;

// Starting only one "stubr" server for all tests
lazy_static! {
    static ref STUBR: Stubr = stubr::Stubr::start_blocking("tests/stubs");
}

#[test]
fn test_no_arguments() {
    let mut cmd = Command::cargo_bin("trieve").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Usage"));
}

#[tokio::test]
async fn test_login() -> Result<(), Box<dyn Error>> {
    let base_uri = STUBR.uri();

    let bin_path = assert_cmd::cargo::cargo_bin("trieve");
    let mut cmd = std::process::Command::new(bin_path);
    cmd.arg("login").arg("--api-key").arg("mocked_api_key");

    let mut process = spawn_command(cmd, Some(300))?;
    process.exp_string("Welcome back to the Trieve CLI! Let's update your configuration.")?;

    process.exp_regex("Would you like to use the production Trieve server")?;
    process.send("n\n")?;

    process.exp_regex("Trieve Server URL:")?;
    process.send_line(&base_uri)?; // put the mock server url like a trieve server url

    process.exp_string("Select an organization to use: ")?;
    process.send("\n")?; // select the first selected organization

    process.exp_string("Enter a name for this profile: ")?;
    process.send("default\n")?;

    process.exp_string("Profile already exists. Overwrite? (y/N)")?;
    process.send("y\n")?;

    process.exp_eof()?;

    Ok(())
}

#[tokio::test]
async fn test_dataset_list() {
    let base_uri = STUBR.uri();

    let output = Command::cargo_bin("trieve")
        .unwrap()
        .arg("dataset")
        .arg("list")
        .env("TRIEVE_NO_PROFILE", "true")
        .env("TRIEVE_API_KEY", "mocked_api_key")
        .env("TRIEVE_API_URL", &base_uri)
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
}

#[tokio::test]
async fn test_create_dataset() {
    let base_uri = STUBR.uri();

    Command::cargo_bin("trieve")
        .unwrap()
        .arg("dataset")
        .arg("create")
        .arg("--name")
        .arg("Trieve")
        .env("TRIEVE_NO_PROFILE", "true")
        .env("TRIEVE_API_KEY", "mocked_api_key")
        .env("TRIEVE_API_URL", &base_uri)
        .env("TRIEVE_ORG_ID", "123e4567-e89b-12d3-a456-426614174000")
        .assert()
        .success()
        .stdout(predicates::str::contains("Dataset created successfully!"));
}

#[tokio::test]
async fn test_create_organization() {
    let base_uri = STUBR.uri();

    Command::cargo_bin("trieve")
        .unwrap()
        .arg("organization")
        .arg("create")
        .arg("Test Organization")
        .env("TRIEVE_API_KEY", "mocked_api_key")
        .env("TRIEVE_API_URL", &base_uri)
        .env("TRIEVE_NO_PROFILE", "true")
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Organization '123e4567-e89b-12d3-a456-426614174000' created.",
        ));
}
