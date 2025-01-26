use e2b_sdk::{E2BClient, ExecuteResponse};
use std::error::Error;
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let api_key = std::env::var("E2B_API_KEY").expect("E2B_API_KEY must be set");
    let client = E2BClient::new(api_key);

    let sandbox_id = client.create_sandbox().await?;
    println!("Have sandbox id: {}", sandbox_id);
    let code = r#"
    import time

    print("Starting...")
    for i in range(5):
        print(f"Sleeping for {i} seconds...")
        time.sleep(1)
    time.sleep(1)
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
