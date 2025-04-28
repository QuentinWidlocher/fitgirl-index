import { purgeCache } from "@netlify/functions";
import { syncLatest, syncRss } from "./_sync";
import { cacheTags } from "../../cache-tags";

export const prerender = false;

export async function GET() {
  const [addedGames, errors] = await syncLatest();

  await Promise.all(
    [cacheTags.index, ...addedGames.map((g) => g.slug.replace(/[^a-z0-9]/gi, "_"))].map(tag => {
      console.log("purge cache", tag)
      return purgeCache({ tags: [tag] });
    })
  )

  return new Response(
    addedGames.map((g) => g.title).join("\n") +
    errors.map((e) => e.toString()).join("\n"),
    {
      headers: {
        "X-Total-Count": String(addedGames.length),
      },
      status: addedGames.length == 0 && errors.length > 0 ? 500 : 200,
    }
  );
}
