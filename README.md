# door-opener

🔑 Passport-based access control & door opener for Hack Night.

[Demo](https://youtu.be/31aA09y7xFA)

## Building/deployment

### On the Door Opener (Raspberry Pi)

**Warning**: most of the code in this repository assumes that you have a user
account named `hackers` on your Pi.

This project can be built on the Raspbian OS Lite image. However, since the project
displays a GUI, you do need X11 installed (read below).

Due to the GPIO libraries used, this project should be built directly on the
door opener. SSH into the door opener, pull any changes, and run `cargo build --release`.
If you are a member of Purdue Hackers and would like to contribute to door opener,
contact an organizer for SSH details!

If you are imaging a new door opener, install the following dependencies:

- `libssl-dev`
- `libudev-dev`
- `libnfc-dev`
- `libclang-dev`

You must also configure details for the NFC board (PN532) you are using. Edit
`/etc/nfc/libnfc.conf` and uncomment `device.connstring`. Depending on which SPI
port/pin you've connected the NFC board to, set this value accordingly. Example:

```
device.connstring = "pn532_spi:/dev/spidev0.0"
```

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

Next, for the display backend, choose your poison:

#### (Old, deprecated, not recommended) X11

Install additional dependencies:

- `xinit`

From `./install/`, copy `opener-app.service` to `/etc/systemd/system`.
Also copy `.xinitrc` to the root of the home directory (so `/home/hackers/.xinitrc`).
Then run:

```
sudo systemctl daemon-reload
sudo systemctl enable --now opener-app
```

#### Wayland

Install additional dependencies:

- `sway`
- `seatd`
- `xwayland`

Note that `xwayland` is required as `macroquad` does not support the Wayland backend yet. (Maybe you can help change this.)

From `./install/`, copy `sway-opener-config` to `~/.config/sway/opener-config`, then run:

```
sudo systemctl enable --now seatd
sudo loginctl enable-linger hackers
```

From `./install/`, copy `opener-app-wayland.service` to `/etc/systemd/system`, then run:

```
sudo systemctl daemon-reload
sudo systemctl enable --now opener-app-wayland
```


### Local development

If you are developing this repository locally, there are some additional packages
you will need to install.

#### For Windows

These dependencies are required by the `nfc1` crate.

Install LLVM and CMake:

```
winget install -e --id LLVM.LLVM
winget install -e --id Kitware.CMake
```

#### For macOS

Install Homebrew, and then install the following dependencies:

```
brew install libnfc pkgconf
```
