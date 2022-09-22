// copyright 2022 mikael lund aka wombat
//
// licensed under the apache license, version 2.0 (the "license");
// you may not use this file except in compliance with the license.
// you may obtain a copy of the license at
//
//     http://www.apache.org/licenses/license-2.0
//
// unless required by applicable law or agreed to in writing, software
// distributed under the license is distributed on an "as is" basis,
// without warranties or conditions of any kind, either express or implied.
// see the license for the specific language governing permissions and
// limitations under the license.

//! Routines for access the MEGA65 FileHost

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Record for an entry on the MEGA65 FileHost website
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
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
    fn _print(&self) {
        println!("{} {}", self.kind, self.title);
    }

    /// Create columns for tui list
    pub fn columns(&self) -> Vec<&str> {
        vec![&self.title, &self.kind, &self.author]
    }
}

/// Get list of records from the filehost
pub fn get_file_list() -> Result<Vec<Record>> {
    let url = "https://files.mega65.org/php/readfilespublic.php";
    let body = reqwest::blocking::get(url)?.text()?;
    Ok(serde_json::from_str(&body)?)
}
