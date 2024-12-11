import { syncAll } from "./_sync";

export const prerender = false;

export async function GET() {
  const addedGames = await syncAll();
  return new Response(
    addedGames.join('\n')
  );
}
