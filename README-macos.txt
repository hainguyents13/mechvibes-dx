MechvibesDX for macOS - EXPERIMENTAL, UNTESTED
==============================================

READ THIS BEFORE INSTALLING.

This build is produced by CI and has never been run on a real Mac by the
maintainer. It is not code-signed and not notarized. It is published so that
macOS users can try it and report back - not because it is known to work.

Known and expected rough edges:
  - Gatekeeper will refuse to open it until you clear the quarantine flag.
  - Input capture needs Accessibility permission, which the app cannot
    request properly without a signed bundle.
  - The tray icon, audio device switching, and soundpack loading are all
    unverified on macOS.

If something is broken, please open an issue with the macOS version and the
console output - that feedback is the whole point of this build.


1. Clearing Gatekeeper
----------------------

Because the app is unsigned, macOS quarantines it on download. Remove the
flag on the extracted files:

  xattr -cr /path/to/mechvibes-dx

For a .app bundle you can also right-click it in Finder, choose "Open", and
confirm the warning dialog - this whitelists that one app.

If macOS reports the app "is damaged and can't be opened", that is the same
quarantine problem, not actual corruption; the xattr command above fixes it.


2. Accessibility permission
---------------------------

Global key capture requires it:

  System Settings -> Privacy & Security -> Accessibility

Add the mechvibes-dx executable (or the .app bundle) and enable the toggle.
You may have to remove and re-add the entry after replacing the binary with a
newer build, because macOS keys the permission to the binary's identity.


3. Running it
-------------

Tarball build (bare binary):

  cd mechvibes-dx
  xattr -cr .
  ./mechvibes-dx

Run it from inside the extracted directory - the app looks for soundpacks/
relative to the executable.

App bundle build (.app / .dmg): drag to /Applications, then apply the xattr
command to the bundle before first launch.


4. Architecture
---------------

The asset filename ends in the CPU architecture it was built for:

  arm64   Apple Silicon (M1 and later)
  x86_64  Intel

There is no universal binary yet. If your architecture is not published in a
release, build from source: `cargo build --release`.


5. Updates
----------

The in-app auto-updater only handles the Windows installer. On macOS,
download new releases manually from the project's GitHub Releases page.
