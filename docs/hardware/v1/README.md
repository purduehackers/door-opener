# door-opener-v1


## External


v1 is comprised of the upper portion housing the Pi, and the lower portion housing
the NFC board and LED strips.

![Door opener v1 external back screw positions](./images/door-opener-v1-external-back-screw-positions.png)

The upper portion is held together with $${\color{red}\text{four T8 screws}}$$,
at the outer perimeter of the enclosure. The
$${\color{blue}\text{inner four Philips \\#0 screws}}$$ hold the fan used to cool
the Pi, and should not be removed unless to service the fan.

The lower portion is held together with $${\color{orange}\text{four T10 screws}}$$.
Try not to open the lower portion as there is a diffuser sheet that you have to
accurately place on the acrylic panel. If it shifts, you can see through the
acrylic panel and the lights will not diffuse properly.


## Internal

### NFC

The NFC module used is a PN532 board. Look for one on Amazon. The one that we
use is a red square with both the SPI and I2C interfaces, configurable with
DIP switches.

The module should be set to **SPI mode**. Refer to the documentation that came
with your board for instructions.

The NFC module is connected via 6 wires to the Pi.

The pin-out is as follows:

| PN532 | Pi      |
|-------|---------|
| SCK   | 23 - GPIO 11 (SPI0 SCLK) |
| MISO  | 21 - GPIO 9 (SPI0 MISO) |
| MOSI  | 19 - GPIO 10 (SPI0 MOSI) |
| SS    | 24 - GPIO 8 (SPI0 CE0) |
| VCC   | 17 - 3v3 Power |
| GND   | 25 - ground (any ground will work — 6, 9, 14, 20, 25, 30, 34, and 39) |
