from machine import Pin
import time

# Button mapping

# Setup buttons to high impedance output (not-pressed)
# Connecting each button to low/ground is a button press
# Numbered by GPx
pins = {
  'R': Pin(2, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  'X': Pin(3, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  'A': Pin(4, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  'B': Pin(5, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  's': Pin(6, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  'S': Pin(7, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  'Y': Pin(8, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  'L': Pin(9, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  'r': Pin(10, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  'u': Pin(11, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  'd': Pin(12, mode=Pin.OPEN_DRAIN, pull=None, value=1),
  'l': Pin(13, mode=Pin.OPEN_DRAIN, pull=None, value=1),
}

led = Pin(25, mode=Pin.OUT, value=0)


# Indicate setup complete
led.value(1)
time.sleep(0.5)
led.value(0)
print('Shaooh initialised...')

x = 5
while x > 0:
    print('countdown = {}'.format(x))
    time.sleep(1)
    x -= 1

print('Testing A input')
for x in range(10):
    print('press {}'.format(x))
    pins['A'].value(0)
    time.sleep(0.2)
    pins['A'].value(1)
    time.sleep(0.1)
    pins['B'].value(0)
    time.sleep(0.2)
    pins['B'].value(1)
    time.sleep(0.1)
    pins['X'].value(0)
    time.sleep(0.2)
    pins['X'].value(1)
    time.sleep(0.1)
    pins['Y'].value(0)
    time.sleep(0.2)
    pins['Y'].value(1)
    time.sleep(0.1)
    pins['A'].value(0)
    pins['B'].value(0)
    pins['X'].value(0)
    pins['Y'].value(0)
    time.sleep(0.2)
    pins['A'].value(1)
    pins['B'].value(1)
    pins['X'].value(1)
    pins['Y'].value(1)
    time.sleep(0.1)
    time.sleep(1)

