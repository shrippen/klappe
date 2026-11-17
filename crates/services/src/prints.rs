//! The fingerprints of the script (`klappe_script::Fingerprint`) as the register stores them
//! (`klappe_model::Print`): `model/` may not know the script crate, so the two meet here.

use klappe_model::Print;
use klappe_script::Fingerprint;

pub fn print_of(fingerprint: Fingerprint) -> Print {
    Print {
        tag: fingerprint.tag,
        hash: fingerprint.hash,
    }
}

pub fn fingerprint_of(print: Print) -> Fingerprint {
    Fingerprint {
        tag: print.tag,
        hash: print.hash,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fingerprint_goes_there_and_back() {
        let fingerprint = Fingerprint { tag: 2, hash: 7 };

        assert_eq!(fingerprint_of(print_of(fingerprint)), fingerprint);
    }
}
