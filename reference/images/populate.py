#!/usr/bin/env python3

import os
import urllib.request
import csv

def try_get_images(root_url, destination, limit, suffix):
    s = "" if len(suffix) == 0 else f"_{suffix}"
    for x in range(1, limit+1):
        zero_padded = f"{x:03}"
        dest = os.path.join(destination, f"{zero_padded}{s}.png")
        url = f"{root_url}{zero_padded}.png"
        if not os.path.exists(dest):
            urllib.request.urlretrieve(url, dest)

def try_get_images_dp(root_url, destination, limit, suffix):
    s = "" if len(suffix) == 0 else f"_{suffix}"
    for x in range(1, limit+1):
        zero_padded = f"{x:03}"
        dest = os.path.join(destination, f"{zero_padded}{s}.png")
        if x == 422:
            zero_padded += "-w"
        url = f"{root_url}{zero_padded}.png"
        if not os.path.exists(dest):
            urllib.request.urlretrieve(url, dest)

def try_get_images_rs(root_url, destination, limit, suffix):
    s = "" if len(suffix) == 0 else f"_{suffix}"
    with open("../pokeapi/data/v2/csv/pokemon_dex_numbers.csv") as f:
        reader = csv.reader(f, delimiter=',', quotechar='"')
        for r in reader:
            if r[1] == "4" and int(r[2]) <= 202: # Hoenn dex and less than (or eq) 202
                hoenn = int(r[2])
                nat = int(r[0])
                zero_padded_nat = f"{nat:03}"
                zero_padded_h = f"{hoenn:03}"
                dest = os.path.join(destination, f"{zero_padded_nat}{s}.gif")
                url = f"{root_url}{zero_padded_h}.gif"
                if not os.path.exists(dest):
                    urllib.request.urlretrieve(url, dest)

    #for x in range(1, limit+1):

def frlg():
    destination = "frlg"
    last_in_dex = 386 # Deoxys
    if not os.path.exists(destination):
        os.mkdir(destination)
    try_get_images("https://www.serebii.net/pokearth/sprites/frlg/", destination, last_in_dex, "")
    try_get_images("https://www.serebii.net/Shiny/FRLG/", destination, 151, "shiny")
    # TODO only gets shiny up to 151

def rs():
    destination = "rs"
    last_in_dex = 386 # Deoxys
    if not os.path.exists(destination):
        os.mkdir(destination)
    try_get_images("https://www.serebii.net/pokearth/sprites/rs/", destination, last_in_dex, "")
    try_get_images_rs("https://www.serebii.net/Shiny/RuSa/", destination, last_in_dex, "shiny")

def diamond_pearl():
    destination = "dp"
    last_in_dex = 493 # Arceus
    if not os.path.exists(destination):
        os.mkdir(destination)
    try_get_images_dp("https://www.serebii.net/pokearth/sprites/dp/", destination, last_in_dex, "")
    try_get_images_dp("https://www.serebii.net/Shiny/DP/", destination, last_in_dex, "shiny")

def hgss():
    destination = "hgss"
    last_in_dex = 251 # TBD rest use Pt sprites?
    if not os.path.exists(destination):
        os.mkdir(destination)
    try_get_images("https://www.serebii.net/pokearth/sprites/hgss/", destination, last_in_dex, "")
    try_get_images("https://www.serebii.net/Shiny/HGSS/", destination, last_in_dex, "shiny")

def bw():
    destination = "bw"
    last_in_dex = 649 # Genesect
    if not os.path.exists(destination):
        os.mkdir(destination)
    try_get_images("https://www.serebii.net/blackwhite/pokemon/", destination, last_in_dex, "")
    try_get_images("https://www.serebii.net/Shiny/BW/", destination, last_in_dex, "shiny")

def usum():
    destination = "usum"
    last_in_dex = 809 # Melmetal
    if not os.path.exists(destination):
        os.mkdir(destination)
    try_get_images("https://www.serebii.net/sunmoon/pokemon/", destination, last_in_dex, "")
    try_get_images("https://www.serebii.net/Shiny/SM/", destination, last_in_dex, "shiny")

def gif2png():
    for root, dirs, files in os.walk('.'):
        for f in files:
            fullname = os.path.join(root, f)
            fullname_png = fullname.replace('.gif', '.png')
            if f.endswith('.gif') and not os.path.exists(fullname_png):
                os.system(f'convert {fullname} {fullname_png}')

if __name__=="__main__":
    frlg()
    rs()
    diamond_pearl()
    hgss()
    bw()
    usum()
    gif2png()
