# Fedi Sleuth

A Windows desktop application for searching and downloading content from **Pixelfed, Mastodon, and Bluesky**, built with Rust and Dioxus.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## Features

### 🌐 Multi-Platform Support
- **Pixelfed**: Search and download from any Pixelfed instance with OAuth 2.0
- **Mastodon**: Access the Fediverse with OAuth authentication
- **Bluesky**: Connect to the ATProto network with app passwords
- **Unified Search**: Query all platforms simultaneously with grouped results
- **Platform Toggles**: Enable/disable each platform independently

### 🔍 Search Capabilities
- **User Search**: Find posts from specific users across platforms (supports `@username@instance.social` format)
- **Hashtag Search**: Discover posts by hashtag on each connected platform
- **Date Range Filtering**: Search posts from the last 7-365 days (default: 60 days)
- **Federated Search**: Cross-instance lookup with full federation support
- **Real-time Results**: Live progress tracking and grouped platform results

### 🔐 Authentication
- **Pixelfed OAuth 2.0**: Secure browser-based authentication with automatic app registration
- **Mastodon OAuth 2.0**: Same streamlined OAuth flow as Pixelfed
- **Bluesky ATProto**: Simple handle + app password authentication
- **Dynamic Port Allocation**: No more port conflicts - finds free ports automatically
- **Persistent Sessions**: Credentials saved securely between app launches
- **Per-Platform Control**: Enable/disable and authenticate each platform independently

### 📥 Download Management
- **Multi-Platform Downloads**: Organized by platform and search context
- **Smart Folder Structure**: `Downloads/Fedi_Sleuth/{Platform}/{username_or_hashtag}_{date}/`
- **Bulk Operations**: Download all media from search results across all platforms
- **Concurrent Downloads**: Configurable simultaneous download limits
- **Progress Tracking**: Real-time download progress with percentage indicators
- **Automatic Organization**: Date-based folders with platform separation

### 🎨 Modern UI
- **Dark/Light/System themes** with custom accent colors
- **Platform-Grouped Results**: Results organized by Pixelfed 🟣, Mastodon 🐘, and Bluesky 🦋
- **Hover previews**: Full post content, images, and videos in popup preview
- **Native aspect ratios**: Media displays in original proportions
- **Video support**: Inline playback with controls for all platforms
- **Responsive interface**: Clean, modern design built with Dioxus
- **Per-Platform Stats**: See result counts and errors for each platform

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

2. **Configure Your Platforms**

   **Pixelfed (OAuth)**
   - Go to Settings → API & Authentication → Pixelfed
   - Enter your instance URL (e.g., `pixelfed.social`)
   - Enable Pixelfed and click "🔑 Sign In with Pixelfed"
   - Authorize in your browser - the app will automatically receive the token

   **Mastodon (OAuth)**
   - Go to Settings → API & Authentication → Mastodon  
   - Enter your instance URL (e.g., `mastodon.social`)
   - Enable Mastodon and click "🔑 Sign In with Mastodon"
   - Authorize in your browser - same OAuth flow as Pixelfed

   **Bluesky (App Password)**
   - Go to Settings → API & Authentication → Bluesky
   - Enable Bluesky
   - Enter your handle (e.g., `yourname.bsky.social`)
   - Create an app password at bsky.app → Settings → App Passwords
   - Paste the app password and click "🔑 Sign In with Bluesky"

3. **Start Searching**
   - Switch to the 🔍 Search tab
   - Choose platforms to search (checkboxes next to Pixelfed/Mastodon/Bluesky)
   - Select "User" or "Hashtag" search type
   - Enter a username (e.g., `@alice@pixelfed.social`) or hashtag (e.g., `#photography`)
   - Set days back to search (7-365, default: 60)
   - Click "Start Search"
   
4. **View Results**
   - Results are grouped by platform with emoji indicators
   - Each platform section shows post count or error messages
   - Hover over results for full preview popups
   - See total posts across all platforms at the top

5. **Download Media**
   - Click "⬇️ Download All" to save media from all platforms
   - Files organized: `Downloads/Fedi_Sleuth/{Platform}/{query}_{timestamp}/`
   - Watch real-time progress as downloads complete

### Search Tips

**Platform Selection:**
- Check/uncheck platforms in the search panel to control which are queried
- Only enabled AND authenticated platforms will be searched
- Results are grouped by platform for easy comparison

**User Search:**
- Local users: `alice` or `@alice`
- Remote users: `@alice@pixelfed.social`, `@alice@mastodon.social`, or `alice.bsky.social`
- First search for remote users may take longer (federation lookup)
- Bluesky requires exact handle format (e.g., `name.bsky.social`)

**Hashtag Search:**
- Enter with or without #: `photography` or `#photography`
- Bluesky uses different hashtag indexing - results may vary
- Popular hashtags may timeout - try more specific tags
- Each platform searches its own federated timeline

**Date Range:**
- Default: 60 days back
- Adjust "Days to search back" slider (7-365 days)
- Longer ranges = more results but slower searches
- Each platform enforces its own API rate limits

### Download Media

1. Perform a multi-platform search
2. Review results - grouped by Pixelfed 🟣, Mastodon 🐘, Bluesky 🦋
3. Hover over items to preview content
4. Click "⬇️ Download All" button
5. Media saved to: `Downloads/Fedi_Sleuth/{Platform}/{query}_{timestamp}/`
   - Example: `Downloads/Fedi_Sleuth/Pixelfed/linux2073_20251207/`
   - Example: `Downloads/Fedi_Sleuth/Mastodon/photography_20251207/`
6. Watch progress bar for real-time download status

## Configuration

### Settings Location
Application settings are stored at:
```
%APPDATA%\fedi-sleuth\config\settings.json
```

### OAuth Credentials (Pixelfed & Mastodon)
- Client ID and Client Secret are automatically generated during OAuth flow
- Access tokens stored securely in settings
- Tokens persist between sessions
- Each platform maintains independent OAuth credentials

### Bluesky Credentials
- Requires handle and app password (not your main password!)
- Create app passwords at: https://bsky.app → Settings → App Passwords
- App passwords have limited scope for security
- DIDs and session tokens stored after successful authentication

### Download Settings
- **Base Path**: Where downloaded media is saved (default: Downloads folder)
- **Max Concurrent**: Number of simultaneous downloads (default: 3)
- **Organize by Date**: Create folders by post date (default: enabled)

## Technical Details

### Built With
- **[Rust](https://www.rust-lang.org/)** - Systems programming language
- **[Dioxus](https://dioxuslabs.com/)** - Reactive UI framework (v0.4)
- **[reqwest](https://github.com/seanmonstar/reqwest)** - HTTP client with OAuth2 support
- **[tokio](https://tokio.rs/)** - Async runtime for concurrent operations
- **[serde](https://serde.rs/)** - Serialization framework for JSON

### API Compatibility
- **Pixelfed**: Mastodon-compatible API (v1 & v2) with OAuth 2.0
- **Mastodon**: Native API (v1 & v2) with OAuth 2.0
- **Bluesky**: ATProto (app.bsky.* lexicons) with session authentication
- WebFinger protocol for federated user lookup (Pixelfed/Mastodon)

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
- **Port conflict**: The app uses dynamic port allocation, automatically finding free ports
- **Browser doesn't open**: Manually copy the URL from logs and paste in browser
- **"invalid_client" error**: App re-registers automatically; try OAuth flow again
- **Mastodon OAuth**: Uses same flow as Pixelfed - automatic app registration

### Bluesky Authentication Issues
- **"Invalid handle" error**: Ensure exact format (e.g., `name.bsky.social`)
- **"Invalid password"**: Must use app password, not your main account password
- **Create app password**: Go to https://bsky.app → Settings → App Passwords → Add App Password
- **Session expired**: Re-authenticate by clicking "Sign In with Bluesky" again

### Federation Search Issues
- **Mastodon users on Pixelfed**: Limited federation; try searching on Mastodon instance directly
- **Pixelfed users on Mastodon**: Usually works, but initial lookup may be slow
- **Bluesky users**: Not federated with ActivityPub - Bluesky search is separate
- **Remote user timeout**: First lookup can take 45 seconds; subsequent searches are cached
- **User not found**: Ensure correct handle format for each platform

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

- [x] Multi-platform support (Pixelfed, Mastodon, Bluesky)
- [x] OAuth 2.0 for Pixelfed and Mastodon
- [x] Bluesky ATProto authentication
- [x] Unified multi-platform search
- [x] Platform-grouped results display
- [x] Multi-platform download organization
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
- Pixelfed, Mastodon, and Bluesky communities for their open APIs
- ActivityPub and ATProto protocols for decentralized social networking
- Mastodon API compatibility layer used by Pixelfed

## Support

- **Issues**: [GitHub Issues](https://github.com/yourusername/fedi-sleuth/issues)
- **Discussions**: [GitHub Discussions](https://github.com/yourusername/fedi-sleuth/discussions)

---

**Note**: This application is not affiliated with or endorsed by Pixelfed, Mastodon, or Bluesky. It's an independent client built using their public APIs.

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