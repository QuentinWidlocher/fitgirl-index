import { purgeCache } from "@netlify/functions";
import { syncAll } from "./_sync";
import { cacheTags } from "../../cache-tags";

export const prerender = false;

export async function GET() {
  const addedGames = await syncAll();

  if (addedGames.length > 0) {
    await purgeCache({
      tags: [cacheTags.index, ...addedGames.map((g) => g.slug.replace(/[^a-z0-9]/gi, "_"))],
    });
  }

  return new Response(addedGames.map((g) => g.title).join("\n"), {
    headers: {
      "X-Total-Count": String(addedGames.length),
    },
  });
}
