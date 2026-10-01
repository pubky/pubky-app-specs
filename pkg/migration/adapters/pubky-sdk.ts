// The port over a session of the pubky SDK, `@synonymdev/pubky` >=0.11 <1: the session
// storage it calls has the same shape and answers from 0.11 to 0.14. Only its types are
// imported: the adapter works on the session it is given, so the host installs the SDK and the
// engine never loads it. The package does not declare it as a peer dependency, since a host on
// another SDK line would fail to install.

import type { Path, PubkyErrorName, Session } from "@synonymdev/pubky";
import { MigrationPortError, refusal } from "../port.js";
import type { MigrationPort, PortErrorKind, PutOptions } from "../port.js";

export interface SdkPortOptions {
  /** URLs per LIST page, 1 to 1000; the homeserver caps it at 1000. */
  pageSize?: number;
}

const MAX_PAGE = 1000;

// What a homeserver without the private root answers a request under `/priv/`. A current one
// names both roots in the same refusal, for paths outside them, which the engine never asks.
const PRE_PRIV = "other than '/pub/' is forbidden";

// Without a status, the SDK's name says whether the request went out at all
const NAMED: Partial<Record<PubkyErrorName, PortErrorKind>> = {
  AuthenticationError: "unauthorized",
  InvalidInput: "rejected",
  ClientStateError: "rejected",
  InternalError: "rejected",
};

const statusOf = (error: unknown): number | undefined => {
  const status = (error as { data?: { statusCode?: unknown } } | null)?.data?.statusCode;
  return typeof status === "number" ? status : undefined;
};

const portError = (error: unknown): MigrationPortError => {
  const message = error instanceof Error ? error.message : String(error);
  const status = statusOf(error);
  if (status === undefined) {
    const name = error instanceof Error ? error.name : "";
    return new MigrationPortError(NAMED[name as PubkyErrorName] ?? "network", message);
  }
  if (status === 403 && message.includes(PRE_PRIV)) return new MigrationPortError("unsupported", message, status);
  // The SDK reads a 410 as missing, as its `exists` does
  if (status === 410) return new MigrationPortError("not_found", message, status);
  return refusal(status, message);
};

class SdkPort implements MigrationPort {
  readonly #storage: Session["storage"];
  readonly #ownerPrefix: string;
  readonly #pageSize: number;

  constructor(session: Session, options: SdkPortOptions = {}) {
    const pageSize = options.pageSize ?? MAX_PAGE;
    if (!Number.isInteger(pageSize) || pageSize < 1 || pageSize > MAX_PAGE) {
      throw new RangeError(`sdkPort: pageSize must be an integer from 1 to ${MAX_PAGE}, not ${pageSize}`);
    }
    this.#storage = session.storage;
    this.#ownerPrefix = `pubky://${session.info.publicKey.z32()}/`;
    this.#pageSize = pageSize;
  }

  async list(prefixUrl: string, cursor?: string): Promise<{ urls: string[]; next?: string }> {
    // The SDK lists directories only, and a cursor is a URL it listed
    const path = this.#path(prefixUrl);
    if (!path.endsWith("/")) {
      throw new MigrationPortError("rejected", `${prefixUrl}: a LIST prefix must end with /`);
    }
    let urls: string[];
    try {
      urls = await this.#storage.list(path, cursor ?? null, false, this.#pageSize, false);
    } catch (error) {
      const failure = portError(error);
      if (failure.kind === "not_found") return { urls: [] };
      throw failure;
    }
    // A server or a proxy may cap the page below the size asked, so only an empty page ends the walk
    return urls.length > 0 ? { urls, next: urls[urls.length - 1] } : { urls };
  }

  async get(url: string): Promise<Uint8Array | null> {
    const path = this.#path(url);
    try {
      return await this.#storage.getBytes(path);
    } catch (error) {
      const failure = portError(error);
      if (failure.kind === "not_found") return null;
      throw failure;
    }
  }

  async head(url: string): Promise<boolean> {
    const path = this.#path(url);
    try {
      return await this.#storage.exists(path);
    } catch (error) {
      const failure = portError(error);
      if (failure.status !== 403) throw failure;
      // A HEAD carries no body, so the reason of the refusal is read from a GET, whose body is
      // dropped unread when it succeeds
      try {
        const response = await this.#storage.get(path);
        await response.body?.cancel();
        return true;
      } catch (retry) {
        const refused = portError(retry);
        if (refused.kind === "not_found") return false;
        throw refused;
      }
    }
  }

  /**
   * With `ifAbsent`, a HEAD then the PUT: the homeserver ignores `If-None-Match` on a PUT, so
   * this is check-then-write, and a write landing between the two is overwritten. A PUT whose
   * answer was lost is retried the same way, so the HEAD finds the copy it made and it throws
   * `exists`.
   */
  async putJson(url: string, object: unknown, options?: PutOptions): Promise<void> {
    const path = this.#path(url);
    await this.#absent(url, options);
    await this.#call(() => this.#storage.putJson(path, object));
  }

  /** `ifAbsent` as `putJson` does it. */
  async putBytes(url: string, bytes: Uint8Array, options?: PutOptions): Promise<void> {
    const path = this.#path(url);
    await this.#absent(url, options);
    await this.#call(() => this.#storage.putBytes(path, bytes));
  }

  async delete(url: string): Promise<void> {
    const path = this.#path(url);
    await this.#call(() => this.#storage.delete(path));
  }

  async #absent(url: string, options?: PutOptions): Promise<void> {
    if (options?.ifAbsent && (await this.head(url))) {
      throw new MigrationPortError("exists", `${url} exists`);
    }
  }

  async #call<T>(call: () => Promise<T>): Promise<T> {
    try {
      return await call();
    } catch (error) {
      throw portError(error);
    }
  }

  #path(url: string): Path {
    const path = url.startsWith(this.#ownerPrefix) ? url.slice(this.#ownerPrefix.length - 1) : "";
    if (!path.startsWith("/pub/") && !path.startsWith("/priv/")) {
      throw new MigrationPortError("rejected", `${url} is not under /pub/ or /priv/ of the session's owner`);
    }
    return path as Path;
  }
}

/**
 * The migration port over a signed-in session of `@synonymdev/pubky` >=0.11 <1: every URL has to
 * be in the session owner's tree. `ifAbsent` is a HEAD then the PUT, which leaves a one round
 * trip window.
 */
const sdkPort = (session: Session, options?: SdkPortOptions): MigrationPort => new SdkPort(session, options);

export { sdkPort };
