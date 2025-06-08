#!/usr/bin/env python3

import cv2
import os

cam = cv2.VideoCapture("/dev/video0")
cam.set(cv2.CAP_PROP_BRIGHTNESS, 50)

name = {
  396: "Starly",
  399: "Bidoof",
  401: "Kricketot",
  403: "Shinx"
        }
imgs = {}
masks = {}

nos = [396, 399, 401, 403]

for no in nos:
    n = name[no]
    m_in = cv2.imread(f"../reference/images/dp/{no:03}.png", cv2.IMREAD_UNCHANGED)
    imgs[n] = cv2.imread(f"../reference/images/dp/{no:03}.png", cv2.IMREAD_COLOR)
    _, masks[n] = cv2.threshold(m_in[:, :, 3], 0, 255, cv2.THRESH_BINARY)
    n = f"{name[no]} (shiny)"
    m_in = cv2.imread(f"../reference/images/dp/{no:03}_shiny.png", cv2.IMREAD_UNCHANGED)
    imgs[n] = cv2.imread(f"../reference/images/dp/{no:03}_shiny.png", cv2.IMREAD_COLOR)
    _, masks[n] = cv2.threshold(m_in[:, :, 3], 0, 255, cv2.THRESH_BINARY)

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

while True:
    ret, frame = cam.read()
    # Remove borders
    frame = frame[y0:y1, x0:x1]
    frame = cv2.resize(frame, (256, 192))

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
    if k == ord('t'):
        cno = None
        m = 0
        tpl = None
        matchLoc = None
        for no, img in imgs.items():
            res = cv2.matchTemplate(frame, img, cv2.TM_CCORR_NORMED, None, masks[no])
            minVal, maxVal, minLoc, maxLoc = cv2.minMaxLoc(res)
            print(f"{no} = {maxVal}")
            if maxVal > m:
                m = maxVal
                cno = no
                tpl = img
                matchLoc = maxLoc
        print(f"I think this is a {cno}")
        display = frame.copy()
        cv2.rectangle(display, matchLoc, (matchLoc[0]+tpl.shape[1], matchLoc[1]+tpl.shape[0]), (0,0,0), 2, 8, 0 )
        cv2.imshow("found", display)

cam.release()
cv2.destroyAllWindows()
