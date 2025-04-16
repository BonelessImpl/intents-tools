# How to run

You can see the available commands with:

```
cargo run -- --help
```

## Funding storage deposits for many tokens

You can see the available options with:

```
cargo run -- fund-storage-deposit --help
```

Example for storage deposit:

```
cargo run fund-storage-deposit --tokens-list-file ../tokens.txt --source-account-for-near <account-id-that-pays> --deposit-beneficiary <intents-contract-account-id>
```

#### Tokens list

The tokens should be in the following format:

```
nep141:sol-c800a4bd850783ccb82c2b2c7e84175443606352.omft.near
nep141:arb.omft.near
nep141:doge.omft.near
nep141:arb-0xfd086bc7cd5c481dcc9c85ebe478a1c0b69fcbb9.omft.near
```

## Replacing PoA token contracts

You can see the available options with:

```
cargo run -- replace-poa-token-contract --help
```

First, place `poa-seed.txt` with the seed phrase that contains the seed, whose key will be added (and eventually removed) to PoA token contracts to do the replacement

```
cargo run -- replace-poa-token-contract --poa-factory-account-id poa-factory.example.near --tokens-prefixes-list-file subtokens.txt --source-account-for-action poa-factory.example.near --poa-token-wasm-file defuse_poa_token.wasm
```
