use block_modes::{block_padding::Pkcs7, BlockMode, Ecb};
use des::Des;
use md5::{Digest as _, Md5};

type DesEcb = Ecb<Des, Pkcs7>;

const PASSWORD: &[u8] = b"MadPoet";

pub fn decrypt(ciphertext: impl AsRef<[u8]>) -> anyhow::Result<String> {
    let ciphertext = ciphertext.as_ref();

    let key = make_key(PASSWORD);
    let cipher = DesEcb::new_from_slices(&key, Default::default())?;

    let plaintext = cipher.decrypt_vec(ciphertext)?;

    let plaintext = String::from_utf8(plaintext)?;

    Ok(plaintext)
}

fn make_key(password: &[u8]) -> [u8; 8] {
    let digest = {
        let mut hasher = Md5::new();
        hasher.update(password);
        hasher.finalize()
    };

    digest[..8].try_into().expect("slice length should be 8")
}
