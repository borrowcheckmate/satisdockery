use std::{io, time::Duration};

use reqwest::{Client, StatusCode, header::HeaderValue, redirect::Policy};
use serde::Deserialize;
use serde_json::{Value, json};

use super::ServerSettings;
use crate::Result;

pub(super) struct Api {
    client: Client,
    endpoint: String,
    authorization: HeaderValue,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct GameState {
    pub(super) num_connected_players: u32,
    pub(super) is_game_running: bool,
    pub(super) active_session_name: String,
    pub(super) auto_load_session_name: String,
}

impl Api {
    pub(super) async fn new(settings: &ServerSettings) -> Result<Self> {
        let token_file = settings.idle_api_token_file.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Server API access requires an application token file",
            )
        })?;
        let token = tokio::fs::read_to_string(token_file).await?;
        let token = token.trim();
        if token.is_empty() {
            return Err(
                io::Error::new(io::ErrorKind::InvalidInput, "Empty application API token").into(),
            );
        }
        let mut authorization =
            HeaderValue::from_str(&format!("Bearer {token}")).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "Invalid application API token")
            })?;
        authorization.set_sensitive(true);
        let client = Client::builder()
            .no_proxy()
            .redirect(Policy::none())
            // The game ships a self-signed certificate. This client only talks to loopback.
            .tls_danger_accept_invalid_certs(true)
            .build()?;
        Ok(Self {
            client,
            endpoint: format!("https://127.0.0.1:{}/api/v1", settings.game_port),
            authorization,
        })
    }

    pub(super) async fn state(&self) -> Result<GameState> {
        #[derive(Deserialize)]
        struct Data {
            #[serde(rename = "serverGameState")]
            server_game_state: GameState,
        }

        let result = self.call("QueryServerState", json!({})).await?;
        Ok(serde_json::from_value::<Data>(result)?.server_game_state)
    }

    pub(super) async fn save(&self, save_name: &str) -> Result<()> {
        self.call("SaveGame", json!({ "saveName": save_name }))
            .await?;
        Ok(())
    }

    pub(super) async fn shutdown(&self) -> Result<()> {
        self.call("Shutdown", json!({})).await?;
        Ok(())
    }

    pub(super) async fn run_command(&self, command: &str) -> Result<(String, bool)> {
        let data = self
            .call("RunCommand", json!({ "command": command }))
            .await?;
        command_result(data)
    }

    async fn call(&self, function: &str, data: Value) -> Result<Value> {
        let timeout = if function == "SaveGame" {
            Duration::from_secs(120)
        } else {
            Duration::from_secs(10)
        };
        let response = self
            .client
            .post(&self.endpoint)
            .timeout(timeout)
            .header(reqwest::header::AUTHORIZATION, self.authorization.clone())
            .json(&json!({ "function": function, "data": data }))
            .send()
            .await?
            .error_for_status()?;
        match response.status() {
            StatusCode::NO_CONTENT => Ok(Value::Null),
            StatusCode::OK => completed_response(&response.bytes().await?, function),
            status => Err(io::Error::other(format!(
                "{function} returned unexpected status {status}"
            ))
            .into()),
        }
    }
}

fn command_result(data: Value) -> Result<(String, bool)> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        #[serde(default)]
        command_result: String,
        return_value: Option<bool>,
    }

    let output: Output = serde_json::from_value(data)?;
    Ok((output.command_result, output.return_value.unwrap_or(true)))
}

fn completed_response(body: &[u8], function: &str) -> Result<Value> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        #[serde(default)]
        data: Value,
        error_code: Option<String>,
    }

    let response: Response = serde_json::from_slice(body)?;
    if response.error_code.is_some() {
        return Err(io::Error::other(format!("{function} returned a game API error")).into());
    }
    Ok(response.data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_console_output_and_failure() {
        let data = json!({ "commandResult": "Unknown command", "returnValue": false });
        assert_eq!(
            command_result(data).unwrap(),
            ("Unknown command".into(), false)
        );
        let older = json!({ "commandResult": "Game saved" });
        assert_eq!(command_result(older).unwrap(), ("Game saved".into(), true));
    }

    #[test]
    fn an_api_error_in_a_successful_http_response_is_still_a_failure() {
        assert!(completed_response(br#"{"errorCode":"save_game_failed"}"#, "SaveGame").is_err());
        assert!(completed_response(b"{}", "SaveGame").is_ok());
    }
}
