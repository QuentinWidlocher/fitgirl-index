import { createWriteStream, rmSync, unlinkSync } from "fs";
import { HTMLElement, parse as parseHTML } from "node-html-parser";
import { z } from "zod";
import { Company, db, desc, Genre, Language, Release, ReleaseGenres, ReleaseLanguages } from "astro:db";
import slug from "slug";
import { ReleaseCompanies } from "astro:db";
import Parser from 'rss-parser';

const base_url = "https://fitgirl-repacks.site/";

const parsedContentSchema = z.object({
  title: z.string(),
  link: z.string(),
  published: z.date(),
  coverSrc: z.string(),
  genres: z.array(z.string()),
  companies: z.array(z.string()),
  languages: z.array(z.string()),
  originalSize: z.string(),
  repackSize: z.string(),
  mirrors: z.array(
    z.object({
      name: z.string(),
      links: z.array(z.object({ name: z.string(), link: z.string() })),
    })
  ),
  screenshots: z.array(z.string()).optional(),
  repackDescription: z.string().optional(),
  gameDescription: z.string().optional(),
});

type ParsedContent = z.infer<typeof parsedContentSchema>;
type ParsedContentInput = Partial<z.infer<typeof parsedContentSchema>>;

function decode(str: string) {
  return str
    .trim()
    .replace(/&#(\d+);/g, function (match, dec) {
      return String.fromCharCode(dec);
    })
    .replaceAll("’", "'");
}

async function getGameList(page = 1) {
  const res = await fetch(`${base_url}/all-my-repacks-a-z/?lcp_page0=${page}`);
  const html = await res.text();

  const root = parseHTML(html);

  const list = root.querySelector(".lcp_catlist")

  if (!list) throw new Error("List is missing")

  return list.querySelectorAll("li")
    .map((li) => {
      const a = li.querySelector("a");
      if (!a) return;
      const link = a.attrs.href;
      const title = decode(a.rawText);
      return { title, link };
    }).filter(Boolean);
}

async function getGame(url: string) {
  const res = await fetch(url, {
    headers: {
      Accept: "text/html",
      "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36",
      Referer: "https://fitgirl-repacks.site/all-my-repacks-a-z/"
    }
  });
  const html = await res.text();

  let parsedContent: ParsedContentInput = {};

  const root = parseHTML(html);
  let content = root.querySelector(".entry-content");

  if (!content) {
    const header = root.querySelector("header.entry-header");
    if (!header) throw new Error("Header is missing");
    const elements = [];
    let currentEl = header.nextElementSibling;
    while (currentEl) {

      elements.push(currentEl);

      currentEl = currentEl.nextElementSibling;
      if (!currentEl || currentEl.tagName === "STYLE") break;
    }

    content = parseHTML("<div>" + elements.map(el => el.outerHTML).join("") + "</div>");
  }

  if (!content) {
    throw new Error("unable to parse html correctly");
  }

  const date = root.querySelector('meta[property="article:published_time"]')?.attrs.content;
  if (!date) throw new Error("Published time is missing");
  parsedContent.published = new Date(date);

  const title = root.querySelector(".entry-title")?.rawText
  if (!title) throw new Error("Title is missing");

  parsedContent.title = decode(title);
  parsedContent.link = url;
  parsedContent.coverSrc = content.querySelector("h3 + p > a > img")?.attrs.src;

  const sections = content.querySelectorAll("h3 + *");

  for (const section of sections) {
    if (section.previousElementSibling?.rawText.includes("Screenshots")) {
      parsedContent.screenshots = content
        .querySelectorAll("a > img")
        .map((img) => img.attrs.src);
    } else if (section.previousElementSibling?.rawText.includes("Repack")) {
      parsedContent.repackDescription = section.innerHTML.trim();
    } else if (section.previousElementSibling?.rawText.includes("Mirrors")) {
      parsedContent.mirrors = section.querySelectorAll("li").map((mirror) =>
        mirror.querySelectorAll("a").reduce((acc, a, index) => {
          if (index == 0) {
            return {
              name: decode(a.rawText),
              links: [{ name: a.rawText, link: a.attrs.href }],
            };
          } else {
            return {
              ...acc,
              links: [
                ...acc.links,
                { name: decode(a.rawText), link: a.attrs.href },
              ],
            };
          }
        }, {} as NonNullable<ParsedContentInput["mirrors"]>[number])
      );
    }
  }

  let { genres, companies, languages, originalSize, repackSize } = content
    .querySelectorAll("h3 + p strong")
    .reduce((acc, info) => {
      const category = info.previousSibling?.rawText.toLowerCase();
      if (!category) throw new Error("Category is missing");

      if (category.includes("genre")) {
        return {
          ...acc,
          genres: info.rawText
            .split(", ")
            .flatMap((x) => x.split("/"))
            .map(decode),
        };
      } else if (category.includes("compan")) {
        return {
          ...acc,
          companies: info.rawText
            .split(", ")
            .flatMap((x) => x.split("/"))
            .map(decode),
        };
      } else if (category.includes("language")) {
        return {
          ...acc,
          languages: info.rawText
            .split("/")
            .flatMap((x) => x.split("/"))
            .map((x) => decode(x.toUpperCase())),
        };
      } else if (category.includes("original")) {
        return { ...acc, originalSize: decode(info.rawText) };
      } else if (category.includes("repack")) {
        return { ...acc, repackSize: decode(info.rawText) };
      } else {
        return acc;
      }
    }, {} as ParsedContentInput);

  parsedContent = {
    ...parsedContent,
    genres,
    companies,
    languages,
    originalSize,
    repackSize,
  };

  parsedContent.gameDescription = content
    .querySelector("h3+ul+.su-spoiler .su-spoiler-content")
    ?.innerHTML.trim();

  return parsedContentSchema.parse(parsedContent);
}

async function storeGame(parsedContent: ParsedContent) {
  for (const lang of parsedContent.languages) {
    await db.insert(Language).values({ name: lang }).onConflictDoNothing();
  }

  for (const company of parsedContent.companies) {
    await db.insert(Company).values({ name: company }).onConflictDoNothing();
  }

  for (const genre of parsedContent.genres) {
    await db.insert(Genre).values({ name: genre }).onConflictDoNothing();
  }

  const [{ id: releaseId }] = await db.insert(Release).values({
    id: crypto.randomUUID(),
    slug: slug(parsedContent.title),
    title: parsedContent.title,
    link: parsedContent.link,
    published: parsedContent.published,
    coverSrc: parsedContent.coverSrc,
    originalSize: parsedContent.originalSize,
    repackSize: parsedContent.repackSize,
    mirrors: JSON.stringify(parsedContent.mirrors),
    screenshots: JSON.stringify(parsedContent.screenshots ?? []),
    repackDescription: parsedContent.repackDescription ?? '',
    gameDescription: parsedContent.gameDescription ?? '',
  }).returning({ id: Release.id });

  for (const language of parsedContent.languages) {
    await db.insert(ReleaseLanguages).values({ language, releaseId }).onConflictDoNothing();
  }

  for (const company of parsedContent.companies) {
    await db.insert(ReleaseCompanies).values({ company, releaseId }).onConflictDoNothing();
  }

  for (const genre of parsedContent.genres) {
    await db.insert(ReleaseGenres).values({ genre, releaseId }).onConflictDoNothing();
  }
}

export async function syncAll() {
  const existingTitles = await db.select({ title: Release.title }).from(Release);

  console.log(`Found ${existingTitles.length} existing titles`);

  let p = 1;
  let fullGameList: Awaited<ReturnType<typeof getGameList>> = [];
  while (true) {
    const gameList = await getGameList(p++);

    if (gameList.length == 0) {
      break;
    }

    fullGameList = [...fullGameList, ...gameList];
  }

  const filteredGameList = fullGameList.filter(({ title }) => !existingTitles.some(({ title: t }) => t == title));

  let addedGames: string[] = []

  for (const game of filteredGameList) {
    try {
      const release = await getGame(game.link)
      console.log("parsed game", release.title)
      await storeGame(release);
      console.log("stored game", release.title)
      addedGames.push(release.title);
    } catch (e) {
      if (e instanceof Error) {
        console.error(e.stack + "\n while fetching " + game.title, true);
      }
    }
  }

  return addedGames
}

export async function syncRss() {
  const parser: Parser<{}, { title: string, link: string, pubDate: string, 'content:encoded': string }> = new Parser();
  const feed = await parser.parseURL('https://fitgirl-repacks.site/feed/')
  const releases = feed.items.filter(x => x.categories?.includes('Lossless Repack'))
  const [{ title: lastReleaseTitle }] = await db.select({ title: Release.title }).from(Release).limit(1).orderBy(desc(Release.published))

  let addedGames: string[] = []

  for (const releaseToAdd of releases) {
    console.log(releaseToAdd.title, lastReleaseTitle)
    if (releaseToAdd.title == lastReleaseTitle) break;

    const release = await getGame(releaseToAdd.link)
    console.log("parsed game", release.title)
    await storeGame(release);
    console.log("stored game", release.title)
    addedGames.push(release.title);
  }

  return addedGames
}