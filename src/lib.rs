//!
//! # Examples
//!
//!
//!```
//!use log::{info, log_enabled, Level};
//!
//!extern crate env_logger;
//!
//!use dotenv::dotenv;
//!
//!use lichessbot::lichessbot::*;
//!
//!#[tokio::main]
//!async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!    dotenv().ok();
//!    env_logger::init();
//!
//!    let bot = Box::leak(Box::new(
//!        LichessBot::new()
//!            .uci_opt("Move Overhead", 500)
//!            .uci_opt("Threads", 4)
//!            .uci_opt("Hash", 128)
//!            .uci_opt("Contempt", -25)
//!            .enable_classical(false)
//!            .enable_rapid(false)
//!            .disable_blitz(false)
//!            .disable_bullet(false)
//!            .enable_ultrabullet(false)
//!            .enable_casual(true)
//!            .disable_rated(false),
//!    ));
//!
//!    if log_enabled!(Level::Info) {
//!        info!("starting bot stream");
//!    }
//!
//!    let (tx, mut rxa) = bot.stream().await;
//!
//!    tokio::time::sleep(tokio::time::Duration::from_millis(120000)).await;
//!
//!    let _ = tx.send("stopped by user".to_string()).await;
//!
//!    let result = rxa.recv().await;
//!
//!    if log_enabled!(Level::Info) {
//!        info!("stop stream result {:?}", result);
//!    }
//!
//!    Ok(())
//!}
//!
//!```


// lib
pub mod lichessbot;
