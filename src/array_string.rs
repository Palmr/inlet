use std::fmt::{Debug, Formatter};

const ARRAY_STRING_SIZE: usize = 128;

#[repr(C)]
#[derive(Eq, PartialEq)]
pub struct ArrayString {
    bytes: [u8; ARRAY_STRING_SIZE],
}

impl ArrayString {
    pub const fn empty() -> ArrayString {
        ArrayString {
            bytes: [0; ARRAY_STRING_SIZE],
        }
    }
    pub fn is_empty(&self) -> bool {
        self.bytes[0] == 0
    }
}

impl From<String> for ArrayString {
    fn from(value: String) -> Self {
        let mut bytes = [0u8; ARRAY_STRING_SIZE];
        assert!(
            value.len() <= ARRAY_STRING_SIZE,
            "ArrayString must be less than or equal to {ARRAY_STRING_SIZE} characters."
        );
        bytes[..value.len()].copy_from_slice(value.as_bytes());
        Self { bytes }
    }
}

impl Debug for ArrayString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"")?;
        for byte in self.bytes {
            if byte == 0 {
                break;
            }
            write!(f, "{}", byte as char)?;
        }
        write!(f, "\"")
    }
}

impl ToString for ArrayString {
    fn to_string(&self) -> String {
        let mut x = String::new();
        for byte in self.bytes {
            if byte == 0 {
                break;
            }
            x.push(byte as char);
        }
        x
    }
}

#[cfg(test)]
mod array_string_tests {
    use crate::array_string::ArrayString;

    #[test]
    fn test_array_string_debug() {
        let a = ArrayString::from(String::from("Hello, World!"));
        let debug = format!("{:?}", a);
        assert_eq!("\"Hello, World!\"", debug);
    }

    #[test]
    fn test_array_string_inner() {
        let a = ArrayString::from(String::from("Hello, World!"));
        let mut expected = [0u8; 128];
        expected[..13]
            .copy_from_slice(&[72, 101, 108, 108, 111, 44, 32, 87, 111, 114, 108, 100, 33]);
        assert_eq!(a.bytes, expected);
    }

    #[test]
    #[should_panic]
    fn test_array_string_fails_to_convert_long_string() {
        let _ = ArrayString::from(String::from("This is a very long string that will cause the conversion method to panic as it is exactly one character too long for the method."));
    }
}
