use eccsecp256k1::crypto::ethereum::check_sum;

fn main() {
    println!(
        "{}",
        check_sum("0xA4FEAf73e6dC6D085e990B55F7110aee3a2a871c")
    );
}
