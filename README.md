# Decentralized Liquidity Pools for Centralized Exchanges(CEX) - Mpesa Case Study

[Mpesa](https://www.safaricom.co.ke/personal/m-pesa/m-pesa-home) money transfer service has made it possible to conveniently send money like sending a quick SMS. With a good number of 30 million active monthly subscribers in Kenya, Mpesa has a great potential of enhancing innovative, richer and Lord-knows-what financial practices.


## How Mpesa Works
Deposit your hard cash money through an M-pesa agent in any viable M-pesa stand. The corresponding deposited amount reflects in your m-pesa e-wallet(for free). You can then send money to other m-pesa/third-party entities like Banks, purchase internet bundles, etc.



You can also withdraw your money for hard cash from a valid mpesa agent at corresponding transaction costs. However, there are common situations when an agent does not have enough or no capital liquidity provision.

```
For a successful withdrawal/deposit transaction, an agent needs a corresponding liquidity amount of the relative deposits/withdrawals needed to conduct the trade.
```

Read this ["Hakuna float"](https://biasharapoint.com/blog/m-pesa-demystifying-m-pesa-hakuna-float-meaning-biasharapoint-blog/) (meaning no liquidity capital) short piece.

## A Decentralized Liquidity Pool for Mpesa
An on-chain liquidity Pool has the potential to solve the liquidity provision problem for Mpesa agents. With a locked pool of liqudity funds, agents can economically outsource liquidity funds to enhance further end-user deposit M-pesa services.

### Prospective consequences
- Enhanced capital efficiency
- Increased trades
- Passive investment for Liquidity providers
- Decentralized funding
- Further DEFI innovations


## Project Implementation
The solution utilizes [Substrate Node](https://github.com/substrate-developer-hub/substrate-node-template) to set up the block chain infrastructure, and FRAME pallets to configure and implement the business runtime logic.

IPFS tool achieves an objective of content addressing hashed liquidity pool funding and Mpesa transactions.

## Help
CLI command to explore all parameters and
subcommands:

```sh
./target/release/node-template -h
```

## Run

`cargo run` command launches a temporary node and its state will be discarded after
you terminate the process. After the project has been built, there are other ways to launch the
node.

### Single-Node Development Chain

This command will start the single-node development chain with non-persistent state:

```bash
./target/release/node-template --dev
```

Purge the development chain's state:

```bash
./target/release/node-template purge-chain --dev
```

Start the development chain with detailed logging:

```bash
RUST_BACKTRACE=1 ./target/release/node-template -ldebug --dev
```

### Run in Docker

First, install [Docker](https://docs.docker.com/get-docker/) and
[Docker Compose](https://docs.docker.com/compose/install/).

Then run the following command to start a single node development chain.

```bash
./scripts/docker_run.sh
```

This command will firstly compile your code, and then start a local development network. You can
also replace the default command
(`cargo build --release && ./target/release/node-template --dev --ws-external`)
by appending your own. A few useful ones are as follow.

```bash
# Run Substrate node without re-compiling
./scripts/docker_run.sh ./target/release/node-template --dev --ws-external

# Purge the local dev chain
./scripts/docker_run.sh ./target/release/node-template purge-chain --dev

# Check whether the code is compilable
./scripts/docker_run.sh cargo check
```
