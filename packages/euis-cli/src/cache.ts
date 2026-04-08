import * as crypto from 'crypto';
import { CompileResult } from './types';

export interface CacheConfig {
  /** Maximum number of cached entries (default: 1000) */
  maxSize: number;
  /** Maximum age in milliseconds (default: 1 hour) */
  maxAge: number;
  /** Enable/disable caching (default: true) */
  enabled: boolean;
}

interface CacheEntry {
  result: CompileResult;
  timestamp: number;
}

/**
 * Compilation cache for avoiding redundant compilations.
 * Uses content + config hashing for cache keys.
 */
export class CompilationCache {
  private cache = new Map<string, CacheEntry>();
  private config: CacheConfig;

  constructor(config: Partial<CacheConfig> = {}) {
    this.config = {
      maxSize: config.maxSize ?? 1000,
      maxAge: config.maxAge ?? 3600000, // 1 hour
      enabled: config.enabled ?? true,
    };
  }

  /**
   * Generate a cache key from source content and config.
   */
  getCacheKey(source: string, configJson: string): string {
    const hash = crypto.createHash('md5');
    hash.update(source);
    hash.update(configJson);
    return hash.digest('hex');
  }

  /**
   * Get a cached result if available and not expired.
   */
  get(key: string): CompileResult | null {
    if (!this.config.enabled) return null;

    const entry = this.cache.get(key);
    if (!entry) return null;

    // Check expiration
    if (Date.now() - entry.timestamp > this.config.maxAge) {
      this.cache.delete(key);
      return null;
    }

    return entry.result;
  }

  /**
   * Store a compilation result in the cache.
   */
  set(key: string, result: CompileResult): void {
    if (!this.config.enabled) return;

    // Evict oldest entries if cache is full
    if (this.cache.size >= this.config.maxSize) {
      const oldestKey = this.cache.keys().next().value;
      if (oldestKey) this.cache.delete(oldestKey);
    }

    this.cache.set(key, {
      result,
      timestamp: Date.now(),
    });
  }

  /**
   * Invalidate a specific cache entry.
   */
  invalidate(key: string): void {
    this.cache.delete(key);
  }

  /**
   * Clear all cache entries.
   */
  clear(): void {
    this.cache.clear();
  }

  /**
   * Get the current cache size.
   */
  get size(): number {
    return this.cache.size;
  }
}

/** Shared cache instance for the CLI */
export const globalCache = new CompilationCache();
