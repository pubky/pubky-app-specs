// The package entry. Every function here is the wasm export of the same name behind three
// checks: the wasm is loaded, each argument has the type its slot takes, and every string
// argument is well-formed UTF-16. A Rust string cannot hold a lone surrogate, and the boundary
// would replace one silently, so a value would be validated and stored as something the caller
// never wrote. Objects cross as JSON.stringify text, whose parser refuses a lone surrogate.
//
// index.cjs is generated from this file at build time: keep relative imports of `.js` files
// and one closing `export { ... };`.

import * as glue from "./pubky_social_specs.js";
import { validationLimits } from "./validationLimits.js";
import { validMimeTypes, mimeToExtTable } from "./mimeTypes.js";

const MALFORMED = "Validation Error: text must be well-formed UTF-16";

let ready = false;
let loading = null;

/** Loads the wasm. Call once and await it before anything else; later calls are free. */
function init() {
  if (!loading) {
    loading = glue.__wbg_init().then(
      () => {
        ready = true;
      },
      (error) => {
        // A failed load can be retried
        loading = null;
        throw error;
      },
    );
  }
  return loading;
}

function wellFormed(text) {
  if (typeof text.isWellFormed === "function") return text.isWellFormed();
  for (let i = 0; i < text.length; i++) {
    const unit = text.charCodeAt(i);
    if (unit >= 0xdc00 && unit <= 0xdfff) return false;
    if (unit >= 0xd800 && unit <= 0xdbff) {
      const next = text.charCodeAt(i + 1);
      if (!(next >= 0xdc00 && next <= 0xdfff)) return false;
      i++;
    }
  }
  return true;
}

// What each argument slot takes. A value of another type never reaches the wasm, where a
// non-string in a string slot reads memory it does not own.
const isObject = (v) => typeof v === "object" && v !== null && !Array.isArray(v);
const KINDS = {
  string: [(v) => typeof v === "string", "a string"],
  "string?": [(v) => v === undefined || v === null || typeof v === "string", "a string or absent"],
  // Any realm's Uint8Array (or a Buffer): a view of single bytes that is not a DataView
  bytes: [
    (v) => ArrayBuffer.isView(v) && v.BYTES_PER_ELEMENT === 1 && !(v instanceof DataView),
    "a Uint8Array",
  ],
  object: [isObject, "an object"],
  "object?": [(v) => v === undefined || v === null || isObject(v), "an object or absent"],
  array: [(v) => Array.isArray(v), "an array"],
  // Array.from visits the holes of a sparse array, which every() skips
  strings: [
    (v) => Array.isArray(v) && Array.from(v).every((s) => typeof s === "string"),
    "an array of strings",
  ],
};

// A string argument, and each string of a `strings` slot, reaches the wasm as it is
function wellFormedArgument(slot, value) {
  if (typeof value === "string") return wellFormed(value);
  return slot !== "strings" || value.every(wellFormed);
}

function wrap(name, ...slots) {
  const inner = glue[name];
  if (typeof inner !== "function") throw new Error(`pubky-social-specs: the build lacks ${name}`);
  return (...args) => {
    if (!ready) {
      throw new Error(`pubky-social-specs: await init() before calling ${name}()`);
    }
    if (args.length > slots.length) {
      throw new Error(`Validation Error: ${name}() takes at most ${slots.length} arguments`);
    }
    slots.forEach((slot, i) => {
      const [accepts, what] = KINDS[slot];
      if (!accepts(args[i])) {
        throw new Error(`Validation Error: ${name}() argument ${i + 1} must be ${what}`);
      }
      if (!wellFormedArgument(slot, args[i])) throw new Error(MALFORMED);
    });
    return inner(...args);
  };
}

// Reading
const parseUri = wrap("parseUri", "string");
const stableId = wrap("stableId", "string");
const resolveDeref = wrap("resolveDeref", "string", "string");
const readObject = wrap("readObject", "string", "bytes");
const validate = wrap("validate", "string", "object");
// Profile and posts
const createUser = wrap("createUser", "string", "object");
const createPost = wrap("createPost", "string", "object");
const createArticlePost = wrap("createArticlePost", "string", "object");
const createCollectionPost = wrap("createCollectionPost", "string", "object");
const createVersion = wrap("createVersion", "string", "object", "object?");
const editVersion = wrap("editVersion", "string", "object", "object");
const planPublish = wrap("planPublish", "string", "string", "string", "object");
const planUnpublish = wrap("planUnpublish", "string", "strings", "strings", "string?");
const planDelete = wrap("planDelete", "string", "string", "strings", "array", "array");
// Feeds
const createFeed = wrap("createFeed", "string", "object");
const feedId = wrap("feedId", "object");
const feedPaths = wrap("feedPaths", "string");
const feedLifecycle = wrap("feedLifecycle", "string");
// Tags, bookmarks, graph
const createTag = wrap("createTag", "string", "string", "string");
const createBookmark = wrap("createBookmark", "string", "string");
const bookmarkFilename = wrap("bookmarkFilename", "string");
const bookmarkTarget = wrap("bookmarkTarget", "string", "object?");
const createFollow = wrap("createFollow", "string", "string");
const createMute = wrap("createMute", "string", "string");
// Media
const createFile = wrap("createFile", "string", "bytes", "string", "string?");
const mimeToExt = wrap("mimeToExt", "string");
const essence = wrap("essence", "string");
// Deletion, prefixes and URIs
const deletionPaths = wrap("deletionPaths", "object");
const listPrefix = wrap("listPrefix", "string", "string");
const legacyListPrefix = wrap("legacyListPrefix", "string");
const userUriBuilder = wrap("userUriBuilder", "string");
const postUriBuilder = wrap("postUriBuilder", "string", "string");
const followUriBuilder = wrap("followUriBuilder", "string", "string");
const muteUriBuilder = wrap("muteUriBuilder", "string", "string");
const bookmarkUriBuilder = wrap("bookmarkUriBuilder", "string", "string");
const tagUriBuilder = wrap("tagUriBuilder", "string", "string");
const fileUriBuilder = wrap("fileUriBuilder", "string", "string");
const feedUriBuilder = wrap("feedUriBuilder", "string", "string");

export {
  init,
  validationLimits,
  validMimeTypes,
  mimeToExtTable,
  parseUri,
  stableId,
  resolveDeref,
  readObject,
  validate,
  createUser,
  createPost,
  createArticlePost,
  createCollectionPost,
  createVersion,
  editVersion,
  planPublish,
  planUnpublish,
  planDelete,
  createFeed,
  feedId,
  feedPaths,
  feedLifecycle,
  createTag,
  createBookmark,
  bookmarkFilename,
  bookmarkTarget,
  createFollow,
  createMute,
  createFile,
  mimeToExt,
  essence,
  deletionPaths,
  listPrefix,
  legacyListPrefix,
  userUriBuilder,
  postUriBuilder,
  followUriBuilder,
  muteUriBuilder,
  bookmarkUriBuilder,
  tagUriBuilder,
  fileUriBuilder,
  feedUriBuilder,
};
