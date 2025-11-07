#ifndef SEQGEN_H
#define SEQGEN_H

/*
 * A small C library that generates all possible sequences of port values from
 * a supplied list.  It is intentionally simple and avoids dependencies so
 * that it can be built on minimal systems.  Rust invokes the generator via
 * FFI.  Each sequence is passed back to a Rust callback along with a
 * user‑supplied pointer.  Clients must ensure the callback does not panic
 * across the FFI boundary.
 */

#include <stddef.h>
#include <stdint.h>

/**
 * Signature of the callback used by generate_sequences().  The callback is
 * invoked once for each sequence of length `seq_len`.  The sequence is
 * presented as a pointer to an array of uint16_t values.  The callback
 * receives the user_data pointer untouched.  Returning from the callback
 * simply continues the generation.  Callbacks must not longjmp or unwind
 * across the FFI boundary.
 */
typedef void (*sequence_callback)(const uint16_t *sequence, size_t length, void *user_data);

/**
 * Generate all sequences of length `seq_len` from the provided array of
 * `ports`.  If `num_ports` is zero or `seq_len` is zero, nothing will be
 * generated.  Each generated sequence is passed to `callback` along with
 * `user_data`.
 *
 * The sequences are generated with repetition; that is, the same port may
 * appear multiple times within a sequence.  The order of generation is
 * deterministic but otherwise unspecified.
 */
void generate_sequences(const uint16_t *ports, size_t num_ports, size_t seq_len, sequence_callback callback, void *user_data);

/**
 * Return the total number of sequences that generate_sequences() would
 * produce for the given number of ports and sequence length.  This is
 * simply `num_ports` raised to the power of `seq_len`.  Use this to size
 * collections prior to generation if desired.
 */
uint64_t count_sequences(size_t num_ports, size_t seq_len);

#endif /* SEQGEN_H */