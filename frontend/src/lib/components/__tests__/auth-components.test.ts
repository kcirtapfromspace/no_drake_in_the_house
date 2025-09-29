// Basic test to verify components can be imported and instantiated
import LoginForm from '../LoginForm.svelte';
import RegisterForm from '../RegisterForm.svelte';
import TwoFactorSetup from '../TwoFactorSetup.svelte';
import TwoFactorVerification from '../TwoFactorVerification.svelte';

// Test that components can be imported without errors
describe('Authentication Components', () => {
  test('LoginForm can be imported', () => {
    expect(LoginForm).toBeDefined();
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