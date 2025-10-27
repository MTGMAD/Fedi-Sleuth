# Pixelfed Search & Download

# Fedi Sleuth

A Windows desktop application for searching and downloading content from Pixelfed instances, built with Rust and Dioxus.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## Features

### 🔍 Search Capabilities
- **User Search**: Search posts from specific Pixelfed users (supports federated users with `@username@instance.social` format)
- **Hashtag Search**: Discover posts by hashtag across your connected instance
- **Date Range Filtering**: Search posts from the last 7-365 days (default: 180 days)
- **Federated Search**: Cross-instance search with `resolve=true` for finding remote users

### 🔐 OAuth 2.0 Authentication
- Secure OAuth authentication with any Pixelfed instance
- Dynamic port allocation to avoid conflicts (no more port 8080 issues!)
- Automatic app registration and token management
- Persistent session storage

### 📥 Download Management
- Bulk download all media from search results
- Automatic organization by date (optional)
- Concurrent downloads with configurable limits
- Progress tracking

### 🎨 Modern UI
- **Dark/Light/System themes** with custom accent colors
- **Hover previews**: Full post content, images, and videos in popup preview
- **Native aspect ratios**: Media displays in original proportions
- **Video support**: Inline playback with controls
- Clean, responsive interface built with Dioxus

### ⚙️ Customization
- Theme selection (Light, Dark, System)
- Custom accent colors
- Download path configuration
- Concurrent download settings
- Organize downloads by date option

## Screenshots

### Search Interface
Search for users or hashtags with OAuth authentication and real-time results.

### Preview Popup
Hover over any result to see full post content, media thumbnails, and engagement stats.

### Settings Panel
Configure instance URL, OAuth credentials, themes, and download preferences.

## Installation

### Prerequisites
- Windows 11 (or Windows 10 with compatible runtime)
- [Rust](https://rustup.rs/) 1.70 or higher (for building from source)

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/yourusername/fedi-sleuth.git
cd fedi-sleuth
```

2. Build the release version:
```bash
cargo build --release
```

3. The executable will be at `target/release/pixelfed-rust.exe`

### Running Pre-built Binary

Download the latest release from the [Releases](https://github.com/yourusername/fedi-sleuth/releases) page and run `pixelfed-rust.exe`.

## Usage

### Getting Started

1. **Launch the application**
   - Run `pixelfed-rust.exe`

2. **Configure your Pixelfed instance**
   - Click the ⚙️ Settings button
   - Enter your Pixelfed instance URL (e.g., `pixelfed.social`)

3. **Authenticate with OAuth**
   - Click "🔑 Sign In with Pixelfed"
   - Authorize the app in your browser
   - The app will automatically receive the authentication token

4. **Start searching**
   - Switch to the 🔍 Search tab
   - Choose "User" or "Hashtag" search type
   - Enter a username (e.g., `@alice@pixelfed.social`) or hashtag (e.g., `#photography`)
   - Click "Start Search"

### Search Tips

**User Search:**
- Local users: `alice` or `@alice`
- Remote users: `@alice@pixelfed.social` or `alice@mastodon.social`
- First search for remote users may take longer (federation lookup)

**Hashtag Search:**
- Enter with or without #: `photography` or `#photography`
- Popular hashtags may timeout - try more specific hashtags
- Searches are limited to authenticated instance

**Date Range:**
- Default: 180 days back
- Adjust "Days to search back" to search older/newer content
- Longer ranges = more results but slower searches

### Download Media

1. Perform a search
2. Review results in the preview popup (hover over items)
3. Click "⬇️ Download All" button
4. Media will be saved to your configured download folder
5. Optional: Enable "Organize by date" in Settings for automatic date-based folders

## Configuration

### Settings Location
Application settings are stored at:
```
%APPDATA%\fedi-sleuth\config\settings.json
```

### OAuth Credentials
- Client ID and Client Secret are automatically generated during OAuth flow
- Access tokens are stored securely in settings
- Tokens persist between sessions

### Download Settings
- **Base Path**: Where downloaded media is saved (default: Downloads folder)
- **Max Concurrent**: Number of simultaneous downloads (default: 3)
- **Organize by Date**: Create folders by post date (default: enabled)

## Technical Details

### Built With
- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[Dioxus](https://dioxuslabs.com/)** - Reactive UI framework (v0.4)
- **[reqwest](https://github.com/seanmonstar/reqwest)** - HTTP client with OAuth2 support
- **[tokio](https://tokio.rs/)** - Async runtime
- **[serde](https://serde.rs/)** - Serialization framework

### API Compatibility
- Pixelfed Mastodon-compatible API (v1 & v2)
- OAuth 2.0 (Authorization Code flow)
- WebFinger protocol for federated user lookup

### Dependencies
All dependencies are managed by Cargo. Key dependencies include:
- `dioxus = "0.4"`
- `dioxus-desktop = "0.4"`
- `reqwest = { version = "0.11", features = ["json"] }`
- `tokio = { version = "1", features = ["full"] }`
- `oauth2 = "4.4"`
- `serde = { version = "1.0", features = ["derive"] }`
- `chrono = "0.4"`
- `anyhow = "1.0"`

See `Cargo.toml` for the complete dependency list.

## Troubleshooting

### OAuth Callback Issues
- **Port conflict**: The app now uses dynamic port allocation, automatically finding free ports
- **Browser doesn't open**: Manually copy the URL from logs and paste in browser
- **"invalid_client" error**: App re-registers automatically; try OAuth flow again

### Federation Search Issues
- **Mastodon users not found**: Pixelfed has limited Mastodon federation; try searching on a Mastodon instance instead
- **Remote user timeout**: First lookup can take 45 seconds; subsequent searches are cached
- **User not found**: Ensure full handle format: `@user@instance.domain`

### Hashtag Search Timeouts
- **Popular hashtags timeout**: These have thousands of posts; try more specific hashtags
- **Timeout after 60s**: Instance may be slow; try again or search smaller hashtag
- **No results**: Hashtag might not exist on your instance; try different instance

### Download Issues
- **Downloads fail**: Check internet connection and download folder permissions
- **Partial downloads**: Check available disk space
- **Slow downloads**: Reduce "Max Concurrent Downloads" in Settings

## Development

### Project Structure
```
pixelfed-rust/
├── src/
│   ├── main.rs              # Application entry point
│   ├── app/                 # Main app component
│   │   └── mod.rs
│   ├── components/          # UI components
│   │   ├── mod.rs
│   │   ├── search_panel.rs  # Search interface
│   │   ├── output_panel.rs  # Results display with preview
│   │   ├── settings_panel.rs # Settings and OAuth
│   │   └── status_bar.rs    # Status messages
│   ├── services/            # Business logic
│   │   ├── auth_service.rs  # OAuth 2.0 flow
│   │   ├── pixelfed_service.rs # API calls
│   │   ├── download_service.rs # Media downloads
│   │   └── settings_service.rs # Config persistence
│   ├── models/              # Data structures
│   │   └── mod.rs
│   ├── assets/              # Static resources
│   │   └── styles.css       # UI styling
│   └── utils/               # Helper functions
│       └── mod.rs
├── Cargo.toml               # Rust dependencies
├── .gitignore
└── README.md
```

### Building for Development
```bash
cargo build        # Debug build with symbols
cargo run          # Build and run debug version
cargo test         # Run tests (if any)
cargo clippy       # Lint code
```

### Building for Release
```bash
cargo build --release
```

The optimized binary will be at `target/release/pixelfed-rust.exe`.

## Roadmap

- [ ] Multi-instance support (switch between instances)
- [ ] Saved searches and bookmarks
- [ ] Export results to CSV/JSON
- [ ] Custom download filters (image/video only, min resolution)
- [ ] Batch operations (delete, archive)
- [ ] Advanced search filters (date range picker, media type)
- [ ] Linux and macOS support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Built with [Dioxus](https://dioxuslabs.com/) - A React-like framework for Rust
- Pixelfed API documentation and community
- Mastodon API compatibility layer

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/fedi-sleuth/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/fedi-sleuth/discussions)

---

**Note**: This application is not affiliated with or endorsed by Pixelfed. It's an independent client built using the public Pixelfed API.

## Features

🔍 **Search Functionality**
- Search by username or hashtag across any Pixelfed instance
- Configurable time range (default: 180 days)
- Real-time search progress and results

⬇️ **Download Management**
- Bulk download of media content
- Organized folder structure by user/hashtag and date
- Concurrent downloads with progress tracking
- Automatic file naming and organization

🔐 **Authentication**
- **Public API Mode**: No authentication required for public content search
- **OAuth Integration**: Full API access with user authentication
- **Dual Mode Support**: Switch between public and authenticated access
- **Secure Token Management**: OAuth 2.0 with PKCE for enhanced security
- **Rate Limit Handling**: Higher limits with authentication, intelligent fallback

⚙️ **Settings & Customization**
- Light/Dark/System theme support
- Customizable accent colors
- Download location configuration
- API and authentication settings

🎨 **Modern UI**
- Windows 11 design language
- Responsive layout
- Fluent animations and transitions
- Accessibility support

## Quick Start

### Prerequisites
- Windows 10/11
- Rust 1.70+ installed
- Git (optional)

### Installation

1. Clone or download this repository
2. Open terminal in the project directory
3. Build and run:
   ```bash
   cargo run
   ```

### First-Time Setup

1. **Configure Instance**: Enter your preferred Pixelfed instance URL (e.g., `pixelfed.social`)
2. **Set Download Location**: Go to Settings → Download Settings to configure where files are saved
3. **Optional OAuth**: If you want to access private content, set up OAuth in Settings → API & Authentication

## Usage

### Basic Search
1. Select search type (User or Hashtag)
2. Enter username (e.g., `@username`) or hashtag (e.g., `#photography`)
3. Set how many days back to search (default: 180)
4. Click "Start Search"

### Downloading Content
1. After search completes, review results in the output panel
2. Click "Download All" to save all media files
3. Files are saved to: `Downloads/pixelfed/[username_or_hashtag]_[date]/`

### Settings Configuration

**Appearance**
- Theme: Light, Dark, or System
- Accent Color: Choose from presets or custom color

**API & Authentication**
- Enable OAuth for private content access
- Configure Client ID and Secret from your Pixelfed app

**Downloads**
- Set custom download location
- Configure maximum concurrent downloads
- Enable/disable date-based organization

## Project Structure

```
src/
├── main.rs              # Application entry point
├── app/                 # Main application component
│   └── mod.rs
├── components/          # UI components
│   ├── mod.rs
│   ├── search_panel.rs  # Search configuration UI
│   ├── settings_panel.rs# Settings interface
│   ├── output_panel.rs  # Results display and download
│   └── status_bar.rs    # Status and progress indicator
├── models/              # Data structures
│   └── mod.rs
├── services/            # Business logic
│   ├── mod.rs
│   ├── pixelfed_service.rs  # API communication
│   ├── download_service.rs  # File download management
│   ├── settings_service.rs  # Configuration persistence
│   └── auth_service.rs      # OAuth authentication
├── assets/              # Static assets
│   └── styles.css       # Application styling
├── config/              # Configuration utilities
│   └── mod.rs
└── utils/               # Helper functions
    └── mod.rs
```

## Development

### Building
```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Running
```bash
# Run in development mode
cargo run

# Run release build
cargo run --release
```

### Testing
```bash
# Run tests
cargo test

# Run with output
cargo test -- --nocapture
```

## Dependencies

### Core Framework
- **dioxus**: Modern React-like framework for Rust
- **dioxus-desktop**: Desktop application support
- **tokio**: Async runtime

### HTTP & API
- **reqwest**: HTTP client for API calls
- **oauth2**: OAuth 2.0 authentication
- **serde**: JSON serialization

### UI & System
- **dirs**: Cross-platform directory access
- **confy**: Configuration file management
- **chrono**: Date/time handling

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test thoroughly
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- Pixelfed community for the federated image sharing platform
- Dioxus team for the excellent Rust UI framework
- Windows 11 design team for the modern UI inspiration

## Support

For issues, feature requests, or questions:
1. Check existing GitHub issues
2. Create a new issue with detailed information
3. Include system information and error logs

---

Built with ❤️ using Rust and Dioxus