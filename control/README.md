# Shaoooh Pico Controller

Python script to allow controlling DS Buttons according to controls sent over a serial link

## Status

Implemented control of buttons

## Protocol

A command is sent with 'q' followed by the button to be modified or 'p' to pause for 0.1 seconds.
A button identifier is followed by 0 or 1, to indicate unpressed or pressed.

Buttons match the name (in uppercase, e.g. A=A). Start is 'S', Select is 's' and D-pad directions
are lowercase 'u'p/'d'own/'l'eft/'r'ight.

E.g. 'qr1qpqr0' is press D-pad Right, wait, unpress

## Display

`display.py` contains a script to use the [Pimoroni GFX Pack](https://shop.pimoroni.com/products/pico-gfx-pack)
to display the current encounters. This expects to be sent `T<Target Dex No>e` to set the target sprite, and
`E<Num encounters>e` to set the current number of encounters (or soft resets).
