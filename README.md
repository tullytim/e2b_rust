# e2b_rust

A simple Rust library for running code in e2b via Rust

## 🚀 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
e2b_sdk = "0.1.0"
```

## Debugging
```sh
$ RUST_LOG=debug
```

## Usage

```rust
use e2b_sdk::{E2BClient, ExecuteResponse};
use std::error::Error;
use tokio::time::{Duration, timeout};

async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = std::env::var("E2B_API_KEY").expect("E2B_API_KEY must be set");
    let client = E2BClient::new(api_key);

    let sandbox_id = client.create_sandbox().await?;
    println!("Have sandbox id: {}", sandbox_id);
    let code = r#"
    print("Starting...")
    for i in range(5):
        print(f"Running for {i} ...")

    print("Finished!")
        "#;    
    let mut rx = client.execute_code(sandbox_id.clone(), code).await?;
    let timeout_duration = Duration::from_secs(10);

    while let Ok(Some(response)) = timeout(timeout_duration, rx.recv()).await {
         match response {
               ExecuteResponse::Stdout { text, timestamp} => println!("stdout: {} {}", text, timestamp),
               ExecuteResponse::Stderr { name } => eprintln!("stderr: {}", name),
               ExecuteResponse::Result { content } => println!("result: {}", content),
               ExecuteResponse::Error { name, value } => eprintln!("error: {} : {}", name, value),
        }
    }
   
    println!("Killing sandbox...");
    client.kill_sandbox(&sandbox_id).await?;

    Ok(())
}
```
