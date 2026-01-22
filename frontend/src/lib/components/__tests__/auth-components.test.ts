// Basic test to verify components can be imported and instantiated
// TODO: LoginForm.svelte doesn't exist - the login functionality is in Login.svelte
// import LoginForm from '../LoginForm.svelte';
import RegisterForm from '../RegisterForm.svelte';
import TwoFactorSetup from '../TwoFactorSetup.svelte';
import TwoFactorVerification from '../TwoFactorVerification.svelte';
import { describe, test, expect } from 'vitest';

// Test that components can be imported without errors
describe('Authentication Components', () => {
  test.skip('LoginForm can be imported (component does not exist)', () => {
    // LoginForm doesn't exist - login is handled in Login.svelte
  });

  test('RegisterForm can be imported', () => {
    expect(RegisterForm).toBeDefined();
  });

  test('TwoFactorSetup can be imported', () => {
    expect(TwoFactorSetup).toBeDefined();
  });

  test('TwoFactorVerification can be imported', () => {
    expect(TwoFactorVerification).toBeDefined();
  });
});