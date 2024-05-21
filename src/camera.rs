use awc::Client;
use chrono::{DateTime, Utc};
use std::fs::File;
use std::io::{Write};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Camera {
    pub base_url: String,
}

impl Camera {
    pub async fn save_snapshot(&mut self) -> Result<()> {
        let client = Client::default();
        if let Ok(mut res) = client.get(format!("{}/snapshot", self.base_url))
            .insert_header(("accept", "image/jpeg"))
            .send()
            .await {

            std::fs::create_dir_all("snapshots")?;
            let now: DateTime<Utc> = Utc::now();

            if res.status().is_success() {
                let bytes = res.body().await?;
                let filename = format!(
                    "snapshots/{}.jpg",
                    now.format("%Y-%m-%dT%H:%M:%S")
                );
                let mut file = File::create(&filename)?;
                file.write_all(&bytes)?;
                println!("snapshot {} downloaded successfully!", &filename);
            } else {
                println!("failed to download image: {}", res.status());
            }
        } else {
            println!("could not connect to cam_url: {}", self.base_url);
        }

        Ok(())
    }
}
