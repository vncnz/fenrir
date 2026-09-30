# Fenrir - ᚠᛖᚾᚱᛁᚱ

## About the name

Skoll is, in norse mythology, a wolf who chases the sun, causing eclipses. And it is my gui launcher. So this is Fenrir, Skoll's brother, and it is my TUI launcher.

## UI preview

![image](screenshots/preview_crop.png)

The upper area provides a compact snapshot of the current system state.

The lower area is the actual launcher: a filter and a box containing search results. For each result, you get name and the real command. If the app has an icon, it is painter in the lower-right corner. The last couple of lines of the TUI contains the description of the application selected currently.

## About this project

Applications filter is resistent to typos and and missing keystrokes, and results sorting is based on launch history.

After some fine-tuning, on my laptop the TUI appears in ~500µs and app list is ready in ~100ms. You can start typing immediately: your keystrokes won't get lost while the list loads.
if you are not running Fenrir in an already open terminal, remember to account for the terminal's startup time. In that case, I suggest you to use kitty with --single-instance mode for a blazing fast experience.

Information about system resources is collected from another process: Ratatoskr. This is a public project that you can find on my GitHub account, its goal is to gather all system resources information and write it to a single json in /tmp folder. If Ratatoskr isn't up and running, you'll see a warning/hint in the system resources area, but the launcher stays fully usable.

## Screenshots

High avgload, "cod" in filter:
![image](screenshots/high_avg.png)

Slightly high memory usage and temperature, empty filter:
![image](screenshots/medium_high_memory_temp_crop.png)

Very high memory usage and high temperature:
![image](screenshots/very_high_memory_temp_crop.png)

With bluetooth headphones connected (headphone battery 100%) and "cal" in filter:
![image](screenshots/with_headphones_crop.png)

## Note

Please note that this is a personal project, for personal use, developed in my (not so much) free time. You'll not find clean code or a flexible, modular system here. You'll find lots of experiments, abandoned ideas, dead code, temporary hacks and workarounds. Oh, and last but not least, I'm just learning both Rust and RataTUI. You've been warned.

## TODO

- ~~Insert weather information~~
- ~~Full network information, like IP and networkname, on a dedicated row~~ Done!
- ~~Fix resources information order~~ Done!
- ~~Add battery ETA if available~~ Done!
- ~~Fuzzy search~~ Done!
- ~~History~~ Done!
