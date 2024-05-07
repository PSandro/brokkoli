use awc::Client;
use std::fs::File;
use chrono::{DateTime, Utc};
use std::io::Write;

#[derive(Debug, Clone)]
pub struct Camera {
    pub base_url: String,
}

impl Camera {
    pub async fn save_snapshot(&mut self) -> std::io::Result<()> {
        let client = Client::default();
        let mut res = client.get(format!("{}/snapshot", self.base_url))
            .insert_header(("accept", "image/jpeg"))
            .send().await.unwrap();
        std::fs::create_dir_all("snapshots")?;
        let now: DateTime<Utc> = Utc::now();

        if res.status().is_success() {
            let bytes = res.body().await.unwrap();
            let filename = format!(
                "snapshots/{}.jpg",
                now.format("%Y-%m-%dT%H:%M:%S")
                );
            let mut file = File::create(&filename).expect("Failed to create file");
            file.write_all(&bytes)?;
            println!("snapshot {} downloaded successfully!", &filename);
        } else {
            println!("failed to download image: {}", res.status());
        }
        Ok(())
    }
}
