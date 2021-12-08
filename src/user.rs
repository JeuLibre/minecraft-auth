use serde_json::Value;
use std::sync::{mpsc::channel, Arc, Mutex};
use tokio::runtime::Runtime;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct User {
    /// Email or Pseudo
    pub username: String,

    /// UUID of the current player
    pub uuid: String,

    /// Client token get on connection
    pub client_token: String,

    /// Access token for this account
    pub access_token: String,
}

impl Default for User {
    fn default() -> Self {
        Self {
            username: "".to_owned(),
            uuid: "".to_owned(),
            client_token: "".to_owned(),
            access_token: "".to_owned(),
        }
    }
}

impl User {
    pub fn new(username: String, uuid: String, client_token: String, access_token: String) -> Self {
        Self {
            username,
            uuid,
            client_token,
            access_token,
        }
    }
}

/// Try a connection to minecraft mojang account
///
/// # Example
/// ```
/// use minecraft_auth::user::try_connect;
///
/// let res = try_connect("Username".to_owned(), "Password".to_owned());
/// println!("{:?}", res);
/// ```
pub fn try_connect(username: String, password: String) -> Result<User, String> {
    if username.is_empty() || password.is_empty() {
        Err("Username or password is empty".to_owned())
    } else {
        let rt = Runtime::new().unwrap();

        let us = Arc::new(Mutex::new(username.clone()));
        let ps = Arc::new(Mutex::new(password));
        let (tx, rx) = channel();

        // Move this on other file
        rt.spawn(async move {
            let username = us.lock().unwrap().to_owned();
            let password = ps.lock().unwrap().to_owned();

            let reqwest_client = reqwest::Client::new();
            let body = format!("{{\"agent\": {{\"name\": \"Minecraft\",\"version\":1}},\"username\":\"{}\",\"password\":\"{}\"}}", username, password);
            let res = reqwest_client
                .post("https://authserver.mojang.com/authenticate")
                .body(body)
                .send()
                .await;

            let json: Value = match res {
                Ok(r) => {
                    let data = r.text().await.unwrap();
                    let json = serde_json::from_str(&data).unwrap();
                    json
                }
                Err(_) => Value::default(),
            };

            tx.send(json).unwrap();
        });

        match rx.recv() {
            Ok(data) => {
                if let Some(error) = data["errorMessage"].as_str() {
                    Err(error.to_owned())
                } else {
                    let client_token = data["clientToken"].as_str().unwrap().to_string();
                    let access_token = data["accessToken"].as_str().unwrap().to_string();
                    let uuid = data["selectedProfile"]["id"].as_str().unwrap().to_string();

                    Ok(User::new(username, uuid, client_token, access_token))
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}

#[test]
fn user_connection() {
    let res = try_connect("Username".to_string(), "Password".to_string());
    assert!(res.is_ok(), "Error: {:?}", res.err());
    let user = res.unwrap();
    assert_ne!(User::default(), user);
}
