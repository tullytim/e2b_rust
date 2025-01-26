use reqwest::{Client};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc;
use futures_util::StreamExt;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct E2BClient {
    api_key: String,
    client: Client,
}

#[derive(Debug, Serialize)]
struct CreateSandboxRequest {
    #[serde(rename = "templateID")]  
    template_id: String,
    timeout: u32,
}

#[derive(Debug, Deserialize)]
struct CreateSandboxResponse {
    #[serde(rename = "sandboxID")]
    pub sandbox_id: String,
    #[serde(rename = "clientID")]
    pub client_id: String,
}

#[derive(Debug, Serialize)]
struct ExecuteCodeRequest {
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ExecuteResponse {
    #[serde(rename = "stdout")]
    Stdout {
        text: String,
        timestamp: DateTime<Utc>,
    },
    #[serde(rename = "stderr")]
    Stderr { name: String },
    #[serde(rename = "result")]
    Result { content: String },
    #[serde(rename = "error")]
    Error { name: String, value: String },
}

impl E2BClient {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { api_key, client }
    }

    pub async fn create_sandbox(&self) -> Result<String, Box<dyn Error>> {
        let request = CreateSandboxRequest {
            template_id: "code-interpreter-beta".to_string(),
            timeout: 10,
        };

        let response = self.client
            .post("https://api.e2b.dev/sandboxes")
            .header("X-API-Key", &self.api_key)
            .json(&request)
            .send()
            .await?;

           // Print the status code and headers
           println!("Status: {}", response.status());
           println!("Headers: {:#?}", response.headers());

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(format!("API error: {}", error_text).into());
        }
        let body = response.text().await?;
        println!("Response body: {}", body);
        let parsed: CreateSandboxResponse = serde_json::from_str(&body)?;
        
        Ok(format!("{}-{}", parsed.sandbox_id, parsed.client_id))
    }

    pub async fn execute_code(
        &self,
        sandbox_id: String,
        code: &str,
    ) -> Result<mpsc::Receiver<ExecuteResponse>, Box<dyn Error>> {
        let request = ExecuteCodeRequest {
            code: code.to_string(),
        };

        let (tx, rx) = mpsc::channel(100);
        let client = self.client.clone();
        let api_key = self.api_key.clone();

        tokio::spawn(async move {
            let response = client
                .post(format!("https://49999-{}.e2b.dev/execute", sandbox_id))
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&request)
                .send()
                .await;

            match response {
                Ok(stream) => {
                    let mut stream = stream.bytes_stream();
                    while let Some(chunk) = stream.next().await {
                        if let Ok(bytes) = chunk {
                            //println!("Received chunk: {:?}", String::from_utf8_lossy(&bytes));
                            if let Ok(response) = serde_json::from_slice::<ExecuteResponse>(&bytes) {
                                println!("Sending response through channel");
                                if let Err(e) = tx.send(response).await {
                                    eprintln!("Failed to send response through channel: {}", e);
                                    break;
                                }
                            }
                            else {
                                eprintln!("Failed to parse response");
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to execute code: {}", e);
                }
            }
        });

        Ok(rx)
    }

    pub async fn kill_sandbox(&self, sandbox_id: &str) -> Result<(), Box<dyn Error>> {
        self.client
            .delete(format!("https://api.e2b.dev/sandboxes/{}", sandbox_id))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;

        Ok(())
    }
}


