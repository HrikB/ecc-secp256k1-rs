pub mod hashing {
    use crate::base16;
    use sha3::{Digest, Keccak256};

    pub fn hash_keccak256(input: &[u8]) -> String {
        let mut hasher = Keccak256::default();
        hasher.update(input);
        let out = hasher.finalize();
        let r = base16::encode_bytes(&out).to_uppercase();
        return r;
    }

    pub fn hash_keccak256_str(input: &String) -> String {
        let mut hasher = Keccak256::default();
        hasher.update(input.clone().into_bytes());
        let out = hasher.finalize();
        let r = base16::encode_bytes(&out).to_uppercase();
        return r;
    }
}

pub mod ethereum {
    use crate::base16;
    use crate::crypto::hashing::{hash_keccak256, hash_keccak256_str};
    use std::u8;

    pub fn derive_address(pub_key: &str) -> String {
        let pub_key_x = String::from(&pub_key[2..66]).to_uppercase();
        let pub_key_y = String::from(&pub_key[66..130]).to_uppercase();

        let origin = format!("{}{}", pub_key_x, pub_key_y);
        let uncompressed_pub_hash = hash_keccak256(&base16::decode_string(&origin));

        let non_check_summed_address = format!("0x{}", &uncompressed_pub_hash[24..64]).to_lowercase();

        let address = check_sum(&non_check_summed_address);

        return address;
    }

    /// Compare non-checksummed address with the first 40 characters of the hash
    /// of the non-checksummed address. If the hex nibble is greater than 8,
    /// capitalize it (only applies to numbers), otherwise lowercase it.
    pub fn check_sum(address: &str) -> String {
        assert!(address.len() == 42);

        let ad = String::from(&address[2..]).to_lowercase();
        let h = hash_keccak256_str(&ad);

        let r: String = ad
            .chars()
            .zip(h.chars())
            .map(|(c, flag)| {
                if c.is_alphabetic()
                    && u8::from_str_radix(flag.to_string().as_str(), 16).unwrap() > 8
                {
                    c.to_ascii_uppercase()
                } else {
                    c
                }
            })
            .collect();

        return format!("0x{}", r);
    }
}

pub mod secp256k1_prod {
    use crate::base16;
    use secp256k1::{PublicKey, Secp256k1, SecretKey};
    use std::str::FromStr;

    pub fn get_public_key(pr: &str) -> String {
        let secp = Secp256k1::new();
        let pr_key = SecretKey::from_str(pr).expect("private-key");
        let pub_key = PublicKey::from_secret_key(&secp, &pr_key);
        return base16::encode_bytes(&pub_key.serialize_uncompressed());
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::ethereum;
    use crate::crypto::secp256k1_prod as secp256k1;

    #[test]
    fn ethereum_check_sum() {
        let ad = String::from("0xfb6916095ca1df60bb79ce92ce3ea74c37c5d359");
        let r = ethereum::check_sum(&ad);

        let e = String::from("0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359");
        assert_eq!(e, r);
    }

    #[test]
    fn ethereum_address() {
        let pr_n = "51bb0a7f49284110c62e4268baa3cfad4a81edcd6e6ec3b2a8ef97f1e3754491";
        let pub_key = secp256k1::get_public_key(pr_n);
        let r = ethereum::derive_address(&pub_key);

        let e = "0x7aa6D878Ac2d1271fCD010802f7e09fAcd8528bf";
        assert_eq!(e, r);
    }
}
