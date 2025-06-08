#!/usr/bin/env python3

import cv2
import os

# White pixel = 255

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

bottom_bar_in_encounter_threshold = 6500
bottom_bar_start_encounter_threshold = 10000
hp_bar_ready_threshold = 1500

state = "IDLE"
current_find = None

while True:
    to_print = ""
    trigger_detect = False
    ret, frame = cam.read()
    # Remove borders
    frame = frame[y0:y1, x0:x1]
    frame = cv2.resize(frame, (256, 192))

    bottom_bar = frame[145:192, 0:256]
    bottom_bar_grey = cv2.cvtColor(bottom_bar, cv2.COLOR_BGR2GRAY)
    _, bottom_bar_thr_w = cv2.threshold(bottom_bar_grey, 210, 255, cv2.THRESH_BINARY)
    _, bottom_bar_thr_b = cv2.threshold(bottom_bar_grey, 40, 255, cv2.THRESH_BINARY_INV)
    hp_bar = frame[100:135, 150:256]
    hp_bar_grey = cv2.cvtColor(hp_bar, cv2.COLOR_BGR2GRAY)
    _, hp_bar_thr_w = cv2.threshold(hp_bar_grey, 210, 255, cv2.THRESH_BINARY)

    no_wh_pix_bottom = cv2.countNonZero(bottom_bar_thr_w)
    no_bk_pix_bottom = cv2.countNonZero(bottom_bar_thr_b)
    no_wh_pix_hp = cv2.countNonZero(hp_bar_thr_w)

    to_print = f"BtWht={no_wh_pix_bottom:5},BtBlk={no_bk_pix_bottom:5},HpWht={no_wh_pix_hp:5}"

    # Get current screen state
    entering_encounter = no_bk_pix_bottom > bottom_bar_start_encounter_threshold
    in_encounter = no_wh_pix_bottom > bottom_bar_in_encounter_threshold
    encounter_ready = no_wh_pix_hp > hp_bar_ready_threshold

    if state == "IDLE":
        if entering_encounter:
            state = "ENTER"
    elif state == "ENTER":
        if in_encounter:
            state = "WAIT"
    elif state == "WAIT":
        if in_encounter and encounter_ready:
            state = "READY"
            trigger_detect = True
    elif state == "READY":
        if not in_encounter and entering_encounter:
            state = "LEAVE"
            current_find = None
            print("\33[2K",end='\r');
    elif state == "LEAVE":
        if not entering_encounter and not in_encounter:
            state = "IDLE"

    to_print = f"State = {state:6}"
    if current_find is not None:
        to_print += f" : in encounter with {current_find}"

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
    if trigger_detect:
        cno = None
        m = 0
        tpl = None
        matchLoc = None
        match_values = []
        for no, img in imgs.items():
            res = cv2.matchTemplate(frame, img, cv2.TM_CCORR_NORMED, None, masks[no])
            minVal, maxVal, minLoc, maxLoc = cv2.minMaxLoc(res)
            match_values.append(f"{no}:{maxVal:.04}")
            if maxVal > m:
                m = maxVal
                cno = no
                tpl = img
                matchLoc = maxLoc
        #to_print += f"I think this is a {cno} ({', '.join(match_values)})"
        #to_print += f"I think this is a {cno}"
        current_find = cno
        display = frame.copy()
        cv2.rectangle(display, matchLoc, (matchLoc[0]+tpl.shape[1], matchLoc[1]+tpl.shape[0]), (0,0,0), 2, 8, 0 )
        cv2.imshow(cno, display)
    print(to_print, end="\r")

print()
print()
cam.release()
cv2.destroyAllWindows()
