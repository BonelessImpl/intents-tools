use std::io::Read;

use anyhow::Context;

pub struct Subtoken {
    pub token_prefix: String,
    pub poa_factory_account: String,
}

impl Subtoken {
    pub fn account_id(&self) -> String {
        format!("{}.{}", self.token_prefix, self.poa_factory_account)
    }

    pub fn from_line(poa_factory_account: impl Into<String>, line: &str) -> anyhow::Result<Self> {
        let line = line.trim();
        if !line.is_ascii() {
            return Err(anyhow::anyhow!(
                "Only ascii tokens supported. Error at: {line}"
            ));
        }

        if line.split_whitespace().count() > 1 {
            return Err(anyhow::anyhow!("Token names can't have white spaces"));
        }

        Ok(Self {
            token_prefix: line.to_string(),
            poa_factory_account: poa_factory_account.into(),
        })
    }
}

pub struct SubtokenList {
    pub tokens_list: Vec<Subtoken>,
}

impl SubtokenList {
    pub fn read_list_from_file(
        poa_factory_account: String,
        subtoken_list_file: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<Self> {
        let mut f = std::fs::File::open(subtoken_list_file).context("Opening subtokens file")?;

        let mut file_data = String::new();
        f.read_to_string(&mut file_data)
            .context("Reading subtokens file")?;

        let result = file_data
            .trim()
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.starts_with("#"))
            .filter(|l| !l.is_empty())
            .map(|l| Subtoken::from_line(poa_factory_account.clone(), l))
            .collect::<Result<Vec<_>, _>>()
            .context("Not all lines have valid data")?;

        Ok(Self {
            tokens_list: result,
        })
    }
}
