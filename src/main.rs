mod replace_poa_contract;
mod run_options;
mod storage_deposit;
mod utils;

use clap::Parser;
use run_options::RunOptions;

fn main() -> anyhow::Result<()> {
    let args = RunOptions::parse();

    match args.command {
        run_options::RunCommand::FundStorageDeposit(opts) => storage_deposit::run(opts),
        run_options::RunCommand::ReplacePOATokenContract(opts) => replace_poa_contract::run(opts),
    }
}
