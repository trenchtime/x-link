# X-Link
Use x-link to compute a solana keypair from a twitter handle deterministically

## How
1. put in seed phrase
2. put in password
3. computes seed
4. hashes seed with twitter handle
5. derives Keypair from hash

Using the same seed phrase + password + handle combo results in the same Keypair every time.

## Why
Not having to keep keys in a DB is good

## What if the user wants the key
The Keypair can be given to the user.
