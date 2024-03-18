use std::mem::size_of;

use sp1_derive::AlignedBorrow;

use crate::memory::MemoryReadCols;
use crate::memory::MemoryWriteCols;
use crate::operations::Add4Operation;

use super::s0::S0Operation;
use super::s1::S1Operation;

pub const NUM_SHA_EXTEND_COLS: usize = size_of::<ShaExtendCols<u8>>();

#[derive(AlignedBorrow, Default, Debug, Clone, Copy)]
#[repr(C)]
pub struct ShaExtendCols<T> {
    /// Inputs.
    pub shard: T,
    pub clk: T,
    pub w_ptr: T,

    /// Control flags.
    pub i: T,
    pub cycle_16: T,
    pub cycle_16_minus_g: T,
    pub cycle_16_minus_g_inv: T,
    pub cycle_16_start: T,
    pub cycle_16_minus_one: T,
    pub cycle_16_minus_one_inv: T,
    pub cycle_16_end: T,
    pub cycle_48: [T; 3],
    pub cycle_48_start: T,
    pub cycle_48_end: T,

    /// The input to `s0`.
    pub w_i_minus_15: MemoryReadCols<T>,

    /// `s0 := (w[i-15] rightrotate  7) xor (w[i-15] rightrotate 18) xor (w[i-15] rightshift  3)`.
    pub s0: S0Operation<T>,

    /// The input to `s1`.
    pub w_i_minus_2: MemoryReadCols<T>,

    /// `s1 := (w[i-2] rightrotate 17) xor (w[i-2] rightrotate 19) xor (w[i-2] rightshift 10)`.
    pub s1: S1Operation<T>,

    /// An input to `s2`.
    pub w_i_minus_16: MemoryReadCols<T>,

    /// An input to `s2`.
    pub w_i_minus_7: MemoryReadCols<T>,

    /// `w[i] := w[i-16] + s0 + w[i-7] + s1`.
    pub s2: Add4Operation<T>,

    /// Result.
    pub w_i: MemoryWriteCols<T>,

    /// Selector.
    pub is_real: T,
}
