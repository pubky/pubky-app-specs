// The walk order over the 0.x tree. The File objects come first because the run reads them
// to rewrite every media reference after them; the rest is leaf first, so a reference tends
// to land after its target, and the private types last.

const BUCKETS = [
  "files",
  "blobs",
  "posts",
  "tags",
  "follows",
  "profile",
  "feeds",
  "bookmarks",
  "mutes",
] as const;

/** One pass of the walk, named by the 0.x resource segment. */
export type Bucket = (typeof BUCKETS)[number];

/**
 * The bucket of a path relative to the 0.x namespace (`posts/X`, `profile.json`), or `null`
 * for one no pass migrates, such as `settings.json` and `last_read`.
 */
const bucketOf = (legacyRelative: string): Bucket | null => {
  if (legacyRelative === "profile.json") return "profile";
  const slash = legacyRelative.indexOf("/");
  if (slash <= 0 || slash === legacyRelative.length - 1) return null;
  const segment = legacyRelative.slice(0, slash);
  return (BUCKETS as readonly string[]).includes(segment) && segment !== "profile"
    ? (segment as Bucket)
    : null;
};

/** Items grouped by bucket in walk order, and the ones no pass takes. */
const ordered = <T>(
  items: T[],
  legacyRelative: (item: T) => string,
): { passes: [Bucket, T[]][]; rest: T[] } => {
  const groups = new Map<Bucket, T[]>(BUCKETS.map((bucket) => [bucket, []]));
  const rest: T[] = [];
  for (const item of items) {
    const bucket = bucketOf(legacyRelative(item));
    if (bucket === null) rest.push(item);
    else groups.get(bucket)!.push(item);
  }
  return { passes: [...groups], rest };
};

export { BUCKETS, bucketOf, ordered };
