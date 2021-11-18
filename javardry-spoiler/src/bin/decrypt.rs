use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    path_in: PathBuf,

    #[structopt(parse(from_os_str))]
    path_out: PathBuf,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let opt = Opt::from_args();

    let ciphertext = std::fs::read(opt.path_in)?;

    let plaintext = javardry_spoiler::cipher::decrypt(ciphertext)?;

    std::fs::write(opt.path_out, plaintext)?;

    Ok(())
}
