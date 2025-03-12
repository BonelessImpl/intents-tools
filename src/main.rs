mod run_options;
mod storage_deposit;

use clap::Parser;
use run_options::RunOptions;

fn main() -> anyhow::Result<()> {
    let args = RunOptions::parse();

    match args.command {
        run_options::RunCommand::FundStorageDeposit(ops) => storage_deposit::run(ops),
    }
}
