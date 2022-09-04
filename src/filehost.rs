use serde::{Deserialize, Serialize};

/// Record for an entry on the MEGA65 FileHost website
#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub fileid: String,
    pub title: String,
    pub category: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub os: String,
    pub rating: String,
    pub downloads: String,
    pub published: String,
    pub sortdate: String,
    pub versionid: String,
    pub filename: String,
    pub size: String,
    pub location: String,
    pub author: String,
}

impl Record {
    pub fn print(&self) {
        println!("{} {}", self.kind, self.title);
    }

    // /// Create headers for use w. tui
    // pub fn table_header(&self) -> Vec<String> {
    //     vec![&self.title, &self.kind, &self.author]
    // }
}

/// Get list of records from the filehost
pub async fn get_file_list() -> Result<Vec<Record>, Box<dyn std::error::Error>> {
    let url = "https://files.mega65.org/php/readfilespublic.php";
    let body = reqwest::get(url).await?.text().await?;
    Ok(serde_json::from_str(&body)?)
}
