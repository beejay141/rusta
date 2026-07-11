import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  integrations: [
    starlight({
      title: 'Ravix',
      logo: {
        dark: './public/logo-dark.svg',
        light: './public/logo-light.svg',
      },
      editLink: {
        baseUrl: 'https://github.com/ravix/ravix/edit/main/docs/',
      },
      head: [
        {
          tag: 'meta',
          attrs: { name: 'description', content: 'Modern Rust API framework with clean architecture and dependency injection' },
        },
      ],
      customCss: ['./src/styles/starlight.css'],
    }),
  ],
  site: 'https://ravix.dev',
  base: '/',
});