export const API_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://127.0.0.1:5000";

export type MetadataFilter = {
  key: string;
  value: string | number | boolean;
};

export type SearchRequest = {
  query: string;
  top_k: number;
  document_id: string | null;
  min_score: number | null;
  filters: MetadataFilter[];
};

export type SearchResult = {
  document_id: string;
  chunk_id: string;
  chunk_index: number;
  text: string;
  score: number;
  metadata: Record<string, unknown>;
};

export type SearchResponse = {
  results: SearchResult[];
};

export type DocumentSummary = {
  document_id: string;
  chunks: number;
};

export type DocumentsResponse = {
  documents: DocumentSummary[];
};

export type UploadPdfResponse = {
  document_id: string;
  chunks_indexed: number;
};

export type DeleteDocumentResponse = {
  document_id: string;
  chunks_deleted: number;
};

export class ApiError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ApiError";
  }
}

function apiBase(): string {
  if (typeof window === "undefined") {
    return API_URL;
  }

  // The browser calls a same-origin rewrite. next.config.ts forwards it to API_URL.
  return "/backend";
}

function readableError(data: unknown, status: number): string {
  if (!data || typeof data !== "object") {
    return `Request failed (${status}).`;
  }

  const record = data as Record<string, unknown>;

  if (typeof record.error === "string" && record.error.trim() !== "") {
    return record.error;
  }

  if (typeof record.detail === "string") {
    if (record.detail.includes("Traceback") || record.detail.length > 300) {
      return `Request failed (${status}).`;
    }
    return record.detail;
  }

  if (Array.isArray(record.detail)) {
    const messages = record.detail
      .map((item) => {
        if (item && typeof item === "object" && "msg" in item && typeof item.msg === "string") {
          return item.msg;
        }
        return null;
      })
      .filter((item): item is string => item !== null);

    if (messages.length > 0) {
      return messages.join(" ");
    }
  }

  return `Request failed (${status}).`;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let response: Response;

  try {
    const headers = new Headers(init?.headers);
    if (!(init?.body instanceof FormData) && !headers.has("Content-Type")) {
      headers.set("Content-Type", "application/json");
    }

    response = await fetch(`${apiBase()}${path}`, {
      ...init,
      headers,
      cache: "no-store",
    });
  } catch {
    throw new ApiError("Could not reach the RECALL API.");
  }

  const text = await response.text();
  let data: unknown = null;

  if (text) {
    try {
      data = JSON.parse(text);
    } catch {
      data = null;
    }
  }

  if (!response.ok) {
    throw new ApiError(readableError(data, response.status));
  }

  if (data && typeof data === "object" && "error" in data) {
    const message = (data as { error?: unknown }).error;
    if (typeof message === "string" && message.trim() !== "") {
      throw new ApiError(message);
    }
  }

  if (data === null) {
    throw new ApiError("The RECALL API returned an empty response.");
  }

  return data as T;
}

export async function checkHealth(): Promise<boolean> {
  try {
    const data = await request<{ status?: string }>("/health");
    return data.status === "ok";
  } catch {
    return false;
  }
}

export function listDocuments(): Promise<DocumentsResponse> {
  return request<DocumentsResponse>("/documents");
}

export function searchDocuments(payload: SearchRequest): Promise<SearchResponse> {
  return request<SearchResponse>("/search", {
    method: "POST",
    body: JSON.stringify(payload),
  });
}

export function uploadPdf(file: File): Promise<UploadPdfResponse> {
  const body = new FormData();
  body.append("file", file);

  return request<UploadPdfResponse>("/documents/pdf", {
    method: "POST",
    body,
  });
}

export function deleteDocument(documentId: string): Promise<DeleteDocumentResponse> {
  return request<DeleteDocumentResponse>(`/documents/${encodeURIComponent(documentId)}`, {
    method: "DELETE",
  });
}
