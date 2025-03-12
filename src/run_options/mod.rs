use clap::{Parser, Subcommand};

pub mod fund_storage_deposit_options;

#[derive(Parser)]
pub struct RunOptions {
    #[clap(subcommand)]
    pub command: RunCommand,
}

#[derive(Subcommand, Clone, Debug)]
pub enum RunCommand {
    /// Run storage deposit funding a list of given tokens
    FundStorageDeposit(fund_storage_deposit_options::FundStorageDepositOptions),
}
