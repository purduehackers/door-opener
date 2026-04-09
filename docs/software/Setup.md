# Setup

Before installing `door-opener` on actual hardware, you need to prepare the
Raspberry Pi for deployment.

**Warning**: most of the code in this repository assumes that you have a user
account named `hackers` on your Pi.

This project can be built on the Raspbian OS Lite image. However, since the project
displays a GUI, you do need Wayland installed (which we will get to shortly).

## Connect RPi to Purdue's Wi-Fi

To connect `door-opener` to Purdue's Wi-Fi, use the following command, replacing
the username and password:

```
nmcli connection add \
    type wifi \
    con-name "PAL3.0" \
    ifname wlan0 \
    ssid "PAL3.0" \
    -- \
    wifi-sec.key-mgmt wpa-eap \
    802-1x.eap peap \
    802-1x.phase2-auth mschapv2 \
    802-1x.identity "[username]" \
    802-1x.password "[password]" \
    802-1x.system-ca-certs yes
nmcli connection up "PAL3.0"
```

For the Purdue Hackers account, please contact an organizer.

## Install dependencies

If you are imaging a new door opener, install the following dependencies:

- `libssl-dev`
- `libudev-dev`
- `libnfc-dev`
- `libclang-dev`

You will also need to install the display backend:

- `sway`
- `seatd`
- `xwayland`

Note that `xwayland` is required as `macroquad` does not support the Wayland backend yet. (Maybe you can help change this.)

## Configure NFC board details

You must also configure details for the NFC board (PN532) you are using. Edit
`/etc/nfc/libnfc.conf` and uncomment `device.connstring`. Depending on which SPI
port/pin you've connected the NFC board to, set this value accordingly. Example:

```
device.connstring = "pn532_spi:/dev/spidev0.0"
```

## Configure [`ada-pusher`](https://github.com/purduehackers/ada-pusher)

If you are connecting `ada-pusher`, you must first enable the adapter, then pair it with `bluetoothctl`:

```
sudo rfkill unblock bluetooth
bluetoothctl
agent on
default-agent
scan on
# wait 10-20 sec
scan off
# replace MAC ID below
trust A0:A1:A2:A3:A4:A5
# type in PIN after
pair A0:A1:A2:A3:A4:A5
connect A0:A1:A2:A3:A4:A5
exit
```

For the MAC address of `ada-pusher`, please consult with an organizer.

---

You are now done with setup! [Follow the instructions in Install](./Install.md)
to continue installing `door-opener`.