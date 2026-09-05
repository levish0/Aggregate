set windows-shell := ["powershell.exe", "-NoLogo", "-NoProfile", "-Command"]

run:
    cargo run -p aggregate-client

check:
    cargo run -p xtask -- check

headless:
    cargo run -p xtask -- headless

fmt:
    cargo fmt --all
