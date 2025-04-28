import type { APIRoute } from "astro"
import { z } from "zod"
import { getList } from "./_list"
import { cacheDuration, cacheTags } from "../../cache-tags"

export const GET: APIRoute = async ({ params }) => {
  const parsedParams = z.object({
    page: z.number().default(1),
    title: z.string().nullish(),
    pinkPaw: z.coerce.boolean().nullish(),
    selectedGenre: z.string().nullish(),
  }).parse(params)

  const releases = await getList(parsedParams)

  return new Response(JSON.stringify(releases), {
    headers: {
      "CDN-Cache-Control": `max-age=${cacheDuration.halfDay}, s-maxage=${cacheDuration.oneYear}`,
      "Cache-Control": `max-age=${cacheDuration.halfDay}, s-maxage=${cacheDuration.oneYear}`,
      "Cache-Tag": cacheTags.index
    }
  })
}
