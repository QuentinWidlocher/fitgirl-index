import { syncRss } from "./_sync";

export const prerender = false;

export async function GET() {
  const [addedGames, errors] = await syncRss();
  return new Response(
    addedGames.join('\n') + errors.map(e => e.toString()).join('\n'),
    {
      headers: {
        "X-Total-Count": String(addedGames.length)
      },
      status: addedGames.length == 0 && errors.length > 0 ? 500 : 200
    }
  );
}
