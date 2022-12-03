# Very Simple Recorder (written in Rust)

For some reason my screen recording on Ubuntu is broken and `peek` has the same
problem.
- https://gitlab.gnome.org/GNOME/gnome-shell/-/issues/5585

I wanted to try Kooha, but I did not want to go through trouble installing it.
Also there are no pre-built packages.

Other alternatives did not catch my eye.

And while still waiting for the fix I have decided to make `ffmpeg` easier to
use.

I also tried to use OBS, but it is overkill. Also I did not figure out a good
way to record only part of the screen.

To start the recording, simply start the main and select screen portion to
record.

```
cargo run
```

To stop the recording, simply discard the notification :D I did not bother to
find how to create a system tray icon. I will gladly accept a PR for that. For
now "this just works good enough".

After the recording is stopped, you can open it with discarding the last
notification which will open the recording in MPV.

Recordings are saved to the `XDG_VIDEO_DIR`, with the current timestamp as a
name.

# DISCLAIMER

This is just a dirty script, I works only on Linux and most likely is specific
to my setup. With that being said, you should need ffmpeg and mpv installed.
