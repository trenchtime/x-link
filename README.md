# X-Link
A collection of crates to force twitter users to gamble on Solana

## Crates
- [x-link-wallet](/crates/x-link-wallet): Deterministically create a Solana wallet from a twitter ID.
- [x-link-client](/crates/x-link-client): Spin up a local HTTP client to interact with the Solana blockhain.
- [x-link-types](/crates/x-link-types): Shared types between the client and wallet crates.
- [x-link-utils](/crates/x-link-utils): Utility functions shared between the client and wallet crates.
- [x-link-solana](/crates/x-link-solana): Solana backend functionality

## X-Link Wallet
Links a twitter account to a Solana wallet.
This is done by generating a Solana wallet from a HD wallet seed and using the twitter ID as the `account` and `change`.
A twitter user ID is a u64, and `account` and `change` are both u32, so this works until Twitter fucks us.

**This is a one way function**
**The wallet can be generated deterministically from the twitter ID, but the twitter ID cannot be generated from the wallet.**

## X-Link Client
A local HTTP client to interact with the Solana blockchain.

### Usage
To securely generate Solana wallets for twitter users, follow these steps:
1. Make sure the secret file is somewhere available to the program.
2. Make sure you know the passphrase
3. Run
```bash
cargo run --release --bin x-link-client -- --secret-file <path-to-secret-file> --port <port>
```
4. You will be prompted to enter the passphrase

**There will be no confirmation that the passphrase is correct, the program will use the input passphrase to generate wallets.**
**For any production environment, TEST with a known input and output to see if hte program is working correctly.**

### RPC Methods
- **getAccount** - Get the account information for a given twitter id.
- **createPool** - WIP
- **buy** - WIP
- **sell** - WIP

#### Note on logging
If you want logging, make sure to specify the crates you want logging for in the `RUST_LOG` environment variable.
For example:
```bash
export RUST_LOG=x_link_client=debug,x_link_solana=debug
```
With just `RUST_LOG=debug`, you will get all logs from all crates (including dependencies).
