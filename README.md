## How to run

You can see the available commands with:

```
cargo run -- --help
```

Example for storage deposit:

```
cargo run fund-storage-deposit --tokens-list-file ../tokens.txt --source-account-for-near <account-id-that-pays> --deposit-beneficiary <intents-contract-account-id>
```
