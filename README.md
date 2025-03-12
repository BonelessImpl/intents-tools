## How to run

### Funding storage deposits for many tokens

You can see the available commands with:

```
cargo run -- --help
```

Example for storage deposit:

```
cargo run fund-storage-deposit --tokens-list-file ../tokens.txt --source-account-for-near <account-id-that-pays> --deposit-beneficiary <intents-contract-account-id>
```

### Tokens list

The tokens should be in the following format:

```
nep141:sol-c800a4bd850783ccb82c2b2c7e84175443606352.omft.near
nep141:arb.omft.near
nep141:doge.omft.near
nep141:arb-0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9.omft.near
```
