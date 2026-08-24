Anzoth CLI release scripts

Production scripts:
- build-win-x86_64.bat
- build-linux-x86_64.bat
- build-macos-x86_64.bat
- build-macos-arm64-on-intel.bat
- build-macos-arm64-on-m1.bat

Behavior:
- optimized Cargo release build
- release debuginfo disabled
- normal public-release workflow
- production staging directories

Debug-symbol scripts:
- build-win-x86_64-debug.bat
- build-linux-x86_64-debug.bat
- build-macos-x86_64-debug.bat
- build-macos-arm64-on-intel-debug.bat
- build-macos-arm64-on-m1-debug.bat

Behavior:
- optimized Cargo release build
- release debuginfo retained
- no stripping when symbols should be preserved
- separate debug staging directories
- use only when explicitly debugging

Notes:
- Windows production uses -j 20
- Linux/macOS use Cargo host-default parallelism unless otherwise specified
- Codex must not execute these scripts
- release/debug compilation is performed manually in a visible terminal
- Expected SSH aliases: anzoth-dev, mac, mac-m1
- The mac-m1 script defaults to ~/anzoth-mac-validation
- Pass another repo path as argument if needed
