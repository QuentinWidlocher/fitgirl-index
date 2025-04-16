import { purgeCache } from "@netlify/functions";
import { syncAll } from "./_sync";
import { cacheTags } from "../../cache-tags";

export const prerender = false;

export async function GET() {
  const addedGames = await syncAll();

  purgeCache({
    tags: [cacheTags.index],
  });

  return new Response(addedGames.join("\n"), {
    headers: {
      "X-Total-Count": String(addedGames.length),
    },
  });
}
