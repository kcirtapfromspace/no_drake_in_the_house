# Product Overview

## Music Streaming Blocklist Manager

A multi-platform music streaming blocklist management system that provides centralized control for users to avoid specific artists across streaming services.

### Core Features

- **Personal DNP Lists**: Users can create and manage "Do Not Play" lists of artists they want to avoid
- **Community Lists**: Shared blocklists that users can subscribe to and contribute to
- **Multi-Platform Support**: Currently supports Spotify with architecture for additional streaming services
- **Enforcement Engine**: Automatically removes blocked content from user libraries and playlists
- **Entity Resolution**: Smart artist matching across different platforms and naming variations

### Key User Flows

1. **Authentication**: Secure user registration with optional 2FA
2. **Service Connection**: OAuth integration with streaming platforms
3. **List Management**: Add/remove artists from personal and community lists
4. **Enforcement**: Scan libraries, plan changes, and execute removals
5. **Community Engagement**: Browse, subscribe to, and contribute to shared lists

### Architecture Goals

- Security-first design with encrypted token storage
- Scalable microservices architecture
- Real-time enforcement capabilities
- Comprehensive audit logging for compliance
- Extensible platform adapter system