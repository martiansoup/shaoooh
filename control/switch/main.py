import nxbt
import time
import os

from contextlib import contextmanager

def control_fifo():
    """Context Manager for named pipes."""
    filename = "./_control_pipe"
    if not os.path.exists(filename):
      os.mkfifo(filename, mode=0o666)
      os.chown(filename, 1000, 1000)
    return filename

nx = nxbt.Nxbt()

# TODO - reconnect
# controller_index = nx.create_controller(
#     nxbt.PRO_CONTROLLER,
#     reconnect_address=nx.get_switch_addresses())
controller_index = nx.create_controller(nxbt.PRO_CONTROLLER, reconnect_address=nx.get_switch_addresses())
print("Waiting for connection...")
nx.wait_for_connection(controller_index)
print("  ...connected")

delay_times = {
  b'm': 0.05, # micro-pause
  b'p': 0.4,  # pause TODO - set as longer for skipped inputs
  b'P': 0.5,  # Pause
  b'M': 1     # mega-pause
}

pins = {
  b'R': [nxbt.Buttons.R],
  b'X': [nxbt.Buttons.X],
  b'A': [nxbt.Buttons.A],
  b'B': [nxbt.Buttons.B],
  b's': [nxbt.Buttons.MINUS],
  b'S': [nxbt.Buttons.PLUS],
  b'Y': [nxbt.Buttons.Y],
  b'L': [nxbt.Buttons.L],
  b'r': [nxbt.Buttons.DPAD_RIGHT],
  b'u': [nxbt.Buttons.DPAD_UP],
  b'd': [nxbt.Buttons.DPAD_DOWN],
  b'l': [nxbt.Buttons.DPAD_LEFT],
  b'h': [nxbt.Buttons.HOME],
  b'!': [nxbt.Buttons.A, nxbt.Buttons.B, nxbt.Buttons.X, nxbt.Buttons.Y]
}

try:
    fifo_fname = control_fifo()
    with open(fifo_fname, 'rb') as f:
        use_next_char = False
        current_cmd = None
        one_button = None
        delay = None
        zero_button = None
        while True:
            byte = f.read(1)

            # 'q' used as delimiter to indicate next char is a valid command
            if byte == b'q':
                use_next_char = True
            elif use_next_char:
                # 'p' indicates pause, else use as indication of button to switch
                if byte in delay_times:
                    delay = delay_times[byte]
                else:
                    current_cmd = byte
                    use_next_char = False
            elif current_cmd is not None:
                val = 1 # Not pressed (active-low)
                if byte == b'1':
                    val = 0
                if current_cmd in pins:
                    if val == 1:
                        one_button = pins[current_cmd]
                    else:
                        zero_button = pins[current_cmd]
                if one_button is not None and delay is not None and zero_button is not None:
                    print(f"PRESS {one_button} for {delay}s")
                    nx.press_buttons(controller_index, one_button, down=delay)
                    one_button = None
                    delay = None
                    zero_button = None
                current_cmd = None

            #nx.press_buttons(controller_index, [nxbt.Buttons.A], down=0.1)
            time.sleep(0.1)
except KeyboardInterrupt:
    print("Done")
