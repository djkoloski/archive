#[cfg(feature = "alloc")]
mod alloc;
mod core;
mod niche;
mod rend;
#[cfg(feature = "std")]
mod std;

// Support for various common crates. These are primarily to get users off the
// ground and build some momentum.

// These are NOT PLANNED to remain in rkyv for the final release. Much like
// serde, these implementations should be moved into their respective crates
// over time. Before adding support for another crate, please consider getting
// rkyv support in the crate instead.

#[cfg(feature = "arrayvec")]
mod arrayvec;
#[cfg(feature = "bitvec")]
mod bitvec;
#[cfg(feature = "bytes")]
mod bytes;
#[cfg(feature = "hashbrown")]
mod hashbrown;
#[cfg(feature = "indexmap")]
mod indexmap;
#[cfg(feature = "noisy_float")]
mod noisy_float;
#[cfg(feature = "smallvec")]
mod smallvec;
#[cfg(feature = "smol_str")]
mod smolstr;
#[cfg(feature = "thin-vec")]
mod thin_vec;
#[cfg(feature = "tinyvec")]
mod tinyvec;
#[cfg(feature = "triomphe")]
mod triomphe;
#[cfg(feature = "uuid")]
mod uuid;
