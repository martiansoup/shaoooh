# Uses NetworkManager and WIFI_CONFIG from https://github.com/pimoroni/pimoroni-pico/tree/main/micropython/examples/common
import time
import sys
import uasyncio
import urequests
import select
import WIFI_CONFIG
import pngdec
from network_manager import NetworkManager

from gfx_pack import GfxPack

print("Starting display")

gp = GfxPack()
display = gp.display
graphics = display

def status_handler(mode, status, ip):  # noqa: ARG001
    graphics.set_pen(15)
    graphics.clear()
    graphics.set_pen(0)
    graphics.text("Network: {}".format(WIFI_CONFIG.SSID), 10, 10, scale=2)
    status_text = "Connecting..."
    if status is not None:
        if status:
            status_text = "Connection successful!"
        else:
            status_text = "Connection failed!"

    graphics.text(status_text, 10, 30, scale=2)
    graphics.text("IP: {}".format(ip), 10, 60, scale=2)
    graphics.update()

try:
    network_manager = NetworkManager(WIFI_CONFIG.COUNTRY, status_handler=status_handler)
    uasyncio.get_event_loop().run_until_complete(network_manager.client(WIFI_CONFIG.SSID, WIFI_CONFIG.PSK))
except Exception as e:  # noqa: BLE001
    print(f"Wifi connection failed! {e}")

WIDTH, HEIGHT = display.get_bounds()
display.set_backlight(0.1)  # turn down the white component of the backlight
display.clear()
gp.set_backlight(0, 25, 0)

# Starting values
encounters = 19290
prob = 100 - (100 * (1.0 - ((8191.0/8192.0)**encounters)))


poll_obj = select.poll()
poll_obj.register(sys.stdin, 1)

read_mode = None
read_buf = ""

sprite_valid = False
data = None

def read_input():
    global read_mode, read_buf, encounters, prob, data, sprite_valid
    if poll_obj.poll(0):
        inp = sys.stdin.read(1)
        if read_mode is None:
            if inp == "E":
                read_mode = "enc"
            elif inp == "T":
                read_mode = "tar"
        else:
            if inp == "e":
                if read_mode == "enc":
                    encounters = int(read_buf)
                    prob = 100 - (100 * (1.0 - ((8191.0/8192.0)**encounters)))
                elif read_mode == "tar":
                    target = int(read_buf)
                    url = "https://www.serebii.net/pokearth/sprites/hgss/{:03}.png".format(target)
                    data = urequests.get(url).content
                    #print(dir(data))
                    sprite_valid = True
                    print(url)
                read_mode = None
                read_buf = ""
            else:
                read_buf += inp

while True:
    display.set_pen(0)
    display.clear()
    display.set_pen(15)
    display.set_font("bitmap14_outline")
    display.set_thickness(2)
    display.text("SHAOOOH", 0, 0, scale=0.5)
    display.set_font("bitmap6")
    display.text("Attempts", 0, 20, scale=0.5)
    display.text(str(encounters), 5, 27)
    
    display.text("Probability", 0, 42, scale=0.5)
    display.text("{:0.4}%".format(prob), 5, 49)
    display.set_pen(4)
    if sprite_valid:
        png = pngdec.PNG(display)
        png.open_RAM(data)
        png.decode(63, 2, source=(10, 10, 70, 70))
    else:
        display.rectangle(83, 12, 40, 40)
    display.update()
    time.sleep(1.0 / 60)
    read_input()
