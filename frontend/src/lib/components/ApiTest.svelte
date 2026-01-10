<script lang="ts">
  import { apiClient } from '../utils/api-client';
  import { authActions, isAuthenticated } from '../stores/auth';
  import { dnpActions } from '../stores/dnp';
  import { connectionActions } from '../stores/connections';
  
  let testResults: any[] = [];
  let isRunning = false;
  
  async function runTests() {
    isRunning = true;
    testResults = [];
    
    // Test 1: Health check (no auth required)
    await testEndpoint('Health Check', 'GET', '/health', false);
    
    // Test 2: OAuth health (no auth required)
    await testEndpoint('OAuth Health', 'GET', '/oauth/health', false);
    
    if ($isAuthenticated) {
      // Test 3: User profile
      await testEndpoint('User Profile', 'GET', '/api/v1/users/profile', true);
      
      // Test 4: DNP list
      await testEndpoint('DNP List', 'GET', '/api/v1/dnp/list', true);
      
      // Test 5: OAuth accounts
      await testEndpoint('OAuth Accounts', 'GET', '/api/v1/auth/oauth/accounts', true);
      
      // Test 6: Artist search
      await testEndpoint('Artist Search', 'GET', '/api/v1/dnp/search?q=test&limit=5', true);
    } else {
      testResults.push({
        name: 'Authentication Required',
        status: 'skipped',
        message: 'Please log in to test authenticated endpoints'
      });
    }
    
    isRunning = false;
  }
  
  async function testEndpoint(name: string, method: string, endpoint: string, requireAuth: boolean) {
    try {
      let response;
      
      if (requireAuth) {
        response = await apiClient.authenticatedRequest(method as any, endpoint);
      } else {
        response = await apiClient.get(endpoint, false);
      }
      
      testResults.push({
        name,
        status: response.success ? 'success' : 'error',
        message: response.success ? 'OK' : response.message,
        data: response.data ? JSON.stringify(response.data, null, 2) : null
      });
    } catch (error: any) {
      testResults.push({
        name,
        status: 'error',
        message: error.message || 'Network error',
        data: null
      });
    }
    
    // Trigger reactivity
    testResults = testResults;
  }
  
  function getStatusColor(status: string) {
    switch (status) {
      case 'success': return 'text-green-400 bg-green-900/50';
      case 'error': return 'text-red-400 bg-red-900/50';
      case 'skipped': return 'text-yellow-400 bg-yellow-900/50';
      default: return 'text-zinc-400 bg-zinc-700';
    }
  }
</script>

<div class="shadow rounded-lg p-6 mb-6" style="background: #27272a; border: 1px solid #52525b;">
  <div class="flex justify-between items-center mb-4">
    <h3 class="text-lg font-medium text-white">API Endpoint Tests</h3>
    <button
      on:click={runTests}
      disabled={isRunning}
      class="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {#if isRunning}
        <svg class="animate-spin -ml-1 mr-2 h-4 w-4 text-white" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
        Testing...
      {:else}
        Run Tests
      {/if}
    </button>
  </div>
  
  <div class="text-sm text-zinc-400 mb-4">
    Authentication Status:
    <span class="font-medium {$isAuthenticated ? 'text-green-400' : 'text-red-400'}">
      {$isAuthenticated ? 'Authenticated' : 'Not Authenticated'}
    </span>
  </div>
  
  {#if testResults.length > 0}
    <div class="space-y-3">
      {#each testResults as result}
        <div class="rounded-lg p-4" style="background: #3f3f46; border: 1px solid #52525b;">
          <div class="flex items-center justify-between mb-2">
            <h4 class="font-medium text-white">{result.name}</h4>
            <span class="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {getStatusColor(result.status)}">
              {result.status}
            </span>
          </div>
          
          <p class="text-sm text-zinc-400 mb-2">{result.message}</p>

          {#if result.data}
            <details class="mt-2">
              <summary class="cursor-pointer text-sm text-indigo-400 hover:text-indigo-300">
                Show Response Data
              </summary>
              <pre class="mt-2 text-xs p-2 rounded overflow-x-auto text-zinc-300" style="background: #27272a;">{result.data}</pre>
            </details>
          {/if}
        </div>
      {/each}
    </div>
  {:else}
    <p class="text-zinc-400 text-center py-8">Click "Run Tests" to check API connectivity</p>
  {/if}
</div>