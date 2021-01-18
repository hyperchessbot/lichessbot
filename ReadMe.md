# lichessbot

[![documentation](https://docs.rs/lichessbot/badge.svg)](https://docs.rs/lichessbot) [![Crates.io](https://img.shields.io/crates/v/lichessbot.svg)](https://crates.io/crates/lichessbot) [![Crates.io (recent)](https://img.shields.io/crates/dr/lichessbot)](https://crates.io/crates/lichessbot)

Lichess bot. Under construction.

# Usage

```rust
extern crate env_logger;

use dotenv::dotenv;

use lichessbot::lichessbot::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	dotenv().ok();
	env_logger::init();

	let mut bot = LichessBot::new()
		.uci_opt("Move Overhead", 500)
		.uci_opt("Threads", 4)
		.uci_opt("Hash", 128)
		.uci_opt("Contempt", -25)
		.enable_classical(false)
		.enable_rapid(false)
		.disable_blitz(false)
		.enable_ultrabullet(false)
		.enable_casual(true)
		.disable_rated(false)
	;

	bot.stream().await
}

```

# Logging

```bash
export RUST_LOG=info
# or 
export RUST_LOG=debug
```

# Config

Set environment as follows:

## Token

`RUST_BOT_TOKEN={lichess API token with bot scopes}`

## Bot name

`RUST_BOT_NAME={bot lichess username}`

example

`RUST_BOT_NAME=chesshyperbot`

## Engine name ( optional )

`RUST_BOT_ENGINE_NAME={engine executable name}`

examples

Linux `RUST_BOT_ENGINE_NAME=./stockfish12`

Windows `RUST_BOT_ENGINE_NAME=stockfish12.exe`

If no engine name is provided, random moves will be played.