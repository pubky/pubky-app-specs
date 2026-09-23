// Declarations of the package entry. The enums, plans, inputs and every other shape without
// unknown members are generated from the Rust types and re-exported from the bindings; the
// stored objects are written here, because the generator cannot express the members a stored
// object keeps beyond the ones this version knows.

import type {
  CreateArticlePostInput,
  CreateCollectionPostInput,
  CreateFeedInput,
  CreatePostInput,
  CreateUserInput,
  CreateVersionOptions,
  DeletePlan,
  DeletionInput,
  EditVersionOptions,
  FeedLifecycle,
  FeedPaths,
  Meta,
  PubkySocialCollectionLayout,
  PubkySocialFeedLayout,
  PubkySocialFeedReach,
  PubkySocialFeedSort,
  PubkySocialPostKind,
  Root,
  StableKey,
  StoredCopy,
  UnpublishPlan,
  UriParts,
  VersionMeta,
} from "./pubky_social_specs.js";

export * from "./pubky_social_specs.js";
export { validationLimits } from "./validationLimits.js";
export { validMimeTypes, mimeToExtTable } from "./mimeTypes.js";

/**
 * Members this version does not know. A stored object keeps them at runtime through every
 * read, edit and `validate`; the interfaces below leave them out so a misspelled field does
 * not compile. Widen with `PubkySocialUser & Extra` to reach one. Deliberate extensions nest
 * under `ext`; treat everything there as hostile input until the extension's own rules have
 * checked it.
 */
export interface Extra {
  [member: string]: unknown;
}

// ---- stored objects ----

export interface PubkySocialUserLink {
  title: string;
  url: string;
}

export interface PubkySocialUser {
  name: string;
  bio?: string | null;
  image?: string | null;
  links?: PubkySocialUserLink[] | null;
  status?: string | null;
}

export interface PubkySocialAttachment {
  uri: string;
  alt?: string;
  name?: string;
}

export interface PubkySocialPost {
  /** Plain text, or for an article or a collection the JSON envelope as a string. */
  content: string;
  kind: PubkySocialPostKind;
  parent?: string | null;
  embed?: string | null;
  attachments: PubkySocialAttachment[];
  lock?: string;
}

/** `JSON.parse(post.content)` of a `kind: "article"` post. */
export interface PubkySocialArticleContent {
  title: string;
  body: string;
  cover_image?: string;
}

export interface PubkySocialCollectionItem {
  uri: string;
  note?: string;
}

/** `JSON.parse(post.content)` of a `kind: "collection"` post. */
export interface PubkySocialCollectionContent {
  name: string;
  description?: string;
  items: PubkySocialCollectionItem[];
  cover_image?: string;
  layout?: PubkySocialCollectionLayout;
}

export interface PubkySocialTag {
  uri: string;
  label: string;
  created_at: number;
}

export interface PubkySocialBookmark {
  created_at: number;
  /** Only in the overflow form, where the filename cannot carry the target. */
  target?: string;
}

export interface PubkySocialFollow {
  created_at: number;
}

export interface PubkySocialMute {
  created_at: number;
}

export interface PubkySocialFeedConfig {
  tags?: string[] | null;
  domain_tags?: string[];
  reach: PubkySocialFeedReach;
  layout: PubkySocialFeedLayout;
  sort: PubkySocialFeedSort;
  content?: PubkySocialPostKind | null;
}

export interface PubkySocialFeed {
  feed: PubkySocialFeedConfig;
  name: string;
  icon?: string;
  created_at: number;
}

/** Media is raw bytes and has no JSON form. */
export interface PubkySocialFile {
  bytes: Uint8Array;
}

// ---- results ----

/** What every builder returns: the object exactly as it is stored, and where it goes. */
export interface Created<T> {
  object: T;
  meta: Meta;
}

/** What `readObject` returns; `kind` tells which object it is. */
export type ReadObject =
  | { kind: "user"; object: PubkySocialUser }
  | { kind: "post"; object: PubkySocialPost }
  | { kind: "follow"; object: PubkySocialFollow }
  | { kind: "mute"; object: PubkySocialMute }
  | { kind: "bookmark"; object: PubkySocialBookmark }
  | { kind: "tag"; object: PubkySocialTag }
  | { kind: "file"; object: PubkySocialFile }
  | { kind: "feed"; object: PubkySocialFeed };

/** Any stored object, as `validate` takes it. */
export type PubkySocialObject = ReadObject["object"];

/** Publishing one private version: run `mediaCopies` first, then PUT `rewrittenPost` at `destPath`. */
export interface PublishPlan {
  /** `[private path, public path]` pairs, in reference order. */
  mediaCopies: [string, string][];
  /** The chosen version with its private media references respelled under the public root. */
  rewrittenPost: PubkySocialPost;
  destPath: string;
}

// ---- the entry ----

/** Loads the wasm. Await it once before calling anything else; every other call throws until then. */
export function init(): Promise<void>;

export function parseUri(uri: string): UriParts;
export function stableId(ownerRelativePath: string): StableKey | null;
export function resolveDeref(tsid: string, v0FileSrc: string): string | null;
export function readObject(uri: string, bytes: Uint8Array): ReadObject;
export function validate(uri: string, object: PubkySocialObject): void;

export function createUser(owner: string, input: CreateUserInput): Created<PubkySocialUser>;
export function createPost(owner: string, input: CreatePostInput): Created<PubkySocialPost>;
export function createArticlePost(
  owner: string,
  input: CreateArticlePostInput,
): Created<PubkySocialPost>;
export function createCollectionPost(
  owner: string,
  input: CreateCollectionPostInput,
): Created<PubkySocialPost>;
export function createVersion(
  owner: string,
  post: PubkySocialPost,
  options?: CreateVersionOptions | null,
): VersionMeta;
export function editVersion(
  owner: string,
  post: PubkySocialPost,
  options: EditVersionOptions,
): VersionMeta;
export function planPublish(
  owner: string,
  postId: string,
  editId: string,
  post: PubkySocialPost,
): PublishPlan;
export function planUnpublish(
  postId: string,
  publicPaths: string[],
  legacyPaths: string[],
  privateHeadPath?: string | null,
): UnpublishPlan;
export function planDelete(
  owner: string,
  postId: string,
  legacyPaths: string[],
  copies: StoredCopy[],
  versions: PubkySocialPost[],
): DeletePlan;

export function createFeed(owner: string, input: CreateFeedInput): Created<PubkySocialFeed>;
export function feedId(feed: PubkySocialFeed): string;
export function feedPaths(id: string): FeedPaths;
export function feedLifecycle(id: string): FeedLifecycle;

export function createTag(owner: string, uri: string, label: string): Created<PubkySocialTag>;
export function createBookmark(owner: string, target: string): Created<PubkySocialBookmark>;
export function bookmarkFilename(target: string): string;
/** `content` is needed only for a `~` overflow filename. */
export function bookmarkTarget(filename: string, content?: PubkySocialBookmark | null): string;
export function createFollow(owner: string, followee: string): Created<PubkySocialFollow>;
export function createMute(owner: string, mutee: string): Created<PubkySocialMute>;

export function createFile(
  owner: string,
  bytes: Uint8Array,
  declaredType: string,
  root?: Root | null,
): Created<PubkySocialFile>;
export function mimeToExt(declared: string): string;
export function essence(declared: string): string | null;

export function deletionPaths(input: DeletionInput): string[];
export function listPrefix(userId: string, root: Root): string;
export function legacyListPrefix(userId: string): string;
