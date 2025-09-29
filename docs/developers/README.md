# Developer Documentation

Welcome to the Music Streaming Blocklist Manager developer documentation. This section provides comprehensive guides for integrating with our platform, extending functionality, and contributing to the project.

## 🚀 Quick Start

- **[API Integration](./api-integration.md)** - Integrate with our REST API
- **[Browser Extension Development](./browser-extension.md)** - Extend or customize the browser extension
- **[Mobile Integration](./mobile-integration.md)** - Build mobile apps and shortcuts
- **[Platform Adapters](./platform-adapters.md)** - Add support for new streaming services

## 📚 Core Documentation

### API Development
- **[Authentication](./authentication.md)** - OAuth flows and JWT handling
- **[Rate Limiting](./rate-limiting.md)** - Handle API rate limits effectively
- **[Webhooks](./webhooks.md)** - Real-time event notifications
- **[Error Handling](./error-handling.md)** - Robust error handling patterns

### Extension Development
- **[Architecture Overview](./extension-architecture.md)** - Extension structure and patterns
- **[Content Scripts](./content-scripts.md)** - DOM manipulation and event handling
- **[Background Scripts](./background-scripts.md)** - Service workers and cross-tab communication
- **[Manifest V3 Guide](./manifest-v3.md)** - Modern extension development

### Mobile Development
- **[iOS Shortcuts](./ios-shortcuts.md)** - Create iOS Shortcuts and Siri integration
- **[Android Intents](./android-intents.md)** - Build Android automation and Tasker integration
- **[Mobile APIs](./mobile-apis.md)** - Mobile-optimized API endpoints

### Platform Integration
- **[Streaming Service APIs](./streaming-apis.md)** - Working with music platform APIs
- **[Entity Resolution](./entity-resolution.md)** - Artist matching and disambiguation
- **[Capability Matrices](./capability-matrices.md)** - Define platform capabilities

## 🛠️ Development Tools

### SDKs and Libraries
- **[JavaScript/TypeScript SDK](./sdk-javascript.md)** - Official JavaScript SDK
- **[Python SDK](./sdk-python.md)** - Python integration library
- **[Go SDK](./sdk-go.md)** - Go client library
- **[Rust SDK](./sdk-rust.md)** - Rust integration crate

### Testing and Debugging
- **[Testing Guide](./testing.md)** - Unit, integration, and end-to-end testing
- **[Debugging Tools](./debugging.md)** - Debug APIs, extensions, and mobile integrations
- **[Mock Services](./mock-services.md)** - Development and testing environments

### Deployment
- **[Self-Hosting](./self-hosting.md)** - Deploy your own instance
- **[Docker Setup](./docker.md)** - Containerized deployment
- **[Kubernetes](./kubernetes.md)** - Production Kubernetes deployment

## 🔧 Extension Development

### Getting Started
```bash
# Clone the extension development template
git clone https://github.com/nodrakeinthe/extension-template
cd extension-template

# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build
```

### Key Concepts

**Content Script Architecture**
```javascript
// content-script.js
class StreamingServiceFilter {
  constructor(platform) {
    this.platform = platform;
    this.dnpList = new Set();
    this.observer = null;
  }

  async initialize() {
    await this.loadDNPList();
    this.setupMutationObserver();
    this.scanExistingContent();
  }

  setupMutationObserver() {
    this.observer = new MutationObserver((mutations) => {
      mutations.forEach(mutation => {
        if (mutation.type === 'childList') {
          this.scanNewContent(mutation.addedNodes);
        }
      });
    });

    this.observer.observe(document.body, {
      childList: true,
      subtree: true
    });
  }
}
```

## 📱 Mobile Development

### iOS Shortcuts Integration

**Basic Shortcut Structure**
```json
{
  "WFWorkflowActions": [
    {
      "WFWorkflowActionIdentifier": "is.workflow.actions.url",
      "WFWorkflowActionParameters": {
        "WFURLActionURL": "https://api.nodrakeinthe.house/v1/dnp/lists"
      }
    },
    {
      "WFWorkflowActionIdentifier": "is.workflow.actions.downloadurl",
      "WFWorkflowActionParameters": {
        "WFHTTPMethod": "POST",
        "WFHTTPHeaders": {
          "Authorization": "Bearer {{API_TOKEN}}",
          "Content-Type": "application/json"
        }
      }
    }
  ]
}
```

### Android Integration

**Tasker Integration**
```java
// TaskerReceiver.java
public class TaskerReceiver extends BroadcastReceiver {
    @Override
    public void onReceive(Context context, Intent intent) {
        if (TaskerPlugin.Event.hostSupportsOnFireVariableReplacement(intent)) {
            TaskerPlugin.Event.addPassThroughData(intent, INTENT_REQUEST_REQUERY);
        }
        
        String artistName = intent.getStringExtra("artist_name");
        String action = intent.getStringExtra("action"); // "add" or "remove"
        
        DNPApiClient.getInstance().modifyDNPList(artistName, action);
    }
}
```

## 🔌 API Integration Examples

### Authentication Flow
```javascript
// OAuth 2.0 with PKCE
class KiroAuth {
  async initiateAuth() {
    const codeVerifier = this.generateCodeVerifier();
    const codeChallenge = await this.generateCodeChallenge(codeVerifier);
    
    const authUrl = `https://api.nodrakeinthe.house/v1/auth/oauth/initiate`;
    const response = await fetch(authUrl, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        provider: 'kiro',
        redirect_uri: 'https://yourapp.com/callback',
        code_challenge: codeChallenge,
        code_challenge_method: 'S256'
      })
    });
    
    const { authorization_url } = await response.json();
    window.location.href = authorization_url;
  }
}
```

### DNP List Management
```python
# Python SDK example
from nodrakeinthe import KiroClient

client = KiroClient(api_key='your_api_key')

# Add artist to DNP list
result = client.dnp_lists.add_artist(
    artist_query='Drake',
    tags=['personal'],
    note='Personal preference'
)

# Plan enforcement
plan = client.enforcement.plan(
    providers=['spotify', 'apple'],
    options={
        'aggressiveness': 'moderate',
        'block_collaborations': True
    },
    dry_run=True
)

# Execute enforcement
execution = client.enforcement.execute(plan.plan_id)
```

## 🏗️ Architecture Patterns

### Platform Adapter Pattern
```rust
// Rust platform adapter example
#[async_trait]
pub trait StreamingPlatformAdapter {
    async fn authenticate(&self, credentials: OAuthCredentials) -> Result<Connection>;
    async fn scan_library(&self, connection: &Connection) -> Result<LibrarySnapshot>;
    async fn plan_enforcement(&self, dnp_list: &[Artist], library: &LibrarySnapshot) -> Result<EnforcementPlan>;
    async fn execute_plan(&self, plan: &EnforcementPlan) -> Result<EnforcementResult>;
    fn get_capabilities(&self) -> PlatformCapabilities;
}

pub struct SpotifyAdapter {
    client: SpotifyClient,
    rate_limiter: RateLimiter,
}

#[async_trait]
impl StreamingPlatformAdapter for SpotifyAdapter {
    async fn authenticate(&self, credentials: OAuthCredentials) -> Result<Connection> {
        // Spotify OAuth implementation
    }
    
    // ... other methods
}
```

## 📊 Monitoring and Analytics

### Custom Metrics
```javascript
// Track custom events
class KiroAnalytics {
  trackEnforcementExecution(executionId, platform, actionCount) {
    this.sendEvent('enforcement_executed', {
      execution_id: executionId,
      platform: platform,
      action_count: actionCount,
      timestamp: new Date().toISOString()
    });
  }

  trackExtensionUsage(action, platform) {
    this.sendEvent('extension_action', {
      action: action, // 'hide', 'skip', 'override'
      platform: platform,
      user_agent: navigator.userAgent
    });
  }
}
```

## 🔒 Security Best Practices

### Token Management
```javascript
// Secure token storage
class SecureTokenManager {
  async storeTokens(tokens) {
    // Use browser's secure storage
    await chrome.storage.local.set({
      access_token: await this.encrypt(tokens.access_token),
      refresh_token: await this.encrypt(tokens.refresh_token),
      expires_at: tokens.expires_at
    });
  }

  async getAccessToken() {
    const stored = await chrome.storage.local.get(['access_token', 'expires_at']);
    
    if (this.isTokenExpired(stored.expires_at)) {
      return await this.refreshToken();
    }
    
    return await this.decrypt(stored.access_token);
  }
}
```

## 🧪 Testing Strategies

### Extension Testing
```javascript
// Jest test for content script
describe('SpotifyContentFilter', () => {
  let filter;
  
  beforeEach(() => {
    document.body.innerHTML = `
      <div data-testid="artist-tile" data-artist-id="drake">
        <span>Drake</span>
      </div>
    `;
    
    filter = new SpotifyContentFilter();
    filter.dnpList = new Set(['drake']);
  });

  test('should hide blocked artist tiles', () => {
    filter.scanExistingContent();
    
    const artistTile = document.querySelector('[data-testid="artist-tile"]');
    expect(artistTile.style.display).toBe('none');
  });
});
```

### API Integration Testing
```python
# Python integration test
import pytest
from nodrakeinthe import KiroClient

@pytest.fixture
def client():
    return KiroClient(api_key='test_key', base_url='http://localhost:3000')

def test_add_artist_to_dnp_list(client):
    result = client.dnp_lists.add_artist(
        artist_query='Test Artist',
        tags=['test']
    )
    
    assert result.artist.canonical_name == 'Test Artist'
    assert 'test' in result.tags
```

## 🤝 Contributing

### Development Setup
```bash
# Clone the repository
git clone https://github.com/nodrakeinthe/music-blocklist-manager
cd music-blocklist-manager

# Set up development environment
make setup

# Start development servers
make dev
```

### Contribution Guidelines
1. **Fork the repository** and create a feature branch
2. **Write tests** for new functionality
3. **Follow code style** guidelines (run `make lint`)
4. **Update documentation** for API changes
5. **Submit a pull request** with detailed description

### Code Review Process
- All changes require review from maintainers
- Automated tests must pass
- Documentation must be updated
- Security review for authentication/token handling changes

---

## 📞 Developer Support

### Getting Help
- **API Issues**: Check [API Status Page](https://status.nodrakeinthe.house)
- **SDK Problems**: Open issues in respective SDK repositories
- **Extension Development**: Join our [Discord](https://discord.gg/nodrakeinthe)
- **General Questions**: Email [developers@nodrakeinthe.house](mailto:developers@nodrakeinthe.house)

### Resources
- **API Reference**: [api.nodrakeinthe.house](https://api.nodrakeinthe.house)
- **SDK Documentation**: [docs.nodrakeinthe.house/sdks](https://docs.nodrakeinthe.house/sdks)
- **Example Projects**: [github.com/nodrakeinthe/examples](https://github.com/nodrakeinthe/examples)
- **Developer Blog**: [blog.nodrakeinthe.house/developers](https://blog.nodrakeinthe.house/developers)