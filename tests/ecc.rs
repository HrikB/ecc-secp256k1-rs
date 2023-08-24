use std::str::FromStr;

use eccsecp256k1::{base16::*, secp256k1::*, u256::U256};

use rand::prelude::*;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

#[test]
// #[ignore]
fn ecc() {
    // generate a random private key
    let mut rng = rand::thread_rng();
    let bytes = [0; 32];
    let random_bytes = bytes
        .into_iter()
        .map(|_| rng.gen_range(0..=255))
        .collect::<Vec<u8>>();
    let pr_n = hex::encode(random_bytes);

    // generate public key with custom-wrote curve arithmetics
    let pub_key1 = SECP256K1::pr_to_pub(&U256::from_str(&pr_n).unwrap());
    let mut pub_key_str1 = pub_key1.to_hex_string();
    pub_key_str1.retain(|c| !c.is_whitespace());
    pub_key_str1 = "04".to_owned() + &pub_key_str1;

    // generate public key with production library
    let secp = Secp256k1::new();
    let pr_key = SecretKey::from_str(&pr_n).expect("private-key");
    let pub_key2 = PublicKey::from_secret_key(&secp, &pr_key);
    let pub_key_str2 = encode_bytes(&pub_key2.serialize_uncompressed());

    assert_eq!(pub_key_str1, pub_key_str2);
}
