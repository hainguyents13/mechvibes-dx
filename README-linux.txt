MechvibesDX for Linux
=====================

This archive contains a portable build:

  mechvibes-dx          the executable
  soundpacks/           bundled keyboard and mouse soundpacks
  mechvibes-dx.desktop  optional application menu entry
  mechvibes-dx.png      application icon


1. Input permissions (REQUIRED - read this first)
-------------------------------------------------

MechvibesDX reads key events from /dev/input/event*, which is owned by the
"input" group. Without membership the app starts but stays silent - no error,
just no sound.

  sudo usermod -a -G input $USER

Then LOG OUT AND LOG BACK IN. The group change only applies to new sessions;
`newgrp input` in a terminal is not enough for a GUI app launched from your
desktop session.

Verify with:

  groups $USER          # should list "input"
  ls -la /dev/input/event*   # should show "crw-rw---- root input"

The .deb package does NOT do this for you either - it ships no maintainer
scripts on purpose, so nothing modifies your system's groups behind your back.


2. Runtime dependencies
-----------------------

The binary is dynamically linked. On Debian/Ubuntu:

  sudo apt-get install libasound2 libevdev2 libxdo3 \
                       libwebkit2gtk-4.1-0 libgtk-3-0 \
                       libayatana-appindicator3-1 librsvg2-2

On Fedora/RHEL the equivalents are alsa-lib, libevdev, xdotool-libs,
webkit2gtk4.1, gtk3, libappindicator-gtk3 and librsvg2.

If you would rather have dependencies resolved for you, use the .deb from the
same release instead of this tarball.


3. Running it
-------------

  ./mechvibes-dx

Run it from inside this directory: the app looks for soundpacks/ relative to
the executable.

To install it system-wide by hand:

  sudo cp mechvibes-dx /usr/local/bin/
  sudo mkdir -p /usr/share/mechvibes-dx
  sudo cp -r soundpacks /usr/share/mechvibes-dx/
  sudo cp mechvibes-dx.desktop /usr/share/applications/
  sudo cp mechvibes-dx.png /usr/share/icons/hicolor/512x512/apps/


4. Troubleshooting
------------------

No sound at all
  Almost always the "input" group step above, or a session that was not
  restarted after it. Check `groups $USER` first.

No tray icon
  Install your desktop's AppIndicator support (GNOME needs the
  AppIndicator/KStatusNotifierItem extension).

Wayland
  Global input capture reads evdev directly, so it works under both X11 and
  Wayland once the group membership is in place.
