use crate::bytes;
use crate::u256::U256;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct EccPoint {
    pub x: U256,
    pub y: U256,
}

impl EccPoint {
    pub fn from_hex_coordinates(x: &str, y: &str) -> Self {
        return EccPoint {
            x: U256::from_str(x).unwrap(),
            y: U256::from_str(y).unwrap(),
        };
    }

    pub fn to_hex_string(&self) -> String {
        return format!("{} {}", self.x.to_string(), self.y.to_string());
    }

    pub fn is_zero_point(&self) -> bool {
        return self.x == U256::from_str("0x0").unwrap()
            && self.y == U256::from_str("0x0").unwrap();
    }
}

pub struct SECP256K1;

impl SECP256K1 {
    pub fn p() -> U256 {
        return U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEFFFFFC2F")
            .unwrap();
    }

    pub fn g() -> EccPoint {
        return EccPoint {
            x: U256::from_str("79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798")
                .unwrap(),
            y: U256::from_str("483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8")
                .unwrap(),
        };
    }

    pub fn n() -> U256 {
        return U256::from_str("FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141")
            .unwrap();
    }

    pub fn zero_point() -> EccPoint {
        return EccPoint {
            x: U256::from_str("0x0").unwrap(),
            y: U256::from_str("0x0").unwrap(),
        };
    }

    pub fn add_points(pt1: &EccPoint, pt2: &EccPoint) -> EccPoint {
        println!("Adding");
        assert!(pt1.x != pt2.x);

        if pt1.is_zero_point() {
            return pt2.clone();
        }
        if pt2.is_zero_point() {
            return pt1.clone();
        }

        let p = &Self::p();

        // slope calc
        let y_diff = &pt1.y.sub_mod(&pt2.y, p);
        let x_diff = &pt1.x.sub_mod(&pt2.x, p);
        let lambda = &y_diff.div_mod(x_diff, p);

        // calculate new x
        let x3 = &lambda
            .mul_mod(lambda, p)
            .sub_mod(&pt1.x, p)
            .sub_mod(&pt2.x, p);

        // calculate new y
        let y3 = &pt1.x.sub_mod(x3, p).mul_mod(lambda, p).sub_mod(&pt1.y, p);

        return EccPoint {
            x: x3.clone(),
            y: y3.clone(),
        };
    }

    pub fn double_point(pt: &EccPoint) -> EccPoint {
        println!("Doubling");
        if pt.is_zero_point() {
            return Self::zero_point();
        }
        if pt.y == U256::from_str("0x0").unwrap() {
            return Self::zero_point();
        }

        let p = &Self::p();
        let const_2 = &U256::from_str("0x2").unwrap();
        let const_3 = &U256::from_str("0x3").unwrap();

        // slope
        let two_y = &pt.y.mul_mod(const_2, p);
        let x1_2_3 = &pt.x.mul_mod(&pt.x, p).mul_mod(const_3, p);
        let lambda = &x1_2_3.div_mod(two_y, p);

        // calculate new x
        let x3 = &lambda
            .mul_mod(lambda, p)
            .sub_mod(&pt.x, p)
            .sub_mod(&pt.x, p);

        // calculate new y
        let y3 = &pt.x.sub_mod(x3, p).mul_mod(lambda, p).sub_mod(&pt.y, p);

        return EccPoint {
            x: x3.clone(),
            y: y3.clone(),
        };
    }

    pub fn pr_to_pub(pr: &U256) -> EccPoint {
        let mut bytes: [u8; 32] = [0; 32];
        pr.to_bytes(&mut bytes);

        let mut binaries: Vec<u8> = vec![];
        bytes::bytes_to_binary(&bytes, &mut binaries);

        let mut base = Self::zero_point().clone();
        let adder = Self::g().clone();

        let mut on = false;
        let mut step = 0;
        for d in binaries.into_iter() {
            println!("Step: {}", step);
            if on {
                base = Self::double_point(&base);
            }
            if d > 0 {
                on = true;
                base = Self::add_points(&base, &adder);
            }
            step += 1;
        }

        return base;
    }
}

mod tests {
    use crate::secp256k1::*;

    #[test]
    fn secp256k1_add_points() {
        let pt1 = EccPoint::from_hex_coordinates(
            "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
        );
        let pt2 = EccPoint::from_hex_coordinates(
            "C6047F9441ED7D6D3045406E95C07CD85C778E4B8CEF3CA7ABAC09B95C709EE5",
            "1AE168FEA63DC339A3C58419466CEAEEF7F632653266D0E1236431A950CFE52A",
        );
        let pt3 = SECP256K1::add_points(&pt1, &pt2);

        assert_eq!(pt3.to_hex_string(), "f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9 388f7b0f632de8140fe337e62a37f3566500a99934c2231b6cb9fd7584b8e672");
    }

    #[test]
    fn secp256k1_double_point() {
        let pt1 = EccPoint::from_hex_coordinates(
            "79BE667EF9DCBBAC55A06295CE870B07029BFCDB2DCE28D959F2815B16F81798",
            "483ADA7726A3C4655DA4FBFC0E1108A8FD17B448A68554199C47D08FFB10D4B8",
        );

        let pt2 = SECP256K1::double_point(&pt1);
        let pt3 = SECP256K1::double_point(&pt2);

        assert_eq!(pt3.to_hex_string(), "e493dbf1c10d80f3581e4904930b1404cc6c13900ee0758474fa94abe8c4cd13 51ed993ea0d455b75642e2098ea51448d967ae33bfbdfe40cfe97bdc47739922");
    }
}
