<img src="src/img/logo.png" height="120"/>

# termcrypt

![](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
<img src="http://ForTheBadge.com/images/badges/built-with-love.svg" height="26"/>

![](https://img.shields.io/badge/license-AGPL--3.0%2B-green)
![](https://img.shields.io/badge/speed-like%20sonic-blue)

A terminal-based interface for efficiently trading the market.

Planned Exchanges termcrypt currently supports:

- [x] FTX
- [ ] Bybit

Right now the project is in very early development, so to test it you will most likely have to clone the source and will have to follow the steps on the build section below.

### Features

termcrypt has many utilities that are growing day by day that help interact with the market. Some of the features include:

- [x] Multiple exchange support (in future)
- [x] Fast Speed (Only delays are API requests)
- [x] Subaccounts
- [x] Get Price of pairs
- [x] Get balances
- [x] Search for pairs
- [x] Get and change account leverage
- [x] Calculate risk % into ordersize
- [x] Open longs or shorts in market or auto limit/stop
- [x] (TPSL, fees, % of sub liquidity) when confirming order
- [x] Keybinds (Up and Down to browse command history)
- [x] Dynamic terminal size detection

For a more comprehensive list, run `help` or `h` inside of termcrypt.

Kanban of planned features will be coming soon.

<img src="src/img/example.png" height="80"/>

### License

As developers, we believe that accountability leads to credibility and without the codebase being open, others cannot gain full trust. In addition, proprietary software does not allow external creativity and contribution which stunts unfunded smaller projects. We also believe that there should be no barrier to to market and it should be free for everybody no matter their current financial circumstances.

Because of this, we chose the AGPL-3.0-or-later license because it is one of the strongest copyleft licenses available.

**The APGL-3+ requires that termcrypt can not be used in any proprietary solution.**

### Build
Following commands require `git`, `rustc` and `cargo`

Clone the repo and open the directory:
```sh
git clone [repo url] termcrypt
cd termcrypt
```

Setup the API and then

Run / Build (for your os) with cargo:
```sh
cargo run
cargo build
```

Build files will be located in `target/debug`

## Setup API

Currently you have to create a file called .env in the project folder.
Then enter information as follows (without square brackets):

```
RUST_LOG=info
API_KEY=[public ftx api key]
API_SECRET=[private ftx api secret]
```

It will definitely change in the future to a database system instead.