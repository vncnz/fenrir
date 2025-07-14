# Fenrir
Skoll is, in norse mythology, a wolf who chases the sun, causing eclipses. And is my gui launcher. So this is Fenrir, Skoll's brother, and is my TUI launcher.

After some fine-tuning, on my laptop the TUI appears in ~500µs and app list is ready in ~100ms. You can start typing immediately: your keystrokes won't get lost while the list loads.
if you are not running Fenrir in an already open terminal, remember to account for the terminal's startup time. In that case, I suggest you to use kitty with --single-instance mode for a blazing fast experience.

Information about system resources is collected from another process: Ratatoskr. This is a public project that you can find on my GitHub account, its goal is to gather all system resources information and write it to a single json in /tmp folder. If Ratatoskr isn't up and running, you'll see a warning/hint in the system resources area, but the launcher stays fully usable.

Please note that this is a personal project, for personal use, developed in my (not so much) free time. You'll not find clean code or a flexible, modular system here. You'll find lots of experiments, abandoned ideas, dead code, temporary hacks and workarounds. Oh, and last but not least, I'm just learning both Rust and GTK. You've been warned.

# To be added
- Network info
- Brightness?
- Follow volume changes instead of polling?

### known bugs
<!-- [ ] Nerd icons are too small: using ttf-roboto-mono-nerd?-->
- Sometimes the app icon don't show up