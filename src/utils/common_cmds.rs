use std::str::FromStr;

use anyhow::Context;

use super::cmd::call_cmd;

pub fn get_account_balance(account_id: &str) -> anyhow::Result<u128> {
    let cmd_args = ["near", "state", account_id, "--networkId", "mainnet"];

    let output = call_cmd(cmd_args)?;

    let output_str = String::from_utf8(output.stdout)?;

    let balance_prefix_in_command = "Native account balance";

    let balance = output_str
        .lines()
        .filter(|l| l.trim().starts_with(balance_prefix_in_command))
        .map(|l| {
            l.trim()
                .strip_prefix(balance_prefix_in_command)
                .expect("Must work")
                .trim()
        })
        .next()
        .ok_or(anyhow::anyhow!(
            "Failed to find account balance for account: {account_id}"
        ))?;

    let balance = near_sdk::NearToken::from_str(balance).context("Parsing balance to near")?;

    Ok(balance.as_yoctonear())
}
