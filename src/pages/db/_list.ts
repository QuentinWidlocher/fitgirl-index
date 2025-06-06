import {
  like,
  Release,
  ReleaseGenres,
  db,
  eq,
  and,
  desc,
  inArray,
  ReleaseCompanies,
} from "astro:db";

const PAGE_SIZE = 100;

export async function getList({
  page = 1,
  pinkPaw,
  company,
  genre,
  slugs,
  title,
}: {
  page?: number;
  pinkPaw?: boolean | null;
  company?: string | null;
  genre?: string | null;
  slugs?: string[];
  title?: string | null;
}) {
  const conditions = [];

  if (title) {
    conditions.push(like(Release.title, `%${title}%`));
  }

  if (pinkPaw) {
    conditions.push(eq(Release.pinkPaw, true));
  }

  if (slugs && slugs.length > 0) {
    conditions.push(inArray(Release.slug, slugs));
  }

  let query = db.select().from(Release).$dynamic();

  if (genre) {
    conditions.push(like(ReleaseGenres.genre, `%${genre}%`));

    // @ts-ignore
    query = query
      .innerJoin(ReleaseGenres, eq(Release.id, ReleaseGenres.releaseId))
      .groupBy(Release.id);
  }

  if (company) {
    conditions.push(like(ReleaseCompanies.company, `%${company}%`));

    // @ts-ignore
    query = query
      .innerJoin(ReleaseCompanies, eq(Release.id, ReleaseCompanies.releaseId))
      .groupBy(Release.id);
  }

  query = query
    .limit(PAGE_SIZE)
    .offset(PAGE_SIZE * page)
    .where(and(...conditions))
    .orderBy(desc(Release.published));

  const releases = await query.then((releasesWithRelations) =>
    releasesWithRelations.map((r) =>
      "Release" in r ? (r.Release as typeof r) : r
    )
  );

  return releases;
}
