<!--
  USWDS + Skeleton UI Showcase Component

  This component demonstrates the new design system combining
  U.S. Web Design System standards with Skeleton UI components.
-->

<script lang="ts">
  let showAlert = true;
  let formData = {
    name: '',
    email: '',
    message: ''
  };

  let errors: Record<string, string> = {};

  function validateForm() {
    errors = {};

    if (!formData.name.trim()) {
      errors.name = 'Name is required';
    }

    if (!formData.email.trim()) {
      errors.email = 'Email is required';
    } else if (!/\S+@\S+\.\S+/.test(formData.email)) {
      errors.email = 'Please enter a valid email address';
    }

    if (!formData.message.trim()) {
      errors.message = 'Message is required';
    }

    return Object.keys(errors).length === 0;
  }

  function handleSubmit() {
    if (validateForm()) {
      console.log('Form submitted:', formData);
      // Reset form
      formData = { name: '', email: '', message: '' };
      showAlert = true;
    }
  }
</script>

<div class="container mx-auto px-4 py-8 max-w-6xl">
  <!-- Header -->
  <div class="mb-8">
    <h1 class="text-4xl font-bold text-zinc-300 mb-4">
      Design System Showcase
    </h1>
    <p class="text-lg text-zinc-400">
      A comprehensive design system combining modern UI standards
      with accessible, compliant interfaces.
    </p>
  </div>

  <!-- Alert Examples -->
  {#if showAlert}
    <div class="alert alert-success mb-6">
      <div class="">
        <svg class="icon icon--md icon--success" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
        </svg>
      </div>
      <div class="flex-1">
        <h3 class="font-semibold mb-1">Success!</h3>
        <p class="text-sm">Your form has been submitted successfully. This alert demonstrates styling.</p>
      </div>
      <button
        type="button"
        on:click={() => showAlert = false}
        class="flex-shrink-0 p-1 hover:bg-black hover:bg-opacity-10 rounded"
        aria-label="Close alert"
      >
        <svg class="icon icon--sm" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  {/if}

  <div class="grid grid-cols-1 lg:grid-cols-2 gap-8">
    <!-- Left Column: Components -->
    <div class="space-y-8">
      <!-- Button Examples -->
      <section class="card">
        <div class="card-header">
          <h2 class="text-2xl font-semibold text-zinc-300">
            Buttons
          </h2>
        </div>
        <div class="card-body space-y-4">
          <div class="flex flex-wrap gap-4">
            <button class="btn btn-primary">
              <svg class="icon icon--sm mr-2" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              Primary Action
            </button>

            <button class="btn btn-secondary">
              <svg class="icon icon--sm mr-2" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15.232 5.232l3.536 3.536m-2.036-5.036a2.5 2.5 0 113.536 3.536L6.5 21.036H3v-3.572L16.732 3.732z" />
              </svg>
              Secondary Action
            </button>
          </div>

          <p class="text-sm text-zinc-400">
            Buttons follow accessibility standards with 44px minimum touch targets and proper focus indicators.
          </p>
        </div>
      </section>

      <!-- Icon System -->
      <section class="card">
        <div class="card-header">
          <h2 class="text-2xl font-semibold text-zinc-300">
            Icon System
          </h2>
        </div>
        <div class="card-body">
          <div class="grid grid-cols-5 gap-4 mb-6">
            <div class="text-center">
              <div class="flex justify-center mb-2">
                <svg class="icon icon--xs icon--primary" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <code class="text-xs bg-zinc-700 px-2 py-1 rounded">xs (12px)</code>
            </div>

            <div class="text-center">
              <div class="flex justify-center mb-2">
                <svg class="icon icon--sm icon--primary" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <code class="text-xs bg-zinc-700 px-2 py-1 rounded">sm (16px)</code>
            </div>

            <div class="text-center">
              <div class="flex justify-center mb-2">
                <svg class="icon icon--md icon--primary" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <code class="text-xs bg-zinc-700 px-2 py-1 rounded">md (20px)</code>
            </div>

            <div class="text-center">
              <div class="flex justify-center mb-2">
                <svg class="icon icon--lg icon--primary" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <code class="text-xs bg-zinc-700 px-2 py-1 rounded">lg (24px)</code>
            </div>

            <div class="text-center">
              <div class="flex justify-center mb-2">
                <svg class="icon icon--xl icon--primary" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <code class="text-xs bg-zinc-700 px-2 py-1 rounded">xl (32px)</code>
            </div>
          </div>

          <!-- Semantic Colors -->
          <div class="grid grid-cols-5 gap-4">
            <div class="text-center">
              <div class="flex justify-center mb-2">
                <svg class="icon icon--lg icon--success" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                </svg>
              </div>
              <code class="text-xs">Success</code>
            </div>

            <div class="text-center">
              <div class="flex justify-center mb-2">
                <svg class="icon icon--lg icon--error" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
              </div>
              <code class="text-xs">Error</code>
            </div>

            <div class="text-center">
              <div class="flex justify-center mb-2">
                <svg class="icon icon--lg icon--warning" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
                </svg>
              </div>
              <code class="text-xs">Warning</code>
            </div>

            <div class="text-center">
              <div class="flex justify-center mb-2">
                <svg class="icon icon--lg icon--primary" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <code class="text-xs">Primary</code>
            </div>

            <div class="text-center">
              <div class="flex justify-center mb-2">
                <svg class="icon icon--lg icon--neutral" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093m0 3h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <code class="text-xs">Neutral</code>
            </div>
          </div>
        </div>
      </section>

      <!-- Alert Examples -->
      <section class="card">
        <div class="card-header">
          <h2 class="text-2xl font-semibold text-zinc-300">
            Alerts
          </h2>
        </div>
        <div class="card-body space-y-4">
          <div class="alert alert-info">
            <div class="flex-shrink-0">
              <svg class="icon icon--md icon--primary" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div>
              <h4 class="font-semibold mb-1">Information</h4>
              <p class="text-sm">This is an informational alert following accessibility guidelines.</p>
            </div>
          </div>

          <div class="alert alert-warning">
            <div class="flex-shrink-0">
              <svg class="icon icon--md icon--warning" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
              </svg>
            </div>
            <div>
              <h4 class="font-semibold mb-1">Warning</h4>
              <p class="text-sm">This is a warning alert with proper color contrast.</p>
            </div>
          </div>

          <div class="alert alert-error">
            <div class="flex-shrink-0">
              <svg class="icon icon--md icon--error" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
            </div>
            <div>
              <h4 class="font-semibold mb-1">Error</h4>
              <p class="text-sm">This is an error alert meeting accessibility standards.</p>
            </div>
          </div>
        </div>
      </section>
    </div>

    <!-- Right Column: Form Example -->
    <div>
      <section class="card">
        <div class="card-header">
          <h2 class="text-2xl font-semibold text-zinc-300">
            Form Components
          </h2>
        </div>
        <div class="card-body">
          <form on:submit|preventDefault={handleSubmit} class="space-y-6">
            <div class="form-field">
              <label for="name" class="form-label">
                Full Name
                <span class="text-red-500 ml-1" aria-label="required">*</span>
              </label>
              <input
                type="text"
                id="name"
                bind:value={formData.name}
                class="form-input {errors.name ? 'form-input--error' : ''}"
                placeholder="Enter your full name"
                aria-describedby={errors.name ? 'name-error' : undefined}
                aria-invalid={errors.name ? 'true' : 'false'}
              />
              {#if errors.name}
                <div id="name-error" class="mt-2 text-sm text-red-500 flex items-center">
                  <svg class="icon icon--sm icon--error mr-1" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {errors.name}
                </div>
              {/if}
            </div>

            <div class="form-field">
              <label for="email" class="form-label">
                Email Address
                <span class="text-red-500 ml-1" aria-label="required">*</span>
              </label>
              <input
                type="email"
                id="email"
                bind:value={formData.email}
                class="form-input {errors.email ? 'form-input--error' : ''}"
                placeholder="Enter your email address"
                aria-describedby={errors.email ? 'email-error' : undefined}
                aria-invalid={errors.email ? 'true' : 'false'}
              />
              {#if errors.email}
                <div id="email-error" class="mt-2 text-sm text-red-500 flex items-center">
                  <svg class="icon icon--sm icon--error mr-1" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {errors.email}
                </div>
              {/if}
            </div>

            <div class="form-field">
              <label for="message" class="form-label">
                Message
                <span class="text-red-500 ml-1" aria-label="required">*</span>
              </label>
              <textarea
                id="message"
                bind:value={formData.message}
                rows="4"
                class="form-input {errors.message ? 'form-input--error' : ''}"
                placeholder="Enter your message"
                aria-describedby={errors.message ? 'message-error' : undefined}
                aria-invalid={errors.message ? 'true' : 'false'}
              ></textarea>
              {#if errors.message}
                <div id="message-error" class="mt-2 text-sm text-red-500 flex items-center">
                  <svg class="icon icon--sm icon--error mr-1" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                  {errors.message}
                </div>
              {/if}
            </div>

            <div class="flex justify-end space-x-4">
              <button type="button" class="btn btn-secondary">
                Cancel
              </button>
              <button type="submit" class="btn btn-primary">
                <svg class="icon icon--sm mr-2" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
                </svg>
                Submit Form
              </button>
            </div>
          </form>
        </div>
        <div class="card-footer">
          <p class="text-sm text-zinc-400">
            All form fields are required and follow accessibility guidelines
            including proper labeling, error handling, and keyboard navigation.
          </p>
        </div>
      </section>
    </div>
  </div>

  <!-- Footer -->
  <div class="mt-12 pt-8 border-t border-zinc-600">
    <div class="text-center">
      <h3 class="text-xl font-semibold text-zinc-300 mb-4">
        Key Features
      </h3>
      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <div class="text-center">
          <div class="flex justify-center mb-3">
            <svg class="icon icon--xl icon--primary" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
            </svg>
          </div>
          <h4 class="font-semibold mb-2">Accessible</h4>
          <p class="text-sm text-zinc-400">
            Follows accessibility standards for usability and compliance.
          </p>
        </div>

        <div class="text-center">
          <div class="flex justify-center mb-3">
            <svg class="icon icon--xl icon--success" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z" />
            </svg>
          </div>
          <h4 class="font-semibold mb-2">High Performance</h4>
          <p class="text-sm text-zinc-400">
            Optimized CSS with minimal bundle size and smooth 60fps animations.
          </p>
        </div>

        <div class="text-center">
          <div class="flex justify-center mb-3">
            <svg class="icon icon--xl icon--warning" aria-hidden="true" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z" />
            </svg>
          </div>
          <h4 class="font-semibold mb-2">Mobile First</h4>
          <p class="text-sm text-zinc-400">
            Responsive design with mobile-first approach and touch-friendly interfaces.
          </p>
        </div>
      </div>
    </div>
  </div>
</div>

<style>
  /* Component-specific styles if needed */
  textarea.form-input {
    resize: vertical;
    min-height: 6rem;
  }
</style>
