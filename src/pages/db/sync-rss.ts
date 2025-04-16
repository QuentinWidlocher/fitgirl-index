import { purgeCache } from "@netlify/functions";
import { syncRss } from "./_sync";
import { cacheTags } from "../../cache-tags";

export const prerender = false;

export async function GET() {
  const [addedGames, errors] = await syncRss();

  if (errors.length <= 0) {
    purgeCache({
      tags: [cacheTags.index],
    });
  }

  return new Response(
    addedGames.join("\n") + errors.map((e) => e.toString()).join("\n"),
    {
      headers: {
        "X-Total-Count": String(addedGames.length),
      },
      status: addedGames.length == 0 && errors.length > 0 ? 500 : 200,
    }
  );
}
