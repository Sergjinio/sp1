use hashbrown::HashMap;
use itertools::{EitherOrBoth, Itertools};
use p3_field::AbstractField;
use sp1_stark::{air::PublicValues, MachineRecord, SP1CoreOpts, SplitOpts};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::events::{
    add_sharded_byte_lookup_events, AluEvent, ByteLookupEvent, ByteRecord, CpuEvent,
    EdDecompressEvent, EllipticCurveAddEvent, EllipticCurveDecompressEvent,
    EllipticCurveDoubleEvent, KeccakPermuteEvent, MemoryInitializeFinalizeEvent, MemoryRecordEnum,
    ShaCompressEvent, ShaExtendEvent, Uint256MulEvent,
};

use super::{Opcode, Program};

/// A record of the execution of a program.
///
/// The trace of the execution is represented as a list of "events" that occur every cycle.
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// The program.
    pub program: Arc<Program>,
    /// A trace of the CPU events which get emitted during execution.
    pub cpu_events: Vec<CpuEvent>,
    /// A trace of the ADD, and ADDI events.
    pub add_events: Vec<AluEvent>,
    /// A trace of the MUL events.
    pub mul_events: Vec<AluEvent>,
    /// A trace of the SUB events.
    pub sub_events: Vec<AluEvent>,
    /// A trace of the XOR, XORI, OR, ORI, AND, and ANDI events.
    pub bitwise_events: Vec<AluEvent>,
    /// A trace of the SLL and SLLI events.
    pub shift_left_events: Vec<AluEvent>,
    /// A trace of the SRL, SRLI, SRA, and SRAI events.
    pub shift_right_events: Vec<AluEvent>,
    /// A trace of the DIV, DIVU, REM, and REMU events.
    pub divrem_events: Vec<AluEvent>,
    /// A trace of the SLT, SLTI, SLTU, and SLTIU events.
    pub lt_events: Vec<AluEvent>,
    /// A trace of the byte lookups that are needed.
    pub byte_lookups: HashMap<u32, HashMap<ByteLookupEvent, usize>>,
    /// A trace of the sha256 extend events.
    pub sha_extend_events: Vec<ShaExtendEvent>,
    /// A trace of the sha256 compress events.
    pub sha_compress_events: Vec<ShaCompressEvent>,
    /// A trace of the keccak256 permute events.
    pub keccak_permute_events: Vec<KeccakPermuteEvent>,
    /// A trace of the edwards add events.
    pub ed_add_events: Vec<EllipticCurveAddEvent>,
    /// A trace of the edwards decompress events.
    pub ed_decompress_events: Vec<EdDecompressEvent>,
    /// A trace of the secp256k1 add events.
    pub secp256k1_add_events: Vec<EllipticCurveAddEvent>,
    /// A trace of the secp256k1 double events.
    pub secp256k1_double_events: Vec<EllipticCurveDoubleEvent>,
    /// A trace of the bn254 add events.
    pub bn254_add_events: Vec<EllipticCurveAddEvent>,
    /// A trace of the bn254 double events.
    pub bn254_double_events: Vec<EllipticCurveDoubleEvent>,
    /// A trace of the k256 decompress events.
    pub k256_decompress_events: Vec<EllipticCurveDecompressEvent>,
    /// A trace of the bls12381 add events.
    pub bls12381_add_events: Vec<EllipticCurveAddEvent>,
    /// A trace of the bls12381 double events.
    pub bls12381_double_events: Vec<EllipticCurveDoubleEvent>,
    /// A trace of the uint256 mul events.
    pub uint256_mul_events: Vec<Uint256MulEvent>,
    /// A trace of the memory initialize events.
    pub memory_initialize_events: Vec<MemoryInitializeFinalizeEvent>,
    /// A trace of the memory finalize events.
    pub memory_finalize_events: Vec<MemoryInitializeFinalizeEvent>,
    /// A trace of the bls12381 decompress events.
    pub bls12381_decompress_events: Vec<EllipticCurveDecompressEvent>,
    /// The public values.
    pub public_values: PublicValues<u32, u32>,
    /// The nonce lookup.
    pub nonce_lookup: HashMap<u128, u32>,
}

impl ExecutionRecord {
    /// Create a new [`ExecutionRecord`].
    #[must_use]
    pub fn new(program: Arc<Program>) -> Self {
        Self { program, ..Default::default() }
    }

    /// Add a mul event to the execution record.
    pub fn add_mul_event(&mut self, mul_event: AluEvent) {
        self.mul_events.push(mul_event);
    }

    /// Add a lt event to the execution record.
    pub fn add_lt_event(&mut self, lt_event: AluEvent) {
        self.lt_events.push(lt_event);
    }

    /// Add a batch of alu events to the execution record.
    pub fn add_alu_events(&mut self, mut alu_events: HashMap<Opcode, Vec<AluEvent>>) {
        for (opcode, value) in &mut alu_events {
            match opcode {
                Opcode::ADD => {
                    self.add_events.append(value);
                }
                Opcode::MUL | Opcode::MULH | Opcode::MULHU | Opcode::MULHSU => {
                    self.mul_events.append(value);
                }
                Opcode::SUB => {
                    self.sub_events.append(value);
                }
                Opcode::XOR | Opcode::OR | Opcode::AND => {
                    self.bitwise_events.append(value);
                }
                Opcode::SLL => {
                    self.shift_left_events.append(value);
                }
                Opcode::SRL | Opcode::SRA => {
                    self.shift_right_events.append(value);
                }
                Opcode::SLT | Opcode::SLTU => {
                    self.lt_events.append(value);
                }
                _ => {
                    panic!("Invalid opcode: {opcode:?}");
                }
            }
        }
    }

    /// Take out events from the [`ExecutionRecord`] that should be deferred to a separate shard.
    ///
    /// Note: we usually defer events that would increase the recursion cost significantly if
    /// included in every shard.
    #[must_use]
    pub fn defer(&mut self) -> ExecutionRecord {
        ExecutionRecord {
            keccak_permute_events: std::mem::take(&mut self.keccak_permute_events),
            secp256k1_add_events: std::mem::take(&mut self.secp256k1_add_events),
            secp256k1_double_events: std::mem::take(&mut self.secp256k1_double_events),
            bn254_add_events: std::mem::take(&mut self.bn254_add_events),
            bn254_double_events: std::mem::take(&mut self.bn254_double_events),
            bls12381_add_events: std::mem::take(&mut self.bls12381_add_events),
            bls12381_double_events: std::mem::take(&mut self.bls12381_double_events),
            sha_extend_events: std::mem::take(&mut self.sha_extend_events),
            sha_compress_events: std::mem::take(&mut self.sha_compress_events),
            ed_add_events: std::mem::take(&mut self.ed_add_events),
            ed_decompress_events: std::mem::take(&mut self.ed_decompress_events),
            k256_decompress_events: std::mem::take(&mut self.k256_decompress_events),
            uint256_mul_events: std::mem::take(&mut self.uint256_mul_events),
            bls12381_decompress_events: std::mem::take(&mut self.bls12381_decompress_events),
            memory_initialize_events: std::mem::take(&mut self.memory_initialize_events),
            memory_finalize_events: std::mem::take(&mut self.memory_finalize_events),
            ..Default::default()
        }
    }

    /// Splits the deferred [`ExecutionRecord`] into multiple [`ExecutionRecord`]s, each which
    /// contain a "reasonable" number of deferred events.
    pub fn split(&mut self, last: bool, opts: SplitOpts) -> Vec<ExecutionRecord> {
        let mut shards = Vec::new();

        macro_rules! split_events {
            ($self:ident, $events:ident, $shards:ident, $threshold:expr, $exact:expr) => {
                let events = std::mem::take(&mut $self.$events);
                let chunks = events.chunks_exact($threshold);
                if !$exact {
                    $self.$events = chunks.remainder().to_vec();
                } else {
                    let remainder = chunks.remainder().to_vec();
                    if !remainder.is_empty() {
                        $shards.push(ExecutionRecord {
                            $events: chunks.remainder().to_vec(),
                            program: self.program.clone(),
                            ..Default::default()
                        });
                    }
                }
                let mut event_shards = chunks
                    .map(|chunk| ExecutionRecord {
                        $events: chunk.to_vec(),
                        program: self.program.clone(),
                        ..Default::default()
                    })
                    .collect::<Vec<_>>();
                $shards.append(&mut event_shards);
            };
        }

        split_events!(self, keccak_permute_events, shards, opts.keccak, last);
        split_events!(self, secp256k1_add_events, shards, opts.deferred, last);
        split_events!(self, secp256k1_double_events, shards, opts.deferred, last);
        split_events!(self, bn254_add_events, shards, opts.deferred, last);
        split_events!(self, bn254_double_events, shards, opts.deferred, last);
        split_events!(self, bls12381_add_events, shards, opts.deferred, last);
        split_events!(self, bls12381_double_events, shards, opts.deferred, last);
        split_events!(self, sha_extend_events, shards, opts.sha_extend, last);
        split_events!(self, sha_compress_events, shards, opts.sha_compress, last);
        split_events!(self, ed_add_events, shards, opts.deferred, last);
        split_events!(self, ed_decompress_events, shards, opts.deferred, last);
        split_events!(self, k256_decompress_events, shards, opts.deferred, last);
        split_events!(self, uint256_mul_events, shards, opts.deferred, last);
        split_events!(self, bls12381_decompress_events, shards, opts.deferred, last);
        // _ = last_pct;

        if last {
            // shards.push(last_shard);

            self.memory_initialize_events.sort_by_key(|event| event.addr);
            self.memory_finalize_events.sort_by_key(|event| event.addr);

            let mut init_addr_bits = [0; 32];
            let mut finalize_addr_bits = [0; 32];
            for mem_chunks in self
                .memory_initialize_events
                .chunks(opts.memory)
                .zip_longest(self.memory_finalize_events.chunks(opts.memory))
            {
                let (mem_init_chunk, mem_finalize_chunk) = match mem_chunks {
                    EitherOrBoth::Both(mem_init_chunk, mem_finalize_chunk) => {
                        (mem_init_chunk, mem_finalize_chunk)
                    }
                    EitherOrBoth::Left(mem_init_chunk) => (mem_init_chunk, [].as_slice()),
                    EitherOrBoth::Right(mem_finalize_chunk) => ([].as_slice(), mem_finalize_chunk),
                };
                let mut shard = ExecutionRecord::default();
                shard.program = self.program.clone();
                shard.memory_initialize_events.extend_from_slice(mem_init_chunk);
                shard.public_values.previous_init_addr_bits = init_addr_bits;
                if let Some(last_event) = mem_init_chunk.last() {
                    let last_init_addr_bits = core::array::from_fn(|i| (last_event.addr >> i) & 1);
                    init_addr_bits = last_init_addr_bits;
                }
                shard.public_values.last_init_addr_bits = init_addr_bits;

                shard.memory_finalize_events.extend_from_slice(mem_finalize_chunk);
                shard.public_values.previous_finalize_addr_bits = finalize_addr_bits;
                if let Some(last_event) = mem_finalize_chunk.last() {
                    let last_finalize_addr_bits =
                        core::array::from_fn(|i| (last_event.addr >> i) & 1);
                    finalize_addr_bits = last_finalize_addr_bits;
                }
                shard.public_values.last_finalize_addr_bits = finalize_addr_bits;

                shards.push(shard);
            }
        }

        shards
    }
}

/// A memory access record.
#[derive(Debug, Copy, Clone, Default)]
pub struct MemoryAccessRecord {
    /// The memory access of the `a` register.
    pub a: Option<MemoryRecordEnum>,
    /// The memory access of the `b` register.
    pub b: Option<MemoryRecordEnum>,
    /// The memory access of the `c` register.
    pub c: Option<MemoryRecordEnum>,
    /// The memory access of the `memory` register.
    pub memory: Option<MemoryRecordEnum>,
}

impl MachineRecord for ExecutionRecord {
    type Config = SP1CoreOpts;

    fn stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("cpu_events".to_string(), self.cpu_events.len());
        stats.insert("add_events".to_string(), self.add_events.len());
        stats.insert("mul_events".to_string(), self.mul_events.len());
        stats.insert("sub_events".to_string(), self.sub_events.len());
        stats.insert("bitwise_events".to_string(), self.bitwise_events.len());
        stats.insert("shift_left_events".to_string(), self.shift_left_events.len());
        stats.insert("shift_right_events".to_string(), self.shift_right_events.len());
        stats.insert("divrem_events".to_string(), self.divrem_events.len());
        stats.insert("lt_events".to_string(), self.lt_events.len());
        stats.insert("sha_extend_events".to_string(), self.sha_extend_events.len());
        stats.insert("sha_compress_events".to_string(), self.sha_compress_events.len());
        stats.insert("keccak_permute_events".to_string(), self.keccak_permute_events.len());
        stats.insert("ed_add_events".to_string(), self.ed_add_events.len());
        stats.insert("ed_decompress_events".to_string(), self.ed_decompress_events.len());
        stats.insert("secp256k1_add_events".to_string(), self.secp256k1_add_events.len());
        stats.insert("secp256k1_double_events".to_string(), self.secp256k1_double_events.len());
        stats.insert("bn254_add_events".to_string(), self.bn254_add_events.len());
        stats.insert("bn254_double_events".to_string(), self.bn254_double_events.len());
        stats.insert("k256_decompress_events".to_string(), self.k256_decompress_events.len());
        stats.insert("bls12381_add_events".to_string(), self.bls12381_add_events.len());
        stats.insert("bls12381_double_events".to_string(), self.bls12381_double_events.len());
        stats.insert("uint256_mul_events".to_string(), self.uint256_mul_events.len());
        stats.insert(
            "bls12381_decompress_events".to_string(),
            self.bls12381_decompress_events.len(),
        );
        stats.insert("memory_initialize_events".to_string(), self.memory_initialize_events.len());
        stats.insert("memory_finalize_events".to_string(), self.memory_finalize_events.len());
        if !self.cpu_events.is_empty() {
            let shard = self.cpu_events[0].shard;
            stats.insert(
                "byte_lookups".to_string(),
                self.byte_lookups.get(&shard).map_or(0, hashbrown::HashMap::len),
            );
        }
        // Filter out the empty events.
        stats.retain(|_, v| *v != 0);
        stats
    }

    fn append(&mut self, other: &mut ExecutionRecord) {
        self.cpu_events.append(&mut other.cpu_events);
        self.add_events.append(&mut other.add_events);
        self.sub_events.append(&mut other.sub_events);
        self.mul_events.append(&mut other.mul_events);
        self.bitwise_events.append(&mut other.bitwise_events);
        self.shift_left_events.append(&mut other.shift_left_events);
        self.shift_right_events.append(&mut other.shift_right_events);
        self.divrem_events.append(&mut other.divrem_events);
        self.lt_events.append(&mut other.lt_events);
        self.sha_extend_events.append(&mut other.sha_extend_events);
        self.sha_compress_events.append(&mut other.sha_compress_events);
        self.keccak_permute_events.append(&mut other.keccak_permute_events);
        self.ed_add_events.append(&mut other.ed_add_events);
        self.ed_decompress_events.append(&mut other.ed_decompress_events);
        self.secp256k1_add_events.append(&mut other.secp256k1_add_events);
        self.secp256k1_double_events.append(&mut other.secp256k1_double_events);
        self.bn254_add_events.append(&mut other.bn254_add_events);
        self.bn254_double_events.append(&mut other.bn254_double_events);
        self.k256_decompress_events.append(&mut other.k256_decompress_events);
        self.bls12381_add_events.append(&mut other.bls12381_add_events);
        self.bls12381_double_events.append(&mut other.bls12381_double_events);
        self.uint256_mul_events.append(&mut other.uint256_mul_events);
        self.bls12381_decompress_events.append(&mut other.bls12381_decompress_events);

        if self.byte_lookups.is_empty() {
            self.byte_lookups = std::mem::take(&mut other.byte_lookups);
        } else {
            self.add_sharded_byte_lookup_events(vec![&other.byte_lookups]);
        }

        self.memory_initialize_events.append(&mut other.memory_initialize_events);
        self.memory_finalize_events.append(&mut other.memory_finalize_events);
    }

    fn register_nonces(&mut self, _opts: &Self::Config) {
        self.add_events.iter().enumerate().for_each(|(i, event)| {
            self.nonce_lookup.insert(event.lookup_id, i as u32);
        });

        self.sub_events.iter().enumerate().for_each(|(i, event)| {
            self.nonce_lookup.insert(event.lookup_id, (self.add_events.len() + i) as u32);
        });

        self.mul_events.iter().enumerate().for_each(|(i, event)| {
            self.nonce_lookup.insert(event.lookup_id, i as u32);
        });

        self.bitwise_events.iter().enumerate().for_each(|(i, event)| {
            self.nonce_lookup.insert(event.lookup_id, i as u32);
        });

        self.shift_left_events.iter().enumerate().for_each(|(i, event)| {
            self.nonce_lookup.insert(event.lookup_id, i as u32);
        });

        self.shift_right_events.iter().enumerate().for_each(|(i, event)| {
            self.nonce_lookup.insert(event.lookup_id, i as u32);
        });

        self.divrem_events.iter().enumerate().for_each(|(i, event)| {
            self.nonce_lookup.insert(event.lookup_id, i as u32);
        });

        self.lt_events.iter().enumerate().for_each(|(i, event)| {
            self.nonce_lookup.insert(event.lookup_id, i as u32);
        });
    }

    /// Retrieves the public values.  This method is needed for the `MachineRecord` trait, since
    fn public_values<F: AbstractField>(&self) -> Vec<F> {
        self.public_values.to_vec()
    }
}

impl ByteRecord for ExecutionRecord {
    fn add_byte_lookup_event(&mut self, blu_event: ByteLookupEvent) {
        *self.byte_lookups.entry(blu_event.shard).or_default().entry(blu_event).or_insert(0) += 1;
    }

    #[inline]
    fn add_sharded_byte_lookup_events(
        &mut self,
        new_events: Vec<&HashMap<u32, HashMap<ByteLookupEvent, usize>>>,
    ) {
        add_sharded_byte_lookup_events(&mut self.byte_lookups, new_events);
    }
}
