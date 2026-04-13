#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub key_bytes: Vec<u8>,
    pub value_bytes: Vec<u8>,
}
