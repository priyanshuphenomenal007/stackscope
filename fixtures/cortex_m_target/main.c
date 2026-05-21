#include <stdint.h>

volatile uint32_t global_counter = 0;

// Leaf function: Simple stack allocation
void leaf_function(void) {
    uint8_t buffer[64];
    buffer[0] = 1;
    global_counter += buffer[0];
}

// Recursive function: Deep stack allocation with a cycle
void recursive_function(uint32_t depth) {
    uint8_t recursive_buffer[128];
    recursive_buffer[0] = (uint8_t)depth;

    if (depth > 0) {
        recursive_function(depth - 1);
    }
}

int main(void) {
    leaf_function();
    recursive_function(3);

    while (1) {
        // Prevent termination
    }
    return 0;
}