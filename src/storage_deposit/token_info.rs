use anyhow::Context;
use std::io::Read;

#[derive(Clone, Debug)]
pub struct TokenInformation {
    #[allow(dead_code)]
    pub token_type: String,
    pub token_account: String,
}

impl TokenInformation {
    pub fn from_line(line: &str) -> anyhow::Result<Self> {
        let parts = line.split(':').collect::<Vec<_>>();

        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid pattern of line, expected <token type>:<token account>. Found: {line}"
            ));
        }

        Ok(TokenInformation {
            token_type: parts[0].to_string(),
            token_account: parts[1].to_string(),
        })
    }

    pub fn read_token_ids_file(p: impl AsRef<std::path::Path>) -> anyhow::Result<Vec<Self>> {
        let mut f = std::fs::File::open(p).context("Opening tokens file")?;

        let mut file_data = String::new();
        f.read_to_string(&mut file_data)
            .context("Reading tokens file")?;

        let result = file_data
            .trim()
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.starts_with("#"))
            .filter(|l| !l.is_empty())
            .map(TokenInformation::from_line)
            .collect::<Result<Vec<_>, _>>()
            .context("Not all lines have valid data")?;

        Ok(result)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct StorageBalanceBounds {
    #[serde(deserialize_with = "u128_from_str")]
    min: u128,
    #[serde(deserialize_with = "u128_from_str")]
    max: u128,
}

impl StorageBalanceBounds {
    pub fn get_preferred_value(&self) -> u128 {
        // We can use both values to calculate the value we prefer to deposit
        self.min
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
#[serde(untagged)]
pub enum StorageBalance {
    Null,
    Value {
        #[serde(deserialize_with = "u128_from_str")]
        available: u128,
        #[serde(deserialize_with = "u128_from_str")]
        total: u128,
    },
}

impl StorageBalance {
    pub fn get_balance(&self) -> u128 {
        match self {
            StorageBalance::Null => 0,
            StorageBalance::Value {
                available: _,
                total,
            } => *total,
        }
    }
}

fn u128_from_str<'de, D>(deserializer: D) -> Result<u128, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;

    let s: &str = serde::Deserialize::deserialize(deserializer)?;
    // do better hex decoding than this
    s.parse::<u128>()
        .map_err(|e| D::Error::custom(format!("u128 parsing from {s}. Error: {e}")))
}
