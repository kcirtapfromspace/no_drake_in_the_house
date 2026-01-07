# Design Document

## Overview

This design creates a streamlined user experience focused on two core features: service connections and blocklist management. The interface will be clean, fast, and intuitive, removing complexity and focusing on the essential user workflows.

## Architecture

### Simplified Navigation Structure

```
App
├── Connections (Primary)
│   ├── Connection Cards (Spotify, Apple Music, etc.)
│   ├── Connection Status
│   └── Setup Instructions
└── Blocklist (Primary)
    ├── Add Artist Search
    ├── Blocked Artists Feed
    └── Quick Actions
```

### Component Hierarchy

```
App.svelte
├── Navigation.svelte (Simplified)
├── ConnectionsPage.svelte (New)
│   ├── ConnectionCard.svelte
│   └── ConnectionStatus.svelte
└── BlocklistPage.svelte (New)
    ├── ArtistSearch.svelte (Simplified)
    ├── BlocklistFeed.svelte (New)
    └── BlockedArtistCard.svelte (New)
```

## Components and Interfaces

### 1. Simplified Navigation

**Design Principles:**
- Only two main sections: Connections and Blocklist
- Clean, minimal design with clear active states
- Mobile-first responsive design
- Instant navigation with smooth transitions

**Interface:**
```typescript
interface NavigationProps {
  currentPage: 'connections' | 'blocklist';
  onNavigate: (page: string) => void;
}
```

### 2. Connections Page

**Design Principles:**
- Focus on connection status and setup
- Clear visual indicators for connected/disconnected states
- Prominent call-to-action buttons
- Step-by-step guidance for setup

**Features:**
- Large connection cards for each service
- Clear status indicators (Connected, Disconnected, Error)
- One-click connection buttons
- Connection health monitoring
- Simple disconnect options

**Interface:**
```typescript
interface ConnectionsPageProps {
  connections: ServiceConnection[];
  onConnect: (provider: string) => void;
  onDisconnect: (provider: string) => void;
}
```

### 3. Blocklist Page (Feed Style)

**Design Principles:**
- Feed-style interface similar to social media
- Prominent search at the top
- Immediate visual feedback
- Easy add/remove actions

**Features:**
- Sticky search bar at top
- Real-time search results
- Feed of blocked artists with images
- Quick unblock actions
- Empty states with helpful guidance

**Interface:**
```typescript
interface BlocklistPageProps {
  blockedArtists: DnpEntry[];
  searchResults: Artist[];
  onAddArtist: (artist: Artist) => void;
  onRemoveArtist: (artistId: string) => void;
  onSearch: (query: string) => void;
}
```

## Data Models

### Simplified Service Connection

```typescript
interface ServiceConnection {
  provider: 'spotify' | 'apple' | 'youtube';
  status: 'connected' | 'disconnected' | 'error' | 'connecting';
  displayName: string;
  icon: string;
  description: string;
  lastConnected?: string;
  errorMessage?: string;
}
```

### Simplified Blocked Artist

```typescript
interface BlockedArtist {
  id: string;
  name: string;
  image?: string;
  blockedAt: string;
  source: 'manual' | 'imported';
}
```

## User Experience Flow

### Connection Setup Flow

1. **Landing on Connections Page**
   - See all available services as cards
   - Clear status for each service
   - Prominent "Connect" buttons for disconnected services

2. **Connecting a Service**
   - Click "Connect Spotify" button
   - See loading state with progress indicator
   - Redirect to OAuth flow
   - Return with success/error feedback

3. **Managing Connections**
   - See connected services with green status
   - Option to disconnect or refresh connection
   - Clear error messages with retry options

### Blocklist Management Flow

1. **Viewing Blocklist**
   - See search bar at top
   - Feed of blocked artists below
   - Empty state with guidance if no artists blocked

2. **Adding Artists**
   - Type in search bar
   - See instant search results
   - Click "Block" button next to artist
   - See artist immediately added to feed

3. **Managing Blocked Artists**
   - Scroll through feed of blocked artists
   - See artist images and names
   - Quick "Unblock" button on each card
   - Immediate removal from feed

## Visual Design

### Color Scheme
- **Primary**: Indigo (#4F46E5) for actions and highlights
- **Success**: Green (#10B981) for connected states
- **Warning**: Yellow (#F59E0B) for attention items
- **Error**: Red (#EF4444) for errors and destructive actions
- **Neutral**: Gray scale for text and backgrounds

### Typography
- **Headers**: Bold, clear hierarchy
- **Body**: Readable, accessible font sizes
- **Actions**: Clear, actionable button text

### Layout
- **Mobile-first**: Responsive design starting with mobile
- **Cards**: Clean card-based layout for connections and artists
- **Spacing**: Generous whitespace for clarity
- **Focus**: Clear focus states for accessibility

## Error Handling

### Connection Errors
- Clear error messages with specific reasons
- Retry buttons for temporary failures
- Help links for configuration issues
- Fallback to manual setup instructions

### Blocklist Errors
- Graceful handling of search failures
- Retry options for failed add/remove operations
- Offline indicators when backend unavailable
- Local caching for better performance

## Performance Considerations

### Loading States
- Skeleton loaders for connection cards
- Search result loading indicators
- Progressive loading for large blocklists
- Optimistic updates for immediate feedback

### Caching Strategy
- Cache connection status locally
- Store recent search results
- Offline support for viewing blocklist
- Background sync when connection restored

## Accessibility

### Keyboard Navigation
- Full keyboard support for all interactions
- Clear focus indicators
- Logical tab order

### Screen Readers
- Proper ARIA labels and descriptions
- Status announcements for dynamic content
- Clear heading hierarchy

### Visual Accessibility
- High contrast color combinations
- Scalable text and UI elements
- Clear visual hierarchy