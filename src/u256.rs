use crate::bytes;
use hex;
use primitive_types::U256 as PU256;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct U256 {
    pub v: PU256,
}

#[derive(Debug, PartialEq, Eq)]
pub struct U256ParseError;

impl FromStr for U256 {
    type Err = U256ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match PU256::from_str_radix(s, 16) {
            Ok(n) => return Ok(Self { v: n }),
            Err(_) => return Err(U256ParseError),
        }
    }
}

impl ToString for U256 {
    fn to_string(&self) -> String {
        let mut bytes = [0; 32];
        self.v.to_big_endian(&mut bytes);
        return hex::encode(bytes);
    }
}

impl U256 {
    /**
     * UTILITIES
     */
    pub fn from_bytes(bs: &[u8]) -> Self {
        assert!(bs.len() <= 32, "big-endian");

        return Self {
            v: PU256::from_big_endian(bs),
        };
    }
    pub fn to_bytes(&self, r: &mut [u8]) {
        self.v.to_big_endian(r);
    }

    pub fn zero() -> Self {
        return Self::from_str("0x0").unwrap();
    }
    pub fn one() -> Self {
        return Self::from_str("0x1").unwrap();
    }

    /**
     * ARITHMETIC
     */

    /// a + b (mod p) = (a mod p + b mod p) mod p
    ///
    /// 0                   p
    /// |___________________|___|_______________________|
    /// |        U256.max       |        U256.max       |
    ///
    /// (a mod p + b mod p) can still result in an overflow since (a mod p + b
    /// mod p) can be greater than U256.max. If this is the case, you would get
    /// a truncated result equal to (a mod p + b mod p) - U256.max - 1.
    ///
    /// Consider the case where (a mod p) = (b mod p) and both are slightly less
    /// than p.  
    ///                          b mod p
    ///                          a mod p
    /// 0                           |  p               (a mod p + b mod p)
    /// |___________________________|__|___|____________________|______________|
    /// |                              | x |  truncated result  |              |
    /// |              U256.max            |              U256.max             |
    ///
    /// The truncated result will not be the correct modulo answer... it is
    /// missing segment x = U256.max - p + 1. So this must be added back.
    pub fn add_mod(&self, b: &Self, p: &Self) -> Self {
        let x1 = self.v.checked_rem(p.v).expect("modulo");
        let x2 = b.v.checked_rem(p.v).expect("modulo");

        // Get truncated result if there is an overflow
        let (mut x3, over) = x1.overflowing_add(x2);

        // If overflows, add back segment x = U256.max - p + 1
        if over {
            x3 = x3
                .checked_add(
                    PU256::MAX
                        .checked_sub(p.v)
                        .expect("sub")
                        .checked_add(PU256::from_big_endian(&[1]))
                        .expect("conversion"),
                )
                .expect("add");
        }

        x3 = x3.checked_rem(p.v).expect("modulo");

        return Self { v: x3 };
    }

    /// a - b (mod p) = (a mod p - b mod p) mod p
    ///
    /// To prevent underflow in case (b mod p) > (a mod p):
    /// = (a mod p + ((p - b) mod p)) mod p
    ///
    /// To prevent underflow in case b > p:
    /// = (a mod p + ((p - (b mod p))) mod p)) mod p
    pub fn sub_mod(&self, b: &Self, p: &Self) -> Self {
        let x1 = self.v.checked_rem(p.v).expect("modulo");
        let x2 = b.v.checked_rem(p.v).expect("modulo");

        return Self { v: x1 }.add_mod(&Self { v: (p.v - x2) }, p);
    }

    /// Uses Add-and-Double algorithm for O(log n) time complexity
    /// Will define multiplication as repeated addition:
    ///
    /// 13 * 10 = 13 + 13 + ... + 13 + 13 (11 times)
    ///
    /// Algorithm would use these steps:
    /// - 0  +  0 + 13 = 13
    /// - 13 + 13      = 26
    /// - 26 + 26 + 13 = 65
    /// - 65 + 65 + 13 = 143
    ///
    /// The algorithm at each step either doubles the previous number, or
    /// doubles the previous number and adds 13. To determine which to do, the
    /// binary representation is required. 11 = 0b1011
    ///
    /// Iterate through the binary string from left to right. If the current bit
    /// is 1, double and add 13. If the current bit is 0, only double.
    ///
    /// *1* - 0  +  0 + 13 = 13
    /// *0* - 13 + 13      = 26
    /// *1* - 26 + 26 + 13 = 65
    /// *1* - 65 + 65 + 13 = 143
    pub fn mul_mod(&self, b: &Self, p: &Self) -> Self {
        let x1 = Self {
            v: self.v.checked_rem(p.v).expect("modulo"),
        };
        let x2 = Self {
            v: b.v.checked_rem(p.v).expect("modulo"),
        };

        let mut base = Self::zero();

        let seq: Self;
        let adder: Self;

        // Assume seq is the smaller of the two factors
        if x1.v < x2.v {
            seq = x1;
            adder = x2;
        } else {
            seq = x2;
            adder = x1;
        }

        let mut seq_bytes = [0; 32];
        seq.to_bytes(&mut seq_bytes);

        let mut seq_binaries: Vec<u8> = vec![];
        bytes::bytes_to_binary(&seq_bytes, &mut seq_binaries);

        // Begin doubling after first 1 bit. Also add the `adder` for every 1
        // bit. Repeated modular addition assures result remains on the finite
        // field
        let mut on = false;
        for d in seq_binaries.into_iter() {
            if on {
                base = base.add_mod(&base, p);
            }
            if d > 0 {
                on = true;
                base = base.add_mod(&adder, p);
            }
        }

        return base;
    }

    /// Will use Square-and-Multiply algorithm for O(log n) time complexity
    /// Similar to the multiplication algorithm above, but instead of repeated
    /// addition, it will be repeated multiplication.
    pub fn exp_mod(&self, e: &Self, p: &Self) -> Self {
        let seq = e;
        let multiplier = U256 {
            v: self.v.checked_rem(p.v).expect("modulo"),
        };

        let mut base = Self::one();

        let mut seq_bytes = [0; 32];
        seq.to_bytes(&mut seq_bytes);

        let mut seq_binaries: Vec<u8> = vec![];
        bytes::bytes_to_binary(&seq_bytes, &mut seq_binaries);

        // Begin squaring after first 1 bit. Also add the `adder` for every 1
        // bit. Repeated modular addition assures result remains on the finite
        // field
        let mut on = false;
        for d in seq_binaries.into_iter() {
            if on {
                base = base.mul_mod(&base, p);
            }
            if d > 0 {
                on = true;
                base = base.mul_mod(&multiplier, p);
            }
        }

        return base;
    }

    /// (a / b) (mod p) = (a * b^-1) (mod p)
    ///
    /// On a finite field, b^(p - 1) = 1:
    /// = (a * b^(p - 1) * b^-1) (mod p)
    /// = (a * b^(p - 2)) (mod p)
    /// = ((a mod p) * (b^(p - 2) mod p)) mod p
    pub fn div_mod(&self, b: &Self, p: &Self) -> Self {
        assert!(p.v >= PU256::from_big_endian(&[2]));
        return self.mul_mod(&b.exp_mod(&U256 { v: p.v - 2 }, p), p);
    }
}

impl PartialEq for U256 {
    fn eq(&self, other: &Self) -> bool {
        return self.v == other.v;
    }
}

#[cfg(test)]
mod tests {
    use crate::u256::U256;
    use std::str::FromStr;

    #[test]
    fn addition_case_1() {
        let a = U256::from_str("0xBD").unwrap();
        let b = U256::from_str("0x2B").unwrap();
        let p = U256::from_str("0xB").unwrap();

        let r = a.add_mod(&b, &p);

        assert_eq!(
            r.to_string(),
            "0000000000000000000000000000000000000000000000000000000000000001"
        );
    }

    #[test]
    fn addition_case_2() {
        let a = U256::from_str("0xa167f055ff75c").unwrap();
        let b = U256::from_str("0xacc457752e4ed").unwrap();
        let p = U256::from_str("0xf9cd").unwrap();

        let r = a.add_mod(&b, &p);

        assert_eq!(
            r.to_string(),
            "0000000000000000000000000000000000000000000000000000000000006bb0"
        );
    }

    #[test]
    fn addition_case_3() {
        let a = U256::from_str("0xa167f055ff75c7f055ff7").unwrap();
        let b = U256::from_str("0x7752acc45e4acc45ed57752e").unwrap();
        let p = U256::from_str("0xf9caf05f05cc45d").unwrap();

        let r = a.add_mod(&b, &p);

        assert_eq!(
            r.to_string(),
            "00000000000000000000000000000000000000000000000006548804e13ad1c2"
        );
    }

    #[test]
    fn subtraction_case_1() {
        let a = U256::from_str("0xa167f055ff75c7f055ff7").unwrap();
        let b = U256::from_str("0x7752acc45e4acc45ed57752e").unwrap();
        let p = U256::from_str("0xf9caf05f05cc45d").unwrap();

        let r = a.sub_mod(&b, &p);

        assert_eq!(
            r.to_string(),
            "00000000000000000000000000000000000000000000000005c0fe76d3e05765"
        );
    }

    #[test]
    fn subtraction_case_2() {
        let a = U256::from_str("0x37ab9cde2a6f51a").unwrap();
        let b = U256::from_str("0x67592e81d48b9e6").unwrap();
        let p = U256::from_str("0x9a8d7f51e").unwrap();

        let r = a.sub_mod(&b, &p);

        assert_eq!(
            r.to_string(),
            "00000000000000000000000000000000000000000000000000000009712a07c4"
        );
    }

    #[test]
    fn multiplication_case_1() {
        let a = U256::from_str("0xa").unwrap();
        let b = U256::from_str("0xd").unwrap();
        let p = U256::from_str("0xabcdef01").unwrap();

        let r = a.mul_mod(&b, &p);

        assert_eq!(
            r.to_string(),
            "0000000000000000000000000000000000000000000000000000000000000082"
        );
    }

    #[test]
    fn multiplication_case_2() {
        let a = U256::from_str("0x7a7b5c6d").unwrap();
        let b = U256::from_str("0x98765432").unwrap();
        let p = U256::from_str("0xabcdef01").unwrap();

        let r = a.mul_mod(&b, &p);

        assert_eq!(
            r.to_string(),
            "000000000000000000000000000000000000000000000000000000009ca42e13"
        );
    }

    #[test]
    fn multiplication_case_3() {
        let a = U256::from_str("0x123456789abcdef").unwrap();
        let b = U256::from_str("0xfedcba9876543210").unwrap();
        let p = U256::from_str("0x2468acf13579bdf").unwrap();

        let r = a.mul_mod(&b, &p);

        assert_eq!(
            r.to_string(),
            "00000000000000000000000000000000000000000000000002468acf13579b9f"
        );
    }

    #[test]
    fn exponentiation_case_1() {
        let a = U256::from_str("0x123456789abcdef").unwrap();
        let b = U256::from_str("0xfedcba9876543210").unwrap();
        let p = U256::from_str("0x2468acf13579bdf").unwrap();

        let r = a.exp_mod(&b, &p);

        assert_eq!(
            r.to_string(),
            "000000000000000000000000000000000000000000000000007c09c4c5916164"
        );
    }

    #[test]
    fn division_case_1() {
        let a = U256::from_str("0x123456789abcdef").unwrap();
        let b = U256::from_str("0xfedcba9876543210").unwrap();
        let p = U256::from_str("0x1a69ea467").unwrap();

        let r = a.div_mod(&b, &p);

        assert_eq!(
            r.to_string(),
            "0000000000000000000000000000000000000000000000000000000124207cf3"
        );
    }
}
