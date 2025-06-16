#ifndef _NEOPIXEL_H_
#define _NEOPIXEL_H_

#include <stdint.h>

typedef struct {
    void * pio_ptr;
    int sm;
    int num_pixels;
} pio_info_t;

pio_info_t init_neopixel(unsigned int num_pixels, unsigned int gpio);

// Array of length num_pixels * 4 (W, R, G, B)
void write_pixels(pio_info_t * pio, uint8_t data[]);

#endif // _NEOPIXEL_H_