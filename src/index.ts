import { createWriteStream, rmSync, unlinkSync } from "fs";
import { HTMLElement, parse as parseHTML } from "node-html-parser";
import { z } from "zod";
import Database from "better-sqlite3";
import type { Database as DatabaseType } from "better-sqlite3";
import { v4 as uuidv4 } from "uuid";
import { setTimeout } from "timers/promises";

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
    .replace(/&#(\d+);/g, function(match, dec) {
      return String.fromCharCode(dec);
    })
    .replaceAll("’", "'");
}

async function getGameList(page = 1) {
  const res = await fetch(`${base_url}/all-my-repacks-a-z/?lcp_page0=${page}`);
  const html = await res.text();

  const root = parseHTML(html);

  return root
    .querySelector(".lcp_catlist")
    .querySelectorAll("li")
    .map((li) => {
      const a = li.querySelector("a");
      const link = a.attrs.href;
      const title = decode(a.rawText);
      return { title, link };
    });
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
    const elements = [];
    let currentEl = header.nextElementSibling;
    while (true) {

      elements.push(currentEl);

      currentEl = currentEl.nextElementSibling;
      if (!currentEl || currentEl.tagName === "STYLE") break;
    }

    content = parseHTML("<div>" + elements.map(el => el.outerHTML).join("") + "</div>");
  }

  if (!content) {
    throw new Error("unable to parse html correctly");
  }

  parsedContent.published = new Date(
    root.querySelector('meta[property="article:published_time"]').attrs.content
  );

  parsedContent.title = decode(root.querySelector(".entry-title")?.rawText);
  parsedContent.link = url;
  parsedContent.coverSrc = content.querySelector("h3 + p > a > img")?.attrs.src;

  const sections = content.querySelectorAll("h3 + *");

  for (const section of sections) {
    if (section.previousElementSibling.rawText.includes("Screenshots")) {
      parsedContent.screenshots = content
        .querySelectorAll("a > img")
        .map((img) => img.attrs.src);
    } else if (section.previousElementSibling.rawText.includes("Repack")) {
      parsedContent.repackDescription = section.innerHTML.trim();
    } else if (section.previousElementSibling.rawText.includes("Mirrors")) {
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
      const category = info.previousSibling.rawText.toLowerCase();

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
    .querySelector(".su-spoiler-content")
    ?.innerHTML.trim();

  return parsedContentSchema.parse(parsedContent);
}

function storeGame(db: DatabaseType, parsedContent: ParsedContent) {
  const langStmt = db.prepare(
    `INSERT OR IGNORE INTO languages (value) VALUES (@value)`
  );
  parsedContent.languages.forEach((lang) => langStmt.run({ value: lang }));

  const companiesStmt = db.prepare(
    `INSERT OR IGNORE INTO companies (value) VALUES (@value)`
  );
  parsedContent.companies.forEach((company) =>
    companiesStmt.run({ value: company })
  );

  const genresStmt = db.prepare(
    `INSERT OR IGNORE INTO genres (value) VALUES (@value)`
  );
  parsedContent.genres.forEach((genre) => genresStmt.run({ value: genre }));

  const stmt = db.prepare(
    `INSERT INTO releases (
            id,
            title,
            link,
            published,
            coverSrc,
            originalSize,
            repackSize,
            mirrors,
            screenshots,
            repackDescription,
            gameDescription
          ) VALUES (
            @id,
            @title,
            @link,
            @published,
            @coverSrc,
            @originalSize,
            @repackSize,
            @mirrors,
            @screenshots,
            @repackDescription,
            @gameDescription
          )`
  );

  const id = uuidv4();

  stmt.run({
    id,
    title: parsedContent.title,
    link: parsedContent.link,
    published: parsedContent.published.toISOString(),
    coverSrc: parsedContent.coverSrc,
    genres: JSON.stringify(parsedContent.genres),
    companies: JSON.stringify(parsedContent.companies),
    languages: JSON.stringify(parsedContent.languages),
    originalSize: parsedContent.originalSize,
    repackSize: parsedContent.repackSize,
    mirrors: JSON.stringify(parsedContent.mirrors),
    screenshots: JSON.stringify(parsedContent.screenshots),
    repackDescription: parsedContent.repackDescription,
    gameDescription: parsedContent.gameDescription,
  });

  parsedContent.languages.forEach((lang) => {
    const langLinkStmt = db.prepare(
      `INSERT OR IGNORE INTO release_language (release_id, language) VALUES (@releaseId, @language)`
    );
    langLinkStmt.run({ releaseId: id, language: lang });
  });

  const genreLinkStmt = db.prepare(
    `INSERT OR IGNORE INTO release_genre (release_id, genre) VALUES (@releaseId, @genre)`
  );
  parsedContent.genres.forEach((genre) =>
    genreLinkStmt.run({ releaseId: id, genre })
  );

  const companyLinkStmt = db.prepare(
    `INSERT OR IGNORE INTO release_company (release_id, company) VALUES (@releaseId, @company)`
  );
  parsedContent.companies.forEach((company) =>
    companyLinkStmt.run({ releaseId: id, company })
  );
}

(async function main(): Promise<void> {
  // rmSync("fitgirl.db", { force: true });
  const db = new Database("fitgirl.db");
  db.pragma("journal_mode = WAL");

  db.exec(`
  CREATE TABLE IF NOT EXISTS genres (
    value TEXT PRIMARY KEY
  );

  CREATE TABLE IF NOT EXISTS languages (
    value TEXT PRIMARY KEY
  );

  CREATE TABLE IF NOT EXISTS companies (
    value TEXT PRIMARY KEY
  );

  CREATE TABLE IF NOT EXISTS releases (
    id TEXT PRIMARY KEY,
    title TEXT,
    link TEXT,
    published DATETIME,
    coverSrc TEXT,
    originalSize TEXT,
    repackSize TEXT,
    mirrors TEXT,
    screenshots TEXT,
    repackDescription TEXT,
    gameDescription TEXT
  );

  CREATE TABLE IF NOT EXISTS release_genre (
    release_id TEXT NOT NULL,
    genre TEXT NOT NULL,
    FOREIGN KEY (release_id ) REFERENCES releases(id),
    FOREIGN KEY ( genre ) REFERENCES genres(value),
    PRIMARY KEY (release_id, genre)
  );

  CREATE TABLE IF NOT EXISTS release_language (
    release_id TEXT NOT NULL,
    language TEXT NOT NULL,
    FOREIGN KEY ( release_id ) REFERENCES releases(id),
    FOREIGN KEY ( language ) REFERENCES languages(value),
    PRIMARY KEY (release_id, language)
  );


  CREATE TABLE IF NOT EXISTS release_company (
    release_id TEXT NOT NULL,
    company TEXT NOT NULL,
    FOREIGN KEY ( release_id ) REFERENCES releases(id),
    FOREIGN KEY ( company ) REFERENCES companies(value),
    PRIMARY KEY (release_id, company)
  );

    `);

  const logFileName = `${new Date().toISOString()}.log`;
  const fileWriteStream = createWriteStream("logs/" + logFileName, {
    flags: "w",
  });

  const log = (message: string, error = false) => {
    if (error) {
      fileWriteStream.write("[ERROR]\n" + message + "\n");
      console.error("[ERROR]", message);
    } else {
      fileWriteStream.write(message + "\n");
      console.log(message);
    }
  };

  const existingTitles = db.prepare(`SELECT title FROM releases`).all() as {
    title: string;
  }[];

  console.log(`Found ${existingTitles.length} existing titles`);

  let p = 1;
  let fullGameList: Awaited<ReturnType<typeof getGameList>> = [];
  while (true) {
    log(`Fetching Page ${p}`);

    const gameList = await getGameList(p++);

    if (gameList.length == 0) {
      break;
    }

    fullGameList = [...fullGameList, ...gameList];
  }

  const filteredGameList = fullGameList.filter(({ title }) => !existingTitles.some(({ title: t }) => t == title));
  const parsedGameList: Array<Awaited<ReturnType<typeof getGame>>> = [];

  for (const game of filteredGameList) {
    log(`Processing ${game.title}`);
    try {
      parsedGameList.push(await getGame(game.link));
    } catch (e) {
      log(e.stack + "\n while fetching " + game.title, true);
    }
  }

  for (const game of parsedGameList.filter(Boolean)) {
    log(`Storing ${game.title}`);
    storeGame(db, game);
  }
})();
