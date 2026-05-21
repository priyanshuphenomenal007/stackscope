#include <stdint.h>

volatile uint32_t sink = 0;

void function_c(void) {
    uint8_t c_buffer[16];
    c_buffer[0] = 1;
    sink += c_buffer[0];
}

void function_b(void) {
    uint8_t b_buffer[64];
    b_buffer[0] = 2;

    function_c();

    sink += b_buffer[0];
}

void function_a(void) {
    uint8_t a_buffer[32];
    a_buffer[0] = 3;

    function_b();

    sink += a_buffer[0];
}

int main(void) {
    uint8_t main_buffer[24];
    main_buffer[0] = 4;

    function_a();

    sink += main_buffer[0];

    while (1) {
    }

    return 0;
}