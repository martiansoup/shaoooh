#include "neopixel.h"
#include <stdio.h>

#include "piolib.h"
#include "ws2812.pio.h"

pio_info_t init_neopixel(unsigned int num_pixels, unsigned int gpio) {
    pio_info_t info;
    unsigned int offset;

    info.pio_ptr = pio0;
    info.sm = pio_claim_unused_sm(info.pio_ptr, true);
    info.num_pixels = num_pixels;
    pio_sm_config_xfer(info.pio_ptr, info.sm, PIO_DIR_TO_SM, 256, 1);

    offset = pio_add_program(info.pio_ptr, &ws2812_program);
    printf("Loaded program at %d, using sm %d, gpio %d\n", offset, info.sm, gpio);

    pio_sm_clear_fifos(info.pio_ptr, info.sm);
    pio_sm_set_clkdiv(info.pio_ptr, info.sm, 1.0);
    ws2812_program_init(info.pio_ptr, info.sm, offset, gpio, 800000.0, false);

    return info;
}

// Array of length num_pixels * 4 (W, R, G, B)
void write_pixels(pio_info_t * pio, uint8_t data[]) {
    pio_sm_xfer_data(pio->pio_ptr, pio->sm, PIO_DIR_TO_SM, (pio->num_pixels*4), data);
}