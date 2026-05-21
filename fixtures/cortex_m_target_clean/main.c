#include <stdint.h>

volatile uint32_t global_counter = 0;

void leaf_function(void) {
    uint8_t buffer[64];
    buffer[0] = 1;
    global_counter += buffer[0];
}

void helper_function(void) {
    uint8_t helper_buffer[32];
    helper_buffer[0] = 7;
    global_counter += helper_buffer[0];
}

int main(void) {
    leaf_function();
    helper_function();

    while (1) {
    }

    return 0;
}