mod token_info;

use std::str::FromStr;

use anyhow::Context;
use token_info::{StorageBalance, StorageBalanceBounds, TokenInformation};

use crate::run_options::fund_storage_deposit_options::FundStorageDepositOptions;

fn format_command(args: &[&str]) -> String {
    args.iter()
        .map(|arg| {
            if arg.chars().any(|c| c.is_whitespace()) {
                format!("'{}'", arg) // Wrap JSON args in single quotes
            } else {
                arg.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn call_cmd<'a>(
    cmd_and_args: impl IntoIterator<Item = &'a str>,
) -> anyhow::Result<std::process::Output> {
    let cmd_and_args = cmd_and_args.into_iter().collect::<Vec<_>>();

    if cmd_and_args.is_empty() {
        return Err(anyhow::anyhow!(
            "Attempted to make a system call to an empty command"
        ));
    }

    let full_cmd = format_command(&cmd_and_args);
    println!("- Calling command: {full_cmd}");

    let mut cmd = std::process::Command::new(cmd_and_args[0]);
    for arg in cmd_and_args.into_iter().skip(1) {
        cmd.arg(arg);
    }

    let output = cmd
        .output()
        .context(format!("While calling command: {}", full_cmd))?;

    if output.status.success() {
        Ok(output)
    } else {
        Err(anyhow::anyhow!(
            "Command `{}` exited with error.\nStdout: {}\n\nStderr: {}",
            full_cmd,
            String::from_utf8(output.stdout)
                .unwrap_or("<Stderr to string conversion failed>".to_string()),
            String::from_utf8(output.stderr)
                .unwrap_or("<Stderr to string conversion failed>".to_string())
        ))
    }
}

fn get_storage_balance_bounds(
    tokens: impl IntoIterator<Item = TokenInformation>,
) -> anyhow::Result<Vec<(TokenInformation, StorageBalanceBounds)>> {
    let mut result = Vec::new();

    for token in tokens {
        let cmd_args = [
            "near",
            "contract",
            "call-function",
            "as-read-only",
            &token.token_account.to_string(),
            "storage_balance_bounds",
            "json-args",
            "{}",
            "network-config",
            "mainnet",
            "now",
        ];

        let output = call_cmd(cmd_args)?;

        let output_str = String::from_utf8(output.stdout)?;

        let storage_balance_bounds: StorageBalanceBounds = serde_json::from_str(&output_str)
            .context(format!(
                "While deserializing StorageBalanceBounds for token `{}`. Attempted to deserialize `{output_str}`",
                token.token_account
            ))?;

        result.push((token, storage_balance_bounds));
    }

    Ok(result)
}

fn filter_tokens_with_already_sufficient_storage(
    tokens: impl IntoIterator<Item = (TokenInformation, StorageBalanceBounds)>,
    target_account_to_check: &str,
) -> anyhow::Result<Vec<(TokenInformation, u128)>> {
    let mut result = Vec::new();
    for (token, required) in tokens {
        let cmd_args = [
            "near",
            "contract",
            "call-function",
            "as-read-only",
            &token.token_account.to_string(),
            "storage_balance_of",
            "json-args",
            &format!("{{ \"account_id\": \"{target_account_to_check}\" }}"),
            "network-config",
            "mainnet",
            "now",
        ];

        let output = call_cmd(cmd_args)?;

        let output_str = String::from_utf8(output.stdout)?;

        let storage_balance: StorageBalance = serde_json::from_str(&output_str)
            .context(format!(
                "While deserializing StorageBalance for token `{}`. Attempted to deserialize `{output_str}`. Stderr: {}",
                token.token_account, String::from_utf8(output.stderr).unwrap_or("<Stderr to string conversion failed>".to_string())
            ))?;

        let value_required = required
            .get_preferred_value()
            .saturating_sub(storage_balance.get_balance());

        if value_required > 0 {
            println!(
                "- Adding token: {} as its balance is not enough ({} >= {})",
                token.token_account,
                storage_balance.get_balance(),
                required.get_preferred_value()
            );
            result.push((token, value_required));
        } else {
            println!(
                "- Skipping token: {} as it already has balance >= required ({} >= {})",
                token.token_account,
                storage_balance.get_balance(),
                required.get_preferred_value()
            );
        }
    }

    Ok(result)
}

fn get_account_balance(account_id: &str) -> anyhow::Result<u128> {
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

fn do_storage_deposits(
    source_account_for_near: &str,
    tokens: &[(TokenInformation, u128)],
    target_account_to_fund: &str,
    dry_run: bool,
) -> anyhow::Result<()> {
    for (token, deposit_amount) in tokens {
        let cmd_args = [
            "near",
            "contract",
            "call-function",
            "as-transaction",
            &token.token_account.to_string(),
            "storage_deposit",
            "json-args",
            &format!("{{ \"account_id\": \"{target_account_to_fund}\" }}"),
            "prepaid-gas",
            "100.0Tgas",
            "attached-deposit",
            &format!("{deposit_amount}yoctonear"),
            "sign-as",
            source_account_for_near,
            "network-config",
            "mainnet",
            "sign-with-keychain",
            "send",
        ];

        if dry_run {
            println!("- Would run command: {}", format_command(&cmd_args))
        } else {
            call_cmd(cmd_args)?;
        }
    }

    Ok(())
}

pub fn run(fund_storage_deposit_options: FundStorageDepositOptions) -> anyhow::Result<()> {
    let token_ids_list_path = fund_storage_deposit_options.tokens_list_file;
    let source_account_for_near = &fund_storage_deposit_options.source_account_for_near;
    let target_account_fund_with_deposits = &fund_storage_deposit_options.deposit_beneficiary;
    let min_required_balance_for_fees_in_yocto_near = 100000000000000000000000u128; // 1 NEAR, to be safe
    let dry_run = false;

    let tokens = TokenInformation::read_token_ids_file(token_ids_list_path)?;

    let storage_bounds = get_storage_balance_bounds(tokens)?;

    let storage_bounds_count = storage_bounds.len();

    let token_vs_required_deposit = filter_tokens_with_already_sufficient_storage(
        storage_bounds,
        target_account_fund_with_deposits,
    )?;

    let total_to_deposit: u128 = token_vs_required_deposit.iter().map(|v| v.1).sum();

    let total_required_yocto_near = min_required_balance_for_fees_in_yocto_near + total_to_deposit;

    println!("Total to deposit: {total_required_yocto_near}");

    let source_account_balance = get_account_balance(source_account_for_near)?;

    if source_account_balance < total_required_yocto_near {
        return Err(anyhow::anyhow!(
            "Not enough balance in account. Required: {source_account_balance}. Available: {total_required_yocto_near}."
        ));
    }

    println!(
        "- Running deposit calls for {} tokens",
        token_vs_required_deposit.len()
    );

    println!(
        "- Skipping {} tokens since they have the required storage deposit already.",
        storage_bounds_count - token_vs_required_deposit.len()
    );

    do_storage_deposits(
        source_account_for_near,
        &token_vs_required_deposit,
        target_account_fund_with_deposits,
        dry_run,
    )?;

    println!("Program has reached its end gracefully.");

    Ok(())
}
