import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    // Private verification scripts under .local are not repository test fixtures.
    include: ['tests/**/*.test.ts'],
  },
});
