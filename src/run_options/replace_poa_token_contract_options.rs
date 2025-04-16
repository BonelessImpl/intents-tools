use clap::Parser;
use std::{path::PathBuf, str::FromStr};

#[derive(Parser, Clone, Debug, Default)]
pub struct ReplacePoATokenContractOptions {
    /// Path to a text file containing the seed phrase that creates the public key that authorizes access to the PoA factory,
    /// and whose public key will be added (and removed) from PoA token contracts to be able to replace their contract code.
    #[clap(long, default_value("poa-seed.txt"))]
    pub poa_seed_file: PathBuf,

    /// The account id that contains the PoA factory
    #[clap(long)]
    pub poa_factory_account_id: String,

    /// A file that contents the list of tokens, but only their prefixes. Not full addresses.
    /// The PoA factory will be the suffix.
    /// If the PoA factory account id is poa-factory.example.near, and tokens under it are
    /// `ft1.poa-factory.example.near` and `ft1.poa-factory.example.near`, then this file
    /// contains lines with entries `ft1` and `ft2`.
    #[clap(long, short('f'), value_name = "PATH")]
    pub tokens_prefixes_list_file: std::path::PathBuf,

    /// The account that will be signing transactions to add full access keys to PoA token contracts' accounts.
    /// Note that this account should have the power to add/remove access keys to PoA tokens.
    /// The key should be in the keychain of the OS.
    #[clap(long, short('s'), value_name = "ACCOUNT-ID")]
    pub source_account_for_action: String,

    /// The minimum amount of near for the source account to have to pay the fees
    /// Example: 1 NEAR, 1000000 yoctoNear, etc.
    #[clap(long, short('n'), value_name = "AMOUNT", default_value("1 NEAR"))]
    #[arg(value_parser(parse_near))]
    pub min_required_balance_for_fees: u128,

    /// Adding and removing full access keys generally requires having attached deposit
    /// of one yoctoNEAR. However, due to a mistake, this may not be enforced in the contract.
    #[clap(long)]
    pub no_one_yocto_for_key_adding: bool,

    /// The wasm files that will be replacing the PoA token contracts
    #[clap(long)]
    pub poa_token_wasm_file: PathBuf,

    /// The directory (absolute or relative) where backup contracts will be stored, before writing new ones.
    #[clap(long, default_value("poa-tokens-contracts-backup"))]
    pub poa_tokens_contract_backup_dir: PathBuf,
}

fn parse_near(val: &str) -> Result<u128, String> {
    near_sdk::NearToken::from_str(val)
        .map(|v| v.as_yoctonear())
        .map_err(|_| format!("'{}' is not a valid integer", val))
}
