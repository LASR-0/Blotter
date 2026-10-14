/// A new ULID as a 26-character string. ULIDs sort by creation time, need no
/// central counter, and stay unique across devices (for post-v1 sync).
pub fn new_id() -> String {
    ulid::Ulid::generate().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_ulids() {
        let a = new_id();
        let b = new_id();
        assert_eq!(a.len(), 26);
        assert_ne!(a, b);
    }
}
