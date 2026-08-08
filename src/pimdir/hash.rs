//! Content hashing for the pimdir blob store.
//!
//! io-replica treats a [`ReplicaHash`] as an opaque, consumer-computed
//! string; it never hashes anything. Bodies are content-addressed so an
//! item present in several places is stored once.
//!
//! This digest MUST match Neverest's, himalaya's and
//! himalaya-android-m3's: a 128-bit FNV-1a variant rendered as 32 hex
//! chars. All four consumers have to agree on object identity, or an
//! item calendula adds will not deduplicate against the same item a
//! sync already stored.

use io_replica::object::ReplicaHash;

/// The content hash of a whole body (32 hex chars).
pub fn content_hash(bytes: &[u8]) -> ReplicaHash {
    let mut a: u64 = 0xcbf2_9ce4_8422_2325;
    let mut b: u64 = 0x9e37_79b9_7f4a_7c15;

    for &byte in bytes {
        a ^= byte as u64;
        a = a.wrapping_mul(0x0000_0100_0000_01b3);
        b = b.wrapping_add(byte as u64);
        b ^= b << 13;
        b = b.wrapping_mul(0xff51_afd7_ed55_8ccd);
    }

    a ^= bytes.len() as u64;
    ReplicaHash::from(format!("{a:016x}{b:016x}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_digest_keeps_the_shape_every_consumer_agreed_on() {
        let hash = content_hash(b"BEGIN:VCALENDAR\r\nVERSION:2.0\r\nEND:VCALENDAR\r\n");

        assert_eq!(hash.0.len(), 32);
        assert!(hash.0.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn the_digest_is_deterministic_and_separates_distinct_bodies() {
        assert_eq!(content_hash(b"same"), content_hash(b"same"));
        assert_ne!(content_hash(b"a"), content_hash(b"b"));

        // The length fold is what keeps two bodies differing only by a
        // trailing byte apart.
        assert_ne!(content_hash(b"ab"), content_hash(b"ab\0"));
    }
}
