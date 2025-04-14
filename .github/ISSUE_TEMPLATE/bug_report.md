---
name: Bug report
about: Create a bug report
title: 'BUG: the bug you faced while using the application'
labels: 'bug'
assignees: ''

---

## Describe the bug

A clear and concise description of what the bug is.

## To Reproduce

NOTE: Before opening a bug report, make sure that it is reproducible on a fresh install. 

This involves:
- Uninstalling the application
- Deleting the `com.crux-bphc.multipartus-downloader` directories for app data and cache, which are located in:
   - `C://Users/<username>/AppData/Local` on Windows
   - `~/.local/share` & `~/.cache` on Linux
   - `Library/Application Support` & `Library/Caches` on Mac
- Installing the latest release

Steps to reproduce the behavior:
1. Go to '...'
2. Click on '....'
3. Scroll down to '....'
4. See error

## Expected behavior
A clear and concise description of what you expected to happen.

## Screenshots
If applicable, add screenshots to help explain your problem.

## System Info:
 - OS: [e.g. Windows]
 - Browser [e.g. chrome, safari]
 - App Version [e.g. v0.0.3]
 - Connection [e.g. LAN]
 - Using a VPN?

## Logs
Please reproduce the issue at least once, then attach the most recent log file (if applicable).

This can be found in:
 - `/tmp/multipartus-downloader/logs` for Linux, 
 - `C:/Users/<your-username>/AppData/Local/Temp/multipartus-downloader/logs` for Windows
 - most likely `~Library/Caches/multipartus-downloader/logs` on MacOS - if not here, then open up your terminal and type in `open $TMPDIR` and navigate to `multipartus-downloader/logs`

## Additional context
Add any other context about the problem here.
