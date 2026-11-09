/** Shared font-control contract; independent of document DTOs and command ownership. */
export interface FontProperties {
  font?: string | null;
  height?: string | null;
  bold?: boolean | null;
  italic?: boolean | null;
  colour?: string | null;
  horizontal?: string | null;
  vertical?: string | null;
}
export type FontProperty = keyof FontProperties;
export interface FontPaletteOptions {
  /** Hide properties inapplicable to the current text layout, retaining their stored values. */
  exclude?: FontProperty[];
  /** Only these properties offer Default. Omitted means all supported properties. */
  defaults?: FontProperty[];
  /** Mixed values from a projected selection without exposing its individual objects. */
  mixed?: FontProperty[];
  /** Owner-specific accessible labels, retaining the shared control implementation. */
  labels?: Partial<Record<FontProperty, string>>;
  /** Attach applicability/source metadata without redefining a control. */
  field?: (key: FontProperty, input: HTMLInputElement | HTMLTextAreaElement) => void;
}
