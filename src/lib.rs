
pub trait Hashable {
    fn hash(&self) -> usize;
}

impl Hashable for String {
    //sdbm hash-encoding for a given string
    fn hash(&self) -> usize {
        let mut hash: usize = 0;
        for _c in self.encode_utf16() {
            hash = usize::from(_c)
                .wrapping_add(hash << 6)
                .wrapping_add(hash << 16)
                .wrapping_sub(hash);
        }
        hash
    }
}

impl Hashable for usize {
    fn hash(&self) -> usize {
        *self
    }
}

pub fn hash_string(key: &String) -> usize {
    key.hash()
}
