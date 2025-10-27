# Development Requirements

## System Requirements

### Windows
- Windows 10 (version 1903 or later) or Windows 11
- 64-bit processor
- Minimum 4GB RAM (8GB recommended)
- 500MB free disk space

### Build Tools
- **Rust**: 1.70 or higher
  - Install from: https://rustup.rs/
  - Includes: `rustc`, `cargo`, and standard library
- **Visual Studio Build Tools** (Windows)
  - Required for compiling native dependencies
  - Install from: https://visualstudio.microsoft.com/downloads/
  - Select "Desktop development with C++"

## Rust Dependencies

All Rust dependencies are automatically managed by Cargo and specified in `Cargo.toml`.

### Core Dependencies
```toml
dioxus = "0.4"                    # UI framework
dioxus-desktop = "0.4"            # Desktop platform support
reqwest = { version = "0.11", features = ["json"] }  # HTTP client
tokio = { version = "1", features = ["full"] }       # Async runtime
```

### Authentication & API
```toml
oauth2 = "4.4"                    # OAuth 2.0 client
serde = { version = "1.0", features = ["derive"] }   # Serialization
serde_json = "1.0"                # JSON support
```

### Utilities
```toml
anyhow = "1.0"                    # Error handling
chrono = "0.4"                    # Date/time handling
urlencoding = "2.1"               # URL encoding
regex = "1.10"                    # Text processing
```

### Configuration
```toml
confy = "0.5"                     # Config file management
dirs = "5.0"                      # Standard directories
```

### Additional Features
```toml
log = "0.4"                       # Logging facade
env_logger = "0.10"               # Logger implementation
```

## Installation Steps

### 1. Install Rust
```powershell
# Download and run rustup-init.exe from https://rustup.rs/
# Or use PowerShell:
Invoke-WebRequest -Uri https://win.rustup.rs/x86_64 -OutFile rustup-init.exe
.\rustup-init.exe
```

### 2. Verify Installation
```powershell
rustc --version
cargo --version
```

### 3. Clone Repository
```powershell
git clone https://github.com/yourusername/fedi-sleuth.git
cd fedi-sleuth
```

### 4. Build Project
```powershell
# Development build
cargo build

# Release build (optimized)
cargo build --release
```

### 5. Run Application
```powershell
# Development
cargo run

# Release
.\target\release\pixelfed-rust.exe
```

## Optional Development Tools

### Code Quality
```powershell
# Linter
rustup component add clippy
cargo clippy

# Formatter
rustup component add rustfmt
cargo fmt

# Security audit
cargo install cargo-audit
cargo audit
```

### IDE Support
- **Visual Studio Code**
  - Extensions: rust-analyzer, CodeLLDB
- **IntelliJ IDEA / CLion**
  - Plugin: Rust

## Runtime Dependencies

The compiled application has minimal runtime dependencies:
- **WebView2 Runtime** (Windows)
  - Usually pre-installed on Windows 11
  - Download: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

All other dependencies are statically linked into the executable.

## Network Requirements

- **Internet connection** for:
  - OAuth authentication
  - API requests to Pixelfed instances
  - Downloading media content
- **Firewall**: Allow outbound HTTPS (port 443)
- **Proxy**: Respects system proxy settings

## Disk Space

- **Source code**: ~50MB
- **Build artifacts**: ~500MB (debug), ~50MB (release)
- **Downloaded media**: Varies based on usage
- **Application data**: <1MB (config and cache)

## Troubleshooting Build Issues

### Linker Errors
```powershell
# Install Visual Studio Build Tools
# Restart shell and try again
```

### OpenSSL Errors (if encountered)
```powershell
# Use native-tls (already configured in Cargo.toml)
# No additional steps needed
```

### WebView2 Errors
```powershell
# Install WebView2 Runtime
# Download from Microsoft
```

### Slow Compilation
```powershell
# Use faster linker (Windows)
cargo install -f cargo-binutils
rustup component add llvm-tools-preview
```

## Platform Notes

### Windows-Specific
- Uses `native-tls` for HTTPS
- Requires WebView2 for rendering
- Default file paths use Windows conventions

### Future Platforms (Not Yet Supported)
- Linux: Will require `webkit2gtk` and `libssl-dev`
- macOS: Will require Xcode Command Line Tools

## Updating Dependencies

```powershell
# Update all dependencies to latest compatible versions
cargo update

# Check for outdated dependencies
cargo install cargo-outdated
cargo outdated

# Update Rust toolchain
rustup update
```
