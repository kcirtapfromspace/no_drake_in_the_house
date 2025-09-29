# Frontend - No Drake in the House

A Svelte-based web application for managing music streaming blocklists.

## Features

- **TypeScript Configuration**: Full TypeScript support with strict type checking
- **Tailwind CSS**: Utility-first CSS framework for rapid UI development
- **Hot Reloading**: Development server with automatic reload on file changes
- **Basic Routing**: Simple client-side routing system
- **Environment-based API Configuration**: Configurable API endpoints via environment variables

## Development Setup

### Prerequisites

- Node.js 18+ 
- npm or yarn

### Installation

```bash
# Install dependencies
npm install

# Copy environment configuration
cp .env.example .env

# Start development server
npm run dev
```

The application will be available at `http://localhost:8080`

### Available Scripts

- `npm run dev` - Start development server with hot reloading
- `npm run build` - Build for production
- `npm run start` - Serve production build
- `npm run check` - Run TypeScript type checking

## Project Structure

```
src/
├── lib/
│   ├── components/     # Svelte components
│   ├── stores/        # State management (Svelte stores)
│   └── utils/         # Utilities (API client, router, config)
├── App.svelte         # Root component
├── main.ts           # Application entry point
└── vite-env.d.ts     # TypeScript environment definitions
```

## Configuration

### Environment Variables

The application uses environment variables for configuration:

- `VITE_API_URL` - Backend API URL (default: http://localhost:3000)
- `VITE_API_VERSION` - API version (default: v1)
- `VITE_APP_NAME` - Application name
- `VITE_ENVIRONMENT` - Environment (development/production)

See `.env.example` for all available configuration options.

### API Client

The API client automatically:
- Adds authentication headers when tokens are available
- Handles API versioning and endpoint normalization
- Provides consistent error handling
- Uses environment-based configuration

### Routing

Simple client-side routing with:
- URL-based navigation
- Browser history support
- Dynamic page titles
- Route-based component rendering

## Components

### Core Components

- `Login.svelte` - Authentication interface
- `Dashboard.svelte` - Main application layout
- `Navigation.svelte` - Top navigation and tabs
- `DnpManager.svelte` - DNP list management
- `ServiceConnections.svelte` - Platform connections

### State Management

Uses Svelte stores for reactive state:
- `auth.ts` - Authentication state and actions
- `dnp.ts` - DNP list state and operations
- `connections.ts` - Service connection state
- `community.ts` - Community list state
- `enforcement.ts` - Enforcement workflow state

## Build System

- **Rollup** - Module bundler with Svelte plugin
- **TypeScript** - Compile-time type checking
- **PostCSS** - CSS processing with Tailwind
- **Autoprefixer** - Automatic vendor prefixes

## Development Guidelines

### TypeScript

- Use strict type checking
- Define interfaces for all data structures
- Avoid `any` types where possible
- Use proper typing for component props

### Styling

- Use Tailwind CSS utility classes
- Follow responsive design principles
- Maintain consistent spacing and colors
- Use semantic HTML elements

### Components

- Keep components focused and reusable
- Use proper prop typing
- Handle loading and error states
- Follow Svelte best practices

## Production Build

```bash
# Build for production
npm run build

# Serve production build
npm run start
```

The build output is generated in `public/build/` and served from the `public/` directory.