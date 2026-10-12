import tailwindcss from '@tailwindcss/vite'
import { devtools } from '@tanstack/devtools-vite'
import react from '@vitejs/plugin-react'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  plugins: [devtools(), react(), tailwindcss()],
  server: {
    port: 5173,
    strictPort: true,
    allowedHosts: ['lake.tail3275e2.ts.net'],
    watch: {
      ignored: ['**/coverage/**', '**/dist/**'],
    },
    proxy: {
      '/v1': 'http://127.0.0.1:3010',
      '/api': 'http://127.0.0.1:3010',
    },
  },
  test: {
    environment: 'jsdom',
    setupFiles: './src/test/setup.ts',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'json-summary'],
      reportsDirectory: './coverage',
      include: ['src/**/*.{ts,tsx}'],
      exclude: [
        'src/**/*.test.{ts,tsx}',
        'src/test/**',
        'src/main.tsx',
        'src/vite-env.d.ts',
      ],
      thresholds: {
        statements: 75,
        branches: 60,
        functions: 75,
        lines: 80,
      },
    },
  },
})
