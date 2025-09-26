# Project Structure

## Root Level
```
├── backend/           # Rust API server
├── frontend/          # Svelte web application  
├── docker-compose.yml # Development infrastructure
├── Makefile          # Development commands
└── README.md         # Project documentation
```

## Backend Structure (`backend/`)

### Core Organization
```
backend/
├── src/
│   ├── main.rs           # Application entry point & route handlers
│   ├── lib.rs            # Library exports
│   ├── models/           # Data structures & database models
│   ├── services/         # Business logic layer
│   └── middleware/       # HTTP middleware (auth, CORS, etc.)
├── tests/               # Integration & unit tests
├── migrations/          # Database schema migrations
├── Cargo.toml          # Dependencies & metadata
└── .env                # Environment configuration
```

### Key Modules

#### Models (`src/models/`)
- `user.rs` - User accounts and authentication
- `artist.rs` - Artist entities and external ID mapping
- `auth.rs` - Authentication request/response types
- `dnp_list.rs` - Personal blocklist models
- `community_list.rs` - Shared list models
- `spotify.rs` - Spotify-specific models
- `token_vault.rs` - Encrypted token storage
- `action.rs` - Enforcement action tracking

#### Services (`src/services/`)
- `auth.rs` - User authentication & session management
- `entity_resolution.rs` - Artist matching across platforms
- `dnp_list.rs` - Personal blocklist management
- `community_list.rs` - Community list operations
- `spotify.rs` - Spotify OAuth & API integration
- `spotify_enforcement.rs` - Spotify content removal
- `spotify_library.rs` - Library scanning & analysis
- `token_vault.rs` - Secure token storage
- `external_apis.rs` - Third-party API integrations

#### Middleware (`src/middleware/`)
- `auth.rs` - JWT validation & user extraction
- Rate limiting and CORS handling

## Frontend Structure (`frontend/`)

### Core Organization
```
frontend/
├── src/
│   ├── App.svelte        # Root component
│   ├── main.ts          # Application entry point
│   ├── lib/
│   │   ├── components/   # Svelte components
│   │   ├── stores/      # State management
│   │   └── utils/       # Shared utilities
├── public/              # Static assets
├── package.json         # Dependencies & scripts
└── rollup.config.js     # Build configuration
```

### Component Organization (`src/lib/components/`)
- `Login.svelte` - Authentication interface
- `Dashboard.svelte` - Main application layout
- `DnpManager.svelte` - Personal blocklist management
- `CommunityLists.svelte` - Community list browser
- `EnforcementPlanning.svelte` - Enforcement workflow
- `ServiceConnections.svelte` - Platform connection management

### State Management (`src/lib/stores/`)
- `auth.ts` - Authentication state & actions
- `dnp.ts` - Personal blocklist state
- `community.ts` - Community list state  
- `enforcement.ts` - Enforcement workflow state
- `connections.ts` - Service connection state

## Database Structure

### Migrations (`backend/migrations/`)
- `001_initial_schema.sql` - Core tables (users, artists, connections)
- `002_indexes.sql` - Performance indexes

### Key Tables
- `users` - User accounts with encrypted fields
- `artists` - Artist catalog with external ID mapping
- `user_artist_blocks` - Personal DNP entries
- `community_lists` - Shared blocklists
- `community_list_subscriptions` - User subscriptions
- `action_batches` - Enforcement operation tracking
- `audit_log` - SOC2 compliance logging

## Naming Conventions

### Rust
- **Files**: `snake_case.rs`
- **Structs**: `PascalCase`
- **Functions**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`

### Frontend
- **Components**: `PascalCase.svelte`
- **Stores**: `camelCase.ts`
- **Functions**: `camelCase`

### Database
- **Tables**: `snake_case`
- **Columns**: `snake_case`
- **Indexes**: `idx_table_column`

## Testing Structure

### Backend Tests (`backend/tests/`)
- `integration_tests.rs` - Full API integration tests
- `*_tests.rs` - Service-specific unit tests
- Test naming: `test_function_name_scenario`

### Test Data
- Use factories for consistent test data
- Mock external APIs with `wiremock`
- Database tests use transactions for isolation