// Deserialise instruction data sent by client
pub mod instruction;

// Manage Accounts state
pub mod state;

// Process logic
pub mod processor;

// Entrypoint for Solana BPF Loader
#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;
