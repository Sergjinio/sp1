use core::fmt::Debug;
use core::mem::size_of;
use std::borrow::Borrow;
use std::borrow::BorrowMut;

use itertools::Itertools;
use p3_field::{AbstractField, PrimeField32};
use serde::{Deserialize, Serialize};

use super::Word;
use crate::stark::PROOF_MAX_NUM_PVS;

/// The number of non padded elements in the SP1 proofs public values vec.
pub const SP1_PROOF_NUM_PV_ELTS: usize = size_of::<PublicValues<Word<u8>, u8>>();

/// The number of 32 bit words in the SP1 proof's commited value digest.
pub const PV_DIGEST_NUM_WORDS: usize = 8;

pub const POSEIDON_NUM_WORDS: usize = 8;

/// The PublicValues struct is used to store all of a shard proof's public values.
#[derive(Serialize, Deserialize, Clone, Copy, Default, Debug)]
#[repr(C)]
pub struct PublicValues<W, T> {
    /// The hash of all the bytes that the guest program has written to public values.
    pub committed_value_digest: [W; PV_DIGEST_NUM_WORDS],

    /// The hash of all deferred proofs that have been witnessed in the VM. It will be rebuilt in
    /// recursive verification as the proofs get verified. The hash itself is a rolling poseidon2
    /// hash of each proof+vkey hash and the previous hash which is initially zero.
    pub deferred_proofs_digest: [T; POSEIDON_NUM_WORDS],

    /// The shard's start program counter.
    pub start_pc: T,

    /// The expected start program counter for the next shard.
    pub next_pc: T,

    /// The exit code of the program.  Only valid if halt has been executed.
    pub exit_code: T,

    /// The shard number.
    pub shard: T,

    /// The largest address that is witnessed for initialization in the previous shard.
    pub previous_init_addr: T,

    /// The largest address that is witnessed for initialization in the current shard.
    pub last_init_addr: T,

    /// The largest address that is witnessed for finalization in the previous shard.
    pub previous_finalize_addr: T,

    /// The largest address that is witnessed for finalization in the current shard.
    pub last_finalize_addr: T,
}

impl PublicValues<u32, u32> {
    /// Convert the public values into a vector of field elements.  This function will pad the vector
    /// to the maximum number of public values.
    pub fn to_vec<F: AbstractField>(&self) -> Vec<F> {
        let mut ret = vec![F::zero(); PROOF_MAX_NUM_PVS];

        let field_values = PublicValues::<Word<F>, F>::from(*self);
        let ret_ref_mut: &mut PublicValues<Word<F>, F> = ret.as_mut_slice().borrow_mut();
        *ret_ref_mut = field_values;
        ret
    }
}

impl<F: PrimeField32> PublicValues<Word<F>, F> {
    /// Returns the commit digest as a vector of little-endian bytes.
    pub fn commit_digest_bytes(&self) -> Vec<u8> {
        self.committed_value_digest
            .iter()
            .flat_map(|w| w.into_iter().map(|f| f.as_canonical_u32() as u8))
            .collect_vec()
    }
}

impl<T: Clone> Borrow<PublicValues<Word<T>, T>> for [T] {
    fn borrow(&self) -> &PublicValues<Word<T>, T> {
        debug_assert_eq!(
            self.len(),
            std::mem::size_of::<PublicValues<Word<u8>, u8>>()
        );
        let (prefix, shorts, _suffix) = unsafe { self.align_to::<PublicValues<Word<T>, T>>() };
        debug_assert!(prefix.is_empty(), "Alignment should match");
        debug_assert_eq!(shorts.len(), 1);
        &shorts[0]
    }
}

impl<T: Clone> BorrowMut<PublicValues<Word<T>, T>> for [T] {
    fn borrow_mut(&mut self) -> &mut PublicValues<Word<T>, T> {
        debug_assert_eq!(
            self.len(),
            std::mem::size_of::<PublicValues<Word<u8>, u8>>()
        );
        let (prefix, shorts, _suffix) = unsafe { self.align_to_mut::<PublicValues<Word<T>, T>>() };
        debug_assert!(prefix.is_empty(), "Alignment should match");
        debug_assert_eq!(shorts.len(), 1);
        &mut shorts[0]
    }
}

impl<F: AbstractField> From<PublicValues<u32, u32>> for PublicValues<Word<F>, F> {
    fn from(value: PublicValues<u32, u32>) -> Self {
        let PublicValues {
            committed_value_digest,
            deferred_proofs_digest,
            start_pc,
            next_pc,
            exit_code,
            shard,
            previous_init_addr,
            last_init_addr,
            previous_finalize_addr,
            last_finalize_addr,
        } = value;

        let committed_value_digest: [_; PV_DIGEST_NUM_WORDS] =
            core::array::from_fn(|i| Word::from(committed_value_digest[i]));

        let deferred_proofs_digest: [_; POSEIDON_NUM_WORDS] =
            core::array::from_fn(|i| F::from_canonical_u32(deferred_proofs_digest[i]));

        let start_pc = F::from_canonical_u32(start_pc);
        let next_pc = F::from_canonical_u32(next_pc);
        let exit_code = F::from_canonical_u32(exit_code);
        let shard = F::from_canonical_u32(shard);
        let previous_init_addr = F::from_canonical_u32(previous_init_addr);
        let last_init_addr = F::from_canonical_u32(last_init_addr);
        let previous_finalize_addr = F::from_canonical_u32(previous_finalize_addr);
        let last_finalize_addr = F::from_canonical_u32(last_finalize_addr);

        Self {
            committed_value_digest,
            deferred_proofs_digest,
            start_pc,
            next_pc,
            exit_code,
            shard,
            previous_init_addr,
            last_init_addr,
            previous_finalize_addr,
            last_finalize_addr,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::air::public_values;

    /// Check that the PI_DIGEST_NUM_WORDS number match the zkVM crate's.
    #[test]
    fn test_public_values_digest_num_words_consistency_zkvm() {
        assert_eq!(
            public_values::PV_DIGEST_NUM_WORDS,
            sp1_zkvm::PV_DIGEST_NUM_WORDS
        );
    }
}
