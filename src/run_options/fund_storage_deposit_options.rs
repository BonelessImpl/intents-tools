use clap::Parser;
use std::str::FromStr;

#[derive(Parser, Clone, Debug, Default)]
pub struct FundStorageDepositOptions {
    /// Run all commands normally, except for transaction commands that would modify the blockchain
    #[clap(long)]
    pub dry_run: bool,

    /// A file that contents the list of tokens, in the format "nep141:<token id>", separated by new lines
    /// You can comment whole lines by prefixing with `#`
    #[clap(long, short('f'), value_name = "PATH")]
    pub tokens_list_file: std::path::PathBuf,

    /// The account that will be used to pay the near required for storage deposits, and that will sign these transactions
    #[clap(long, short('s'), value_name = "ACCOUNT-ID")]
    pub source_account_for_near: String,

    /// The account, which will benefit from these storage deposits
    #[clap(long, short('t'), value_name = "ACCOUNT-ID")]
    pub deposit_beneficiary: String,

    /// The minimum amount of near for the source account to have to pay the fees, besides the storage deposit costs
    /// Example: 1 NEAR, 1000000 yoctoNear, etc.
    #[clap(long, short('n'), value_name = "AMOUNT", default_value("1 NEAR"))]
    #[arg(value_parser(parse_near))]
    pub min_required_balance_for_fees: u128,
}

fn parse_near(val: &str) -> Result<u128, String> {
    near_sdk::NearToken::from_str(val)
        .map(|v| v.as_yoctonear())
        .map_err(|_| format!("'{}' is not a valid integer", val))
}
