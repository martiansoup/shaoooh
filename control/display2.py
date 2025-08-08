import micropython
import select
import sys
import time
import random
from picographics import PicoGraphics, DISPLAY_PICO_DISPLAY_2, PEN_RGB332
from pimoroni import RGBLED
import _thread

led = RGBLED(26, 27, 28)

display = PicoGraphics(display=DISPLAY_PICO_DISPLAY_2, pen_type=PEN_RGB332)
display.set_backlight(1.0)

WIDTH, HEIGHT = display.get_bounds()

ds_w = 256
ds_h = 192
chunk_size = 32

BG = display.create_pen(0, 0, 40)
display.set_pen(BG)
display.clear()

pen = display.create_pen(0, 20, 100)
display.set_pen(pen)

x_off = (WIDTH-ds_w)//2
y_off = (HEIGHT-ds_h)//2

display.line(x_off-1, y_off-1, x_off-1, y_off+ds_h+1)
display.line(x_off-1, y_off-1, x_off+ds_w+1, y_off-1)
display.line(x_off+ds_w+1, y_off-1, x_off+ds_w+1, y_off+ds_h+1)
display.line(x_off-1, y_off+ds_h+1, x_off+ds_w+1, y_off+ds_h+1)

poll_obj = select.poll()
poll_obj.register(sys.stdin, 1)

# read modes:
#  0 - wait for D as start
#  1 - wait for coord byte
#  2 - wait for data
read_mode = 0

coord_x = 0
coord_y = 0
local_x = 0
local_y = 0
data_count = 0

fb = memoryview(display)

chunk_num_pixels = chunk_size * chunk_size

def read_input():
    global read_mode, fb, coord_x, coord_y, local_x, local_y, data_count

    while poll_obj.poll(0):
        inp = sys.stdin.buffer.read(1)
        if read_mode == 0:
            if inp == b'D':
                read_mode = 1
                micropython.kbd_intr(-1)
        elif read_mode == 1:
            coord_x = x_off + ((inp[0] & 0xf) * chunk_size)
            coord_y = y_off + (((inp[0] & 0xf0) >> 4) * chunk_size)
            read_mode = 2
            data_count = chunk_num_pixels
            local_x = 0
            local_y = 0
        elif read_mode == 2:
            pixel = inp[0]
            x = coord_x + local_x
            y = coord_y + local_y
            pixel_index = x + (y * WIDTH)
            fb[pixel_index] = inp[0]
            local_x += 1
            if local_x == chunk_size:
                local_x = 0
                local_y += 1
            data_count -= 1
            if data_count == 0:
                read_mode = 0
                micropython.kbd_intr(3)

val = 0

def up_thread():
    while True:
        time.sleep(1.0 / 20)
        display.update()

seoncd = _thread.start_new_thread(up_thread, ())

while True:
    time.sleep(1.0 / 120)
    read_input()
    g = 50 if read_mode != 0 else 0
    if val % 20 < 10:
        led.set_rgb(50, g, 0)
    else:
        led.set_rgb(0, g, 0)
    val += 1
