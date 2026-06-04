use keyring::Entry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let entry = Entry::new("aether-echo", "groq-api-key")?;
    let key = entry.get_password()?;
    
    let client = reqwest::Client::new();
    let res = client.get("https://api.groq.com/openai/v1/models")
        .header("Authorization", format!("Bearer {}", key))
        .send()
        .await?;
        
    let text = res.text().await?;
    println!("API response: {}", text);
    Ok(())
}
