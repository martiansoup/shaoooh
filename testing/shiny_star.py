#!/usr/bin/env python3

import cv2
import os

# White pixel = 255

cam = cv2.VideoCapture("/dev/video0")
cam.set(cv2.CAP_PROP_BRIGHTNESS, 50)

border_l = 16
border_r = border_l
border_t = 20
border_b = border_t

cam.set(cv2.CAP_PROP_FRAME_WIDTH, 320)
cam.set(cv2.CAP_PROP_FRAME_HEIGHT, 240)
# Get the default frame width and height
frame_width = int(cam.get(cv2.CAP_PROP_FRAME_WIDTH))
frame_height = int(cam.get(cv2.CAP_PROP_FRAME_HEIGHT))

border_keep = 0
x0 = border_l-border_keep
y0 = border_t-border_keep
x1 = frame_width - (border_r-border_keep)
y1 = frame_height - (border_b-border_keep)

print(f"{x0},{y0} {x1},{y1}")

imgout_idx = 0

state = "IDLE"
current_find = None

min = 99999999
max = 0

while True:
    to_print = ""
    trigger_detect = False
    ret, frame = cam.read()
    # Remove borders
    frame = frame[y0:y1, x0:x1]
    frame = cv2.resize(frame, (256, 192))

    star_region = frame[52:66, 106:120]

    star_grey = cv2.cvtColor(star_region, cv2.COLOR_BGR2GRAY)
    cv2.imshow('star', star_grey)
    _, star_wht = cv2.threshold(star_grey, 200, 255, cv2.THRESH_BINARY)
    no_star_wht = cv2.countNonZero(star_wht)
    if no_star_wht > max:
        max = no_star_wht
    if no_star_wht < min:
        min = no_star_wht
    print(f"{no_star_wht} max={max} min={min}                      ", end='\r')

    cv2.imshow('Camera', frame)

    k = cv2.waitKey(1)

    if k == ord('q'):
        break
    if k == ord('i'):
        print(type(frame))
    if k == ord('c'):
        while os.path.exists(f"{imgout_idx}.png"):
            imgout_idx += 1
        cv2.imwrite(f"{imgout_idx}.png", frame)
        imgout_idx += 1

print()
print()
cam.release()
cv2.destroyAllWindows()
