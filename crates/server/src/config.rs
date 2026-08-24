use std::{env, net::SocketAddr};

use anyhow::{Context, Result};

const DEFAULT_BIND_ADDRESS: &str = "127.0.0.1:8080";
const DEFAULT_DATABASE_URL: &str = "postgres://omarchy_bbs:omarchy_bbs@127.0.0.1:5432/omarchy_bbs";

#[derive(Clone, Debug)]
pub struct Config {
    pub bind_address: SocketAddr,
    pub database_url: String,
}

impl Config {
    pub fn from_environment() -> Result<Self> {
        let bind_address = env::var("BBS_BIND_ADDRESS")
            .unwrap_or_else(|_| DEFAULT_BIND_ADDRESS.to_owned())
            .parse()
            .context("BBS_BIND_ADDRESS must be a socket address such as 127.0.0.1:8080")?;

        let database_url =
            env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned());

        Ok(Self {
            bind_address,
            database_url,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_BIND_ADDRESS, DEFAULT_DATABASE_URL};

    #[test]
    fn development_defaults_are_local_only() {
        assert!(DEFAULT_BIND_ADDRESS.starts_with("127.0.0.1:"));
        assert!(DEFAULT_DATABASE_URL.contains("@127.0.0.1:"));
    }
}
