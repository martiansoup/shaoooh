from machine import Pin, UART
import time

# Button mapping

# Setup buttons to high impedance output (not-pressed)
# Connecting each button to low/ground is a button press
# Numbered by GPx
pins = {
  b'R': Pin(2, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b'X': Pin(3, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b'A': Pin(4, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b'B': Pin(5, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b's': Pin(6, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b'S': Pin(7, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b'Y': Pin(8, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b'L': Pin(9, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b'r': Pin(10, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b'u': Pin(11, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b'd': Pin(12, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  b'l': Pin(13, mode=Pin.OPEN_DRAIN, pull=None, value=1),
}

led = Pin(25, mode=Pin.OUT, value=0)
uart = UART(0, baudrate=115200, bits=8, parity=None, tx=Pin(16), rx=Pin(17))


# Indicate setup complete
led.value(1)
time.sleep(0.5)
led.value(0)
print('Shaooh initialised...')

# Start monitoring UART
use_next_char = False
current_cmd = None
while True:
  if uart.any():
    byte = uart.read(1)
    # 'q' used as delimiter to indicate next char is a valid command
    if byte == b'q':
        use_next_char = True
    elif use_next_char:
        # 'p' indicates pause, else use as indication of button to switch
        if byte == b'p':
          time.sleep(0.1)
        else:
          current_cmd = byte
        use_next_char = False
    elif current_cmd is not None:
        val = 1 # Not pressed (active-low)
        if byte == b'1':
            val = 0
        pins[current_cmd].value(val)
        current_cmd = None
  else:
    time.sleep(0.01)

