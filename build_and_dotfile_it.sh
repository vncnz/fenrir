#!/bin/bash

# This script builds fenrir and then copies the exec to ~/.config/niri

cargo build --release;
cp ~/Repositories/fenrir/target/release/fenrir ~/.config/niri/;
echo -e "\n\033[0;32m\033[1mfenrir built and copied to ~/.config/niri\033[0m";