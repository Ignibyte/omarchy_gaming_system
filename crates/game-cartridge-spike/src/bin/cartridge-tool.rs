use std::{env, path::Path};

use anyhow::{Context, Result, bail};
use omarchygs_game_cartridge_spike::{
    PUBLISHER_KEY_ID, generate_key_pair, load_signing_key, load_verifying_key, sign_cartridge,
    verify_cartridge,
};

fn main() -> Result<()> {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match arguments.as_slice() {
        [command, private, public] if command == "keygen" => {
            generate_key_pair(Path::new(private), Path::new(public))
                .context("failed to generate proof key pair")?;
            println!("generated private={} public={}", private, public);
        }
        [command, directory, private] if command == "sign" => {
            let signing_key =
                load_signing_key(Path::new(private)).context("failed to load publisher key")?;
            let digest = sign_cartridge(Path::new(directory), PUBLISHER_KEY_ID, &signing_key)
                .context("failed to sign cartridge")?;
            println!("{digest}");
        }
        [command, directory, public] if command == "verify" => {
            let verifying_key =
                load_verifying_key(Path::new(public)).context("failed to load publisher key")?;
            let cartridge =
                verify_cartridge(Path::new(directory), PUBLISHER_KEY_ID, &verifying_key)
                    .context("failed to verify cartridge")?;
            println!("{}", cartridge.digest);
        }
        _ => bail!(
            "usage: cartridge-tool keygen <private> <public> | sign <directory> <private> | verify <directory> <public>"
        ),
    }
    Ok(())
}
