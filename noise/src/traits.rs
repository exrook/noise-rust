/// A trait for fixed size u8 array.

// Inspired by ArrayVec and SmallVec, but no unsafe.

// Use this trait so that we don't have to use `Vec` for some semi-fixed length buffers and
// input/output types.
pub trait U8Array {
    /// Create a new array filled with all zeros.
    fn new() -> Self;
    /// Create a new array filled with a same value.
    fn new_with(u8) -> Self;
    /// Create a new array from a slice.
    ///
    /// # Panics
    ///
    /// The slice must be of the same length.
    fn from_slice(&[u8]) -> Self;
    /// Length of the array.
    fn len() -> usize;
    /// As slice.
    fn as_slice(&self) -> &[u8];
    /// As mutable slice.
    fn as_mut(&mut self) -> &mut [u8];
}

macro_rules! impl_array {
    ($len:expr) => {
        impl U8Array for [u8; $len] {
            fn new() -> Self {
                [0u8; $len]
            }

            fn new_with(x: u8) -> Self {
                [x; $len]
            }

            fn from_slice(data: &[u8]) -> Self {
                let mut a = [0u8; $len];
                a.copy_from_slice(data);
                a
            }

            fn len() -> usize {
                $len
            }

            fn as_slice(&self) -> &[u8] {
                self
            }

            fn as_mut(&mut self) -> &mut [u8] {
                self
            }
        }
    }
}

impl_array!(32);
impl_array!(64);
impl_array!(128);

/// A DH.
pub trait DH {
    /// Type of private key.
    type Key: U8Array + Clone;
    /// Type of pubkey key.
    type Pubkey: U8Array + Clone;
    /// Type of output.
    type Output: U8Array + Clone;

    /// Name of this DH function, e.g., “25519”.
    fn name() -> &'static str;

    /// Randomly generate a new private key.
    fn genkey() -> Self::Key;

    /// Calculate public key from a private key.
    fn pubkey(&Self::Key) -> Self::Pubkey;

    /// Perform DH key exchange.
    fn dh(&Self::Key, &Self::Pubkey) -> Self::Output;
}

/// An AEAD.
pub trait Cipher {
    /// Name of this cipher function.
    fn name() -> &'static str;
    /// Type of key.
    type Key: U8Array + Clone;

    /// Length of key.
    fn key_len() -> usize {
        Self::Key::len()
    }

    /// Length of auth tag.
    ///
    /// All ciphers specified in the spec has tag length 16.
    fn tag_len() -> usize {
        16
    }

    /// AEAD encryption.
    ///
    /// out.len() == plaintext.len() + Self::tag_len()
    fn encrypt(k: &Self::Key, nonce: u64, ad: &[u8], plaintext: &[u8], out: &mut [u8]);

    /// AEAD decryption.
    ///
    /// out.len() == ciphertext.len() - Self::tag_len()
    fn decrypt(k: &Self::Key,
               nonce: u64,
               ad: &[u8],
               ciphertext: &[u8],
               out: &mut [u8])
               -> Result<(), ()>;
}

/// A hash function.
pub trait Hash: Default {
    /// Name of the hash function.
    fn name() -> &'static str;

    /// Type of a block.
    type Block: U8Array;
    /// Type of output.
    type Output: U8Array + Copy;

    /// Length of block.
    fn block_len() -> usize {
        Self::Block::len()
    }

    /// Length of hash output, in number of bytes.
    fn hash_len() -> usize {
        Self::Output::len()
    }

    /// Reset state of hash context.
    fn reset(&mut self) {
        *self = Default::default();
    }

    /// Update hash context with some input.
    fn input(&mut self, data: &[u8]);

    /// Get hash result.
    fn result(&mut self) -> Self::Output;

    /// Calculate hash of some data.
    fn hash(data: &[u8]) -> Self::Output {
        let mut h: Self = Default::default();
        h.input(data);
        h.result()
    }

    /// Calculate HMAC-THIS-HASH, with some `key` and several messages.
    fn hmac_many(key: &[u8], data: &[&[u8]]) -> Self::Output {
        assert!(key.len() <= Self::block_len());

        let mut ipad = Self::Block::new_with(0x36u8);
        let mut opad = Self::Block::new_with(0x5cu8);

        for count in 0..key.len() {
            ipad.as_mut()[count] ^= key[count];
            opad.as_mut()[count] ^= key[count];
        }

        let mut hasher: Self = Default::default();
        hasher.input(ipad.as_slice());
        for d in data {
            hasher.input(d);
        }
        let inner_output = hasher.result();

        hasher.reset();
        hasher.input(opad.as_slice());
        hasher.input(inner_output.as_slice());
        hasher.result()
    }

    /// Calculate HMAC-THIS-HASH, with some `key` and a message.
    fn hmac(key: &[u8], data: &[u8]) -> Self::Output {
        Self::hmac_many(key, &[data])
    }

    /// Calculate HKDF, as specified in the noise spec.
    fn hkdf(chaining_key: &[u8], input_key_material: &[u8]) -> (Self::Output, Self::Output) {
        let temp_key = Self::hmac(chaining_key, input_key_material);
        let out1 = Self::hmac(temp_key.as_slice(), &[1u8]);
        let out2 = Self::hmac_many(temp_key.as_slice(), &[out1.as_slice(), &[2u8]]);
        (out1, out2)
    }
}
