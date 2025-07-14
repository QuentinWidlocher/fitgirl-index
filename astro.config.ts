import { defineConfig, fontProviders } from "astro/config";

import netlify from "@astrojs/netlify";
import tailwind from "@astrojs/tailwind";
import db from "@astrojs/db";

// https://astro.build/config
export default defineConfig({
  adapter: netlify({
    cacheOnDemandPages: true,
  }),
  integrations: [tailwind(), db()],
  prefetch: false,
  output: "server",
  experimental: {
    clientPrerender: true,
    fonts: [
      {
        provider: fontProviders.fontsource(),
        name: "DM Sans",
        cssVariable: "--font-default",
      },
    ],
  },
});
