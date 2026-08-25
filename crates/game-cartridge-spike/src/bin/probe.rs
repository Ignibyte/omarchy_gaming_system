use std::{env, time::Duration};

use anyhow::{Context, Result, bail};
use omarchygs_game_cartridge_spike::ProofResponse;
use reqwest::redirect::Policy;

#[tokio::main]
async fn main() -> Result<()> {
    let broker_url = env::args()
        .nth(1)
        .context("usage: probe <http://loopback:port/v1/proof>")?;
    let client = reqwest::Client::builder()
        .no_proxy()
        .redirect(Policy::none())
        .timeout(Duration::from_secs(10))
        .build()
        .context("failed to build proof client")?;
    let response = client
        .post(broker_url)
        .header("content-type", "application/json")
        .body("{}")
        .send()
        .await
        .context("broker proof request failed")?
        .error_for_status()
        .context("broker proof request was rejected")?;
    let proof = response
        .json::<ProofResponse>()
        .await
        .context("broker proof response was invalid")?;
    if proof.status != "ready"
        || proof.revision != 1
        || !proof.idempotent_replay
        || !proof.duplicate_event_rejected
        || proof.raw_persona_disclosed
        || proof.device_token_disclosed
        || proof.database_access_disclosed
        || !proof.pairwise_subject_verified
        || proof.presentation.screens.is_empty()
        || proof.view.board.first().map(String::as_str) != Some("X")
    {
        bail!("proof invariants failed: {proof:?}");
    }
    println!("{}", serde_json::to_string_pretty(&proof)?);
    Ok(())
}
