# termcrypt

![](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
<img src="http://ForTheBadge.com/images/badges/built-with-love.svg" height="26"/>

![](https://img.shields.io/badge/license-AGPL--3.0%2B-green)
![](https://img.shields.io/badge/speed-like%20sonic-blue)

A terminal-based interface for efficiently trading the market.

Exchanges termcrypt currently supports:

- [x] FTX
- [ ] Bybit

### License

As developers, we believe that accountability leads to credibility and without the codebase being open, others cannot gain full trust. In addition, proprietary software does not allow external creativity and contribution which stunts unfunded smaller projects. We also believe that there should be no barrier to to market and it should be free for everybody no matter their current financial circumstances.

Because of this, we chose the AGPL-3.0-or-later license because it is one of the strongest copyleft licenses available.

**The APGL-3+ requires that termcrypt can not be used in any proprietary solution.**

### Build
This project requires `rustc` and `cargo`

Clone the repo and open the directory:
```sh
git clone [repo url] termcrypt
cd termcrypt
```

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