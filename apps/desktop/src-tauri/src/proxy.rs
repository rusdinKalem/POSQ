use serde::de::DeserializeOwned;
use reqwest::Client;

pub async fn forward_request<T: DeserializeOwned>(
    master_ip: &str,
    endpoint: &str,
) -> Result<T, String> {
    let url = format!("http://{}/api/{}", master_ip, endpoint);
    
    let client = Client::new();
    let res = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Network Error: {}", e))?;

    if res.status().is_success() {
        let data = res.json::<T>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;
        Ok(data)
    } else {
        Err(format!("Server returned error: {}", res.status()))
    }
}
