import { column, defineDb, defineTable } from 'astro:db';

const Company = defineTable({
  columns: {
    name: column.text({ primaryKey: true, unique: true })
  }
})

const Genre = defineTable({
  columns: {
    name: column.text({ primaryKey: true, unique: true })
  }
})

const Language = defineTable({
  columns: {
    name: column.text({ primaryKey: true, unique: true })
  }
})

const ReleaseCompanies = defineTable({
  columns: {
    releaseId: column.text({ references: () => Release.columns.id }),
    company: column.text({ references: () => Company.columns.name })
  },
  indexes: [{ on: ["releaseId", "company"], unique: true }],
})
const ReleaseGenres = defineTable({
  columns: {
    releaseId: column.text({ references: () => Release.columns.id }),
    genre: column.text({ references: () => Genre.columns.name })
  },
  indexes: [{ on: ["releaseId", "genre"], unique: true }],
})
const ReleaseLanguages = defineTable({
  columns: {
    releaseId: column.text({ references: () => Release.columns.id }),
    language: column.text({ references: () => Language.columns.name })
  },
  indexes: [{ on: ["releaseId", "language"], unique: true }],
})

const Release = defineTable({
  columns: {
    id: column.text({ primaryKey: true, unique: true }),
    slug: column.text({ unique: true }),
    title: column.text(),
    link: column.text(),
    published: column.date(),
    coverSrc: column.text(),
    originalSize: column.text(),
    repackSize: column.text(),
    mirrors: column.json(),
    screenshots: column.json(),
    repackDescription: column.text({ multiline: true }),
    gameDescription: column.text(),
  }
})

export default defineDb({
  tables: {
    Release,
    Company,
    Genre,
    Language,
    ReleaseCompanies,
    ReleaseGenres,
    ReleaseLanguages,
  },
});
