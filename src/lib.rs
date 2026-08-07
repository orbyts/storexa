//! Storexa is a domain-agnostic persistence foundation for Rust applications.

/// Returns Storexa's initial greeting.
pub const fn hello() -> &'static str {
    "Hello from Storexa!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn says_hello() {
        assert_eq!(hello(), "Hello from Storexa!");
    }
}
