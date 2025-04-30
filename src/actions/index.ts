import { defineAction } from 'astro:actions';
import { z } from 'astro:schema';

export const server = {
  toggleBookmark: defineAction({
    input: z.string(),
    handler: async (slug, context) => {
      const cookieOption = { path: '/' }

      console.debug('context.cookies?.has(bookmarks)', context.cookies?.has('bookmarks'))
      const existingBookmarks: string[] = await context.cookies?.get('bookmarks')?.json() ?? []
      console.debug('existingBookmarks', existingBookmarks)

      if (existingBookmarks.includes(slug)) {
        console.log('bookmark exists')
        context.cookies?.set('bookmarks', JSON.stringify(existingBookmarks.filter(s => s != slug)), cookieOption)
        return false
      } else {
        console.log('bookmark does not exists')
        context.cookies?.set('bookmarks', JSON.stringify([...existingBookmarks, slug]), cookieOption)
        return true
      }
    }
  })
}
