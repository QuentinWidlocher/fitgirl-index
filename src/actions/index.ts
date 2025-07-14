import { defineAction } from "astro:actions";
import { z } from "astro:schema";

export const server = {
  toggleBookmark: defineAction({
    input: z.string(),
    handler: async (slug, context) => {
      const cookieOption = { path: "/" };

      const existingBookmarks: string[] =
        (await context.cookies?.get("bookmarks")?.json()) ?? [];

      if (existingBookmarks.includes(slug)) {
        context.cookies?.set(
          "bookmarks",
          JSON.stringify(existingBookmarks.filter((s) => s != slug)),
          cookieOption
        );
        return false;
      } else {
        context.cookies?.set(
          "bookmarks",
          JSON.stringify([...existingBookmarks, slug]),
          cookieOption
        );
        return true;
      }
    },
  }),
};
