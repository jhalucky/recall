const RUST_VARIANTS = new Set(["String", "Integer", "Float", "Boolean"]);

export function displayMetadataValue(value: unknown): string {
  if (typeof value === "string") {
    return value;
  }

  if (typeof value === "number" && Number.isFinite(value)) {
    return String(value);
  }

  if (typeof value === "boolean") {
    return value ? "true" : "false";
  }

  if (!value || typeof value !== "object" || Array.isArray(value)) {
    return "";
  }

  const entries = Object.entries(value as Record<string, unknown>);
  if (entries.length !== 1) {
    return "";
  }

  const [variant, inner] = entries[0];
  if (!RUST_VARIANTS.has(variant)) {
    return "";
  }

  return displayMetadataValue(inner);
}

export function pageFromMetadata(metadata: Record<string, unknown>): string | null {
  if (!Object.prototype.hasOwnProperty.call(metadata, "page")) {
    return null;
  }

  const page = displayMetadataValue(metadata.page);
  return page === "" ? null : page;
}

export function metadataRows(
  metadata: Record<string, unknown>,
): { key: string; value: string }[] {
  return Object.entries(metadata)
    .filter(([key]) => key !== "text")
    .map(([key, value]) => ({
      key,
      value: displayMetadataValue(value),
    }))
    .filter((row) => row.value !== "");
}
