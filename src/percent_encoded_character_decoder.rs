#![warn(clippy::pedantic)]

use std::collections::HashSet;
use std::convert::TryFrom;

// This is the character set containing just numbers.
lazy_static! {
    static ref DIGIT: HashSet<char> =
        ('0'..='9')
        .collect();
}

// This is the character set containing just the upper-case
// letters 'A' through 'F', used in upper-case hexadecimal.
lazy_static! {
    static ref HEX_UPPER: HashSet<char> =
        ('A'..='F')
        .collect();
}

// This is the character set containing just the lower-case
// letters 'a' through 'f', used in lower-case hexadecimal.
lazy_static! {
    static ref HEX_LOWER: HashSet<char> =
        ('a'..='f')
        .collect();
}

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("illegal character")]
    IllegalCharacter,
}

pub struct PercentEncodedCharacterDecoder {
    decoded_character: u8,
    digits_left: usize,
}

impl PercentEncodedCharacterDecoder {
    pub fn new() -> Self {
        Self{
            decoded_character: 0,
            digits_left: 2,
        }
    }

    pub fn next(
        &mut self,
        c: char
    ) -> Result<Option<u8>, Error> {
        self.shift_in_hex_digit(c)?;
        self.digits_left -= 1;
        if self.digits_left == 0 {
            let output = self.decoded_character;
            self.reset();
            Ok(Some(output))
        } else {
            Ok(None)
        }
    }

    fn reset(&mut self) {
        self.decoded_character = 0;
        self.digits_left = 2;
    }

    fn shift_in_hex_digit(
        &mut self,
        c: char
    ) -> Result<(), Error> {
        self.decoded_character <<= 4;
        if let Some(ci) = c.to_digit(16) {
            self.decoded_character += u8::try_from(ci).unwrap();
        } else {
            self.reset();
            return Err(Error::IllegalCharacter);
        }
        Ok(())
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn good_sequences() {
        named_tuple!(
            struct TestVector {
                sequence: [char; 2],
                expected_output: u8,
            }
        );
        let test_vectors: &[TestVector] = &[
            (['4', '1'], b'A').into(),
            (['5', 'A'], b'Z').into(),
            (['6', 'e'], b'n').into(),
            (['e', '1'], b'\xe1').into(),
            (['C', 'A'], b'\xca').into(),
        ];
        for test_vector in test_vectors {
            let mut pec = PercentEncodedCharacterDecoder::new();
            assert_eq!(
                Ok(None),
                pec.next(test_vector.sequence()[0])
            );
            assert_eq!(
                Ok(Some(*test_vector.expected_output())),
                pec.next(test_vector.sequence()[1])
            );
        }
    }

    #[test]
    fn bad_sequences() {
        let test_vectors = [
            'G', 'g', '.', 'z', '-', ' ', 'V',
        ];
        for test_vector in &test_vectors {
            let mut pec = PercentEncodedCharacterDecoder::new();
            assert!(pec.next(*test_vector).is_err());
        }
    }

}
