# lichessbot

[![documentation](https://docs.rs/lichessbot/badge.svg)](https://docs.rs/lichessbot) [![Crates.io](https://img.shields.io/crates/v/lichessbot.svg)](https://crates.io/crates/lichessbot) [![Crates.io (recent)](https://img.shields.io/crates/dr/lichessbot)](https://crates.io/crates/lichessbot)

Lichess bot. Under construction.

# Usage

```rust
extern crate env_logger;

use dotenv::dotenv;

use futures_util::TryStreamExt;
use licorice::client::{Lichess};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
	dotenv().ok();
	env_logger::init();
	
	let lichess = Lichess::default();

	 let query_params = vec![
        ("max", "10"),        
    ];

    let mut stream = lichess
        .export_all_games_json("chesshyperbot", Some(&query_params))
        .await?;

    while let Some(game) = stream.try_next().await? {
    	println!("{:?}", game);
    }
	
	Ok(())
}

```

# Logging

```bash
export RUST_LOG=info
# or 
export RUST_LOG=debug
```
