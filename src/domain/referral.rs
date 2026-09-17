use rand::Rng;

/// Characters used for referral codes: uppercase letters and digits, with
/// visually-ambiguous characters (0/O, 1/I) removed to keep codes easy for a
/// human to read back or type in.
const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
const CODE_LEN: usize = 8;

/// Generate a short, random, uppercase alphanumeric referral code (8 chars by
/// default). Not guaranteed unique at the type level — see the repo layer for
/// the known limitation that uniqueness is not enforced by the database in
/// this iteration.
pub fn generate_referral_code() -> String {
    let mut rng = rand::thread_rng();
    (0..CODE_LEN)
        .map(|_| {
            let idx = rng.gen_range(0..ALPHABET.len());
            ALPHABET[idx] as char
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_code_of_expected_length_and_alphabet() {
        let code = generate_referral_code();
        assert_eq!(code.len(), CODE_LEN);
        assert!(code.chars().all(|c| ALPHABET.contains(&(c as u8))));
    }

    #[test]
    fn generates_different_codes_across_calls() {
        let a = generate_referral_code();
        let b = generate_referral_code();
        // Not a uniqueness guarantee, just a sanity check that we're not
        // returning a constant.
        assert_ne!(a, b);
    }
}
