#!/bin/bash

echo "Memulai aplikasi POSQ dalam Mode Development..."

# Menambahkan path cargo jika diperlukan (default path di Linux)
export PATH="$PATH:$HOME/.cargo/bin"

# Pindah ke direktori desktop yang berada di bawah direktori script ini
cd "$(dirname "$0")/apps/desktop" || exit 1

# Menjalankan tauri dev
npm run tauri dev
