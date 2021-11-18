use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long)]
    plaintext: bool,

    #[structopt(parse(from_os_str))]
    path_in: PathBuf,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let opt = Opt::from_args();

    let scenario = if opt.plaintext {
        let buf = std::fs::read_to_string(opt.path_in)?;
        javardry_spoiler::Scenario::load_from_plaintext(buf)?
    } else {
        let buf = std::fs::read(opt.path_in)?;
        javardry_spoiler::Scenario::load_from_ciphertext(buf)?
    };

    dbg!(&scenario);

    Ok(())
}
