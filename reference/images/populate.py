#!/usr/bin/env python3

import os
import urllib.request

def try_get_images(root_url, destination, limit, suffix):
    s = "" if len(suffix) == 0 else f"_{suffix}"
    for x in range(1, limit+1):
        zero_padded = f"{x:03}"
        dest = os.path.join(destination, f"{zero_padded}{s}.png")
        url = f"{root_url}{zero_padded}.png"
        if not os.path.exists(dest):
            urllib.request.urlretrieve(url, dest)

def frlg():
    destination = "frlg"
    last_in_dex = 386 # Deoxys
    if not os.path.exists(destination):
        os.mkdir(destination)
    try_get_images("https://www.serebii.net/pokearth/sprites/frlg/", destination, last_in_dex, "")
    try_get_images("https://www.serebii.net/Shiny/FRLG/", destination, last_in_dex, "shiny")
    # TODO only gets shiny up to 151

def diamond_pearl():
    destination = "dp"
    last_in_dex = 493 # Arceus
    if not os.path.exists(destination):
        os.mkdir(destination)
    try_get_images("https://www.serebii.net/pokearth/sprites/dp/", destination, last_in_dex, "")
    try_get_images("https://www.serebii.net/Shiny/DP/", destination, last_in_dex, "shiny")

if __name__=="__main__":
    frlg()
    diamond_pearl()
