import { like, Release, ReleaseGenres, db, eq, and, desc } from "astro:db";

const PAGE_SIZE = 100

export async function getList({
  title,
  pinkPaw,
  selectedGenre,
  page = 1,
}: {
  title?: string | null,
  pinkPaw?: boolean | null,
  selectedGenre?: string | null,
  page?: number,
}) {
  const conditions = [];

  if (title) {
    conditions.push(like(Release.title, `%${title}%`));
  }

  if (pinkPaw) {
    conditions.push(eq(Release.pinkPaw, true));
  }

  let query;
  if (selectedGenre) {
    conditions.push(eq(ReleaseGenres.genre, selectedGenre));
    query = db
      .select()
      .from(Release)
      .innerJoin(ReleaseGenres, eq(Release.id, ReleaseGenres.releaseId))
      .groupBy(Release.id)
      .limit(PAGE_SIZE)
      .offset(PAGE_SIZE * page)
      .where(and(...conditions))
      .orderBy(desc(Release.published));
  } else {
    query = db
      .select()
      .from(Release)
      .limit(PAGE_SIZE)
      .offset(PAGE_SIZE * page)
      .where(and(...conditions))
      .orderBy(desc(Release.published));
  }

  const releases = await query.then((releasesWithGenres) =>
    releasesWithGenres.map((r) => ("Release" in r ? r.Release : r)),
  );

  return releases
}
