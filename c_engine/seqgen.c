#include "seqgen.h"
#include <stdlib.h>
#include <string.h>

/*
 * Internal recursive function used to build sequences one element at a time.
 * At each recursion level it sets the current position in the working buffer
 * and then recurses until the sequence length is reached, at which point it
 * passes the buffer to the caller supplied callback.  The working buffer
 * contents are not copied when invoking the callback – the callback must
 * immediately copy the sequence if it wishes to retain it.
 */
static void generate_sequences_recursive(const uint16_t *ports, size_t num_ports,
                                         size_t seq_len, uint16_t *buffer,
                                         size_t depth, sequence_callback callback,
                                         void *user_data) {
    if (depth == seq_len) {
        callback(buffer, seq_len, user_data);
        return;
    }
    for (size_t i = 0; i < num_ports; i++) {
        buffer[depth] = ports[i];
        generate_sequences_recursive(ports, num_ports, seq_len, buffer, depth + 1,
                                     callback, user_data);
    }
}

void generate_sequences(const uint16_t *ports, size_t num_ports, size_t seq_len,
                        sequence_callback callback, void *user_data) {
    if (!ports || num_ports == 0 || seq_len == 0 || !callback) {
        return;
    }
    uint16_t *buffer = (uint16_t *)malloc(sizeof(uint16_t) * seq_len);
    if (!buffer) {
        return;
    }
    generate_sequences_recursive(ports, num_ports, seq_len, buffer, 0, callback, user_data);
    free(buffer);
}

uint64_t count_sequences(size_t num_ports, size_t seq_len) {
    if (seq_len == 0) {
        return 0;
    }
    uint64_t count = 1;
    for (size_t i = 0; i < seq_len; i++) {
        count *= (uint64_t)num_ports;
    }
    return count;
}