pub mod fund_storage_deposit_options;
pub mod replace_poa_token_contract_options;

use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct RunOptions {
    #[clap(subcommand)]
    pub command: RunCommand,
}

#[derive(Subcommand, Clone, Debug)]
pub enum RunCommand {
    /// Run storage deposit funding for a list of given tokens using the StorageManagement interface
    FundStorageDeposit(fund_storage_deposit_options::FundStorageDepositOptions),
    /// Replaces the contract code of a list of PoA token contracts. All contracts are expected to be under one PoA factory contract.
    /// The process adds full access key, replaces the contract, then removes the key.
    ReplacePOATokenContract(replace_poa_token_contract_options::ReplacePoATokenContractOptions),
}
