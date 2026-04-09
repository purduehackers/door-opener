# Install

## Download `door-opener` binary

Run:

```
mkdir -p ~/door-opener
cd ~/door-opener
wget https://github.com/purduehackers/door-opener/releases/download/latest/openerapp_aarch64
chmod +x openerapp_aarch64
```

## Configure `sway`

From `./install/`, copy `sway-opener-config` to `~/.config/sway/opener-config`, then run:

```
sudo systemctl enable --now seatd
sudo loginctl enable-linger hackers
```

## Configure `systemd` service

From `./install/`, copy `opener-app-wayland.service` to `/etc/systemd/system`, then run:

```
sudo systemctl daemon-reload
sudo systemctl enable --now opener-app-wayland
```

You should now see `door-opener` running!
