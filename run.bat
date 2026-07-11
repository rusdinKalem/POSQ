@echo off
echo Memulai aplikasi POSQ dalam Mode Development...
SET PATH=%PATH%;C:\Users\aryan\.cargo\bin
cd /d "%~dp0\apps\desktop"
npm run tauri dev
