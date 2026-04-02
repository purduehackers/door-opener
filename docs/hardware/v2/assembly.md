

# Assembly

Start with the 3D print of the enclosure:

![Starting 3D print](./images/3d-print-start.png)

Using a soldering iron such as a Pinecil, insert heat set inserts into the
appropriate holes.

- For the back plate retention screw hole, insert a M3 heat set insert with a
  depth of 3 mm.
- For the NFC module screw holes, insert two M3 heat set inserts with a depth of
  3 mm.
- For the camera module screw holes, insert four M2 heat sert inserts with a
  depth of 2.5 mm.

Before:

![Before heat set inserts](./images/before-heat-set.png)
<image>

After:

![After heat set inserts](./images/after-heat-set.png)

Next, as the image above shows, put a piece of clear tape that covers the camera
hole, to prevent dust intrusion.

Screw in the PN532 NFC module with two M3 screws, depth 6.7 mm. Use a Torx T8
bit.

![Installing NFC module](./images/nfc-module.png)

Screw in the RPi camera module with four M2 screws, depth 5.8 mm. Use a Torx T6
bit.

![Installing camera module](./images/camera-module.png)

Insert the RPi display module from the front. The USB-C port should be situated
at the cutout.

![Inserting display from front](./images/rpi-display-front.png)

![USB-C cutout location and alignment](./images/usb-c-cutout.png)

Screw in the RPi display module from the back with four M2.5 screws, depth 10 mm.
Use a Torx T8 bit.

Warning: due to printing tolerances the fit might not be exact. If this happens,
drill into the 3D print and create new holes, then try screwing in the display
again. Attempting to forcefully screw in the display may break it!

![Screwing in the display](./images/display-screws.png)

Connect the cable from the camera module to the other free port on the RPi.

Connect the NFC module. The pinout is [the same as the v1 door-opener](../v1/README.md).

Slide on the back cover. Be careful not to pinch or damage any of the internal
cables.

Screw on the back cover with one M3 screw, depth 20 mm. Use a Torx T10 bit.

![Screwing on back cover](./images/back-plate-screw.png)

You should now have a finished `door-opener`! Proceed to imaging Raspberry Pi
OS onto an SD card, then follow the
[building/deployment section of the main README](../../../README.md#Building).

![Finished door-opener!](./images/finished.png)
