mod subtoken;

use std::path::{Path, PathBuf};

use anyhow::Context;
use near_sdk::NearToken;
use rand::{Rng, distr::Alphanumeric};
use subtoken::{Subtoken, SubtokenList};

use crate::{
    run_options::replace_poa_token_contract_options::ReplacePoATokenContractOptions,
    utils::{
        CACHEDIR_TAG_CONTENTS, cmd::call_cmd, common_cmds::get_account_balance,
        once_destructor::OnceDestructor,
    },
};

fn generate_duplicate_file_suffix() -> String {
    let datetime = chrono::Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let suffix: String = rand::rng()
        .sample_iter(&Alphanumeric)
        .take(6)
        .map(char::from)
        .collect();
    format!("{}_{}", datetime, suffix)
}

fn read_seed_phrase(file_path: &Path) -> anyhow::Result<String> {
    if !file_path.exists() {
        return Err(anyhow::anyhow!(
            "The seed phrase file `{}` does not exist.",
            file_path.display()
        ));
    }

    let seed = std::fs::read_to_string(file_path).context("While reading seed phrase file")?;
    let seed = seed.trim();
    if seed.split_whitespace().count() != 12 || !seed.is_ascii() {
        return Err(anyhow::anyhow!(
            "Invalid seed phrase found in seed phrase file `{}`. Expected 12 ascii words.",
            file_path.display()
        ));
    }
    Ok(seed.to_string())
}

fn public_key_from_seed_phrase(seed_phrase: &str) -> anyhow::Result<String> {
    let cmd_args = [
        "near",
        "--quiet",
        "account",
        "get-public-key",
        "from-seed-phrase",
        seed_phrase,
        "--seed-phrase-hd-path",
        "m/44'/397'/0'",
    ];

    let call_result =
        call_cmd(cmd_args).context("While calling command to get public key from seed")?;

    let public_key_output = String::from_utf8(call_result.stderr)
        .expect("Failed to convert public key output to to UTF-8 string");

    let public_key = public_key_output
        .lines()
        .filter_map(|l| l.strip_prefix("Public key:"))
        .next()
        .expect("Getting the public key from the output failed. Output found: `{public_key}`")
        .trim();

    Ok(public_key.to_string())
}

fn add_full_access_key_to_poa_token(
    signer: &str,
    token: &Subtoken,
    public_key: &str,
    attach_one_yoctonear: bool,
) -> anyhow::Result<()> {
    let yoctonear_amount = attach_one_yoctonear as u128;
    let cmd_args = [
        "near",
        "contract",
        "call-function",
        "as-transaction",
        &token.account_id(),
        "add_full_access_key",
        "json-args",
        &format!("{{ \"public_key\": \"{public_key}\" }}"),
        "prepaid-gas",
        "100.0Tgas",
        "attached-deposit",
        &format!("{yoctonear_amount}yoctonear"),
        "sign-as",
        signer,
        "network-config",
        "mainnet",
        "sign-with-keychain",
        "send",
    ];

    call_cmd(cmd_args)?;

    println!("✔ - Added full access key to `{}`", token.account_id());

    Ok(())
}

fn delete_key_to_poa_token(
    signer: &str,
    token: &Subtoken,
    public_key: &str,
    attach_one_yoctonear: bool,
) -> anyhow::Result<()> {
    let yoctonear_amount = attach_one_yoctonear as u128;
    let cmd_args = [
        "near",
        "contract",
        "call-function",
        "as-transaction",
        &token.account_id(),
        "delete_key",
        "json-args",
        &format!("{{ \"public_key\": \"{public_key}\" }}"),
        "prepaid-gas",
        "100.0Tgas",
        "attached-deposit",
        &format!("{yoctonear_amount}yoctonear"),
        "sign-as",
        signer,
        "network-config",
        "mainnet",
        "sign-with-keychain",
        "send",
    ];

    call_cmd(cmd_args)?;

    println!("✔ - Removed access key from `{}`", token.account_id());

    Ok(())
}

fn deploy_contract(
    seed_phrase: &str,
    token: &Subtoken,
    wasm_file_path: &Path,
) -> anyhow::Result<()> {
    let cmd_args = [
        "near",
        "contract",
        "deploy",
        &token.account_id(),
        "use-file",
        wasm_file_path.to_str().ok_or(anyhow::anyhow!(
            "Failed to convert wasm file path `{}` to string",
            wasm_file_path.display()
        ))?,
        "without-init-call",
        "network-config",
        "mainnet",
        "sign-with-seed-phrase",
        seed_phrase,
        "--seed-phrase-hd-path",
        "m/44'/397'/0'",
        "send",
    ];

    call_cmd(cmd_args)?;

    println!("✔ - Replaced contract in `{}`", token.account_id());

    Ok(())
}

fn backup_contracts(contract_backup_dir: &Path, tokens: &SubtokenList) -> anyhow::Result<()> {
    {
        std::fs::create_dir_all(contract_backup_dir)
            .context("Creating dir for contract backups")?;
        let cachedir_tag_path = contract_backup_dir.join("CACHEDIR.TAG");
        std::fs::write(cachedir_tag_path, CACHEDIR_TAG_CONTENTS).context("Writing CACHETAG.DIR")?;
    }

    for subtoken in &tokens.tokens_list {
        println!("Backing up contract for token: {}", subtoken.account_id());
        let wasm_backup_path = contract_backup_dir.join(format!("{}.wasm", subtoken.account_id()));
        // If the file already exists, rename it and don't replace
        let wasm_backup_path = if wasm_backup_path.exists() {
            let stem = wasm_backup_path.with_extension(""); // remove wasm extension
            let stem = stem.to_str().expect("Converting stem to string failed");
            let new_path = format!("{stem}_{}.wasm", generate_duplicate_file_suffix());
            PathBuf::from(new_path)
        } else {
            wasm_backup_path
        };

        // Download wasm
        {
            let cmd_args = [
                "near",
                "contract",
                "download-wasm",
                &subtoken.account_id(),
                "save-to-file",
                wasm_backup_path
                    .to_str()
                    .expect("Converting path to str failed"),
                "network-config",
                "mainnet",
                "now",
            ];

            call_cmd(cmd_args)?;

            println!(
                "✔ - Backup of wasm in contract `{}` is successful.",
                subtoken.account_id()
            );
        }
    }

    println!("✔ - All backups successful");

    Ok(())
}

pub fn run(options: ReplacePoATokenContractOptions) -> anyhow::Result<()> {
    let token_ids_list_path = options.tokens_prefixes_list_file;
    let source_account = &options.source_account_for_action;
    let min_required_balance_for_fees_in_yocto_near = options.min_required_balance_for_fees;
    let no_one_yocto_for_key_adding = options.no_one_yocto_for_key_adding;
    let poa_factory_contract_id = options.poa_factory_account_id;
    let poa_token_wasm_file = options.poa_token_wasm_file;
    let contract_backup_dir = options.poa_tokens_contract_backup_dir;
    let seed_phrase_file_path = options.poa_seed_file;

    if !poa_factory_contract_id.is_ascii() || poa_factory_contract_id.split_whitespace().count() > 1
    {
        return Err(anyhow::anyhow!(
            "Invalid PoA factory contract id. Expected ASCII without spaces."
        ));
    }

    if !poa_token_wasm_file.exists() {
        return Err(anyhow::anyhow!(
            "The PoA token wasm file `{}` does not exist.",
            poa_token_wasm_file.display(),
        ));
    }

    let source_account_balance =
        get_account_balance(source_account).context("While checking source account balance")?;

    if source_account_balance < min_required_balance_for_fees_in_yocto_near {
        return Err(anyhow::anyhow!(
            "The provided source account `{source_account}` has balance {} less than the lower limit: {}",
            NearToken::from_yoctonear(source_account_balance),
            NearToken::from_yoctonear(min_required_balance_for_fees_in_yocto_near)
        ));
    }

    let tokens = SubtokenList::read_list_from_file(poa_factory_contract_id, token_ids_list_path)?;

    let seed_phrase = read_seed_phrase(&seed_phrase_file_path)?;

    let public_key = public_key_from_seed_phrase(&seed_phrase)?;

    println!("Derived public key from your seed phrase: {}", public_key);

    backup_contracts(&contract_backup_dir, &tokens)?;

    println!(
        "\n⚠ About to start replacing contracts. The process will start in a few seconds. DO NOT stop it before it finishes.\n"
    );

    std::thread::sleep(std::time::Duration::from_secs(8));

    for subtoken in &tokens.tokens_list {
        add_full_access_key_to_poa_token(
            source_account,
            subtoken,
            &public_key,
            !no_one_yocto_for_key_adding,
        )
        .context("While adding full access key")?;

        // We delete the key eventually
        let _access_key_deleter = OnceDestructor::new(|| {
            delete_key_to_poa_token(
                source_account,
                subtoken,
                &public_key,
                !no_one_yocto_for_key_adding,
            )
            .context("While adding full access key")
            .expect("Deleting access key failed")
        });

        deploy_contract(&seed_phrase, subtoken, &poa_token_wasm_file)?;
    }

    println!();
    println!("End of program reached");

    Ok(())
}
