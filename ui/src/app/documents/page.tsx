"use client";

import { ChangeEvent, useEffect, useState } from "react";
import { DocumentList } from "@/components/DocumentList";
import {
  deleteDocument,
  listDocuments,
  uploadPdf,
  type DocumentSummary,
  type UploadPdfResponse,
} from "@/lib/api";

export default function DocumentsPage() {
  const [documents, setDocuments] = useState<DocumentSummary[]>([]);
  const [loading, setLoading] = useState(true);
  const [uploading, setUploading] = useState(false);
  const [deletingId, setDeletingId] = useState<string | null>(null);
  const [uploaded, setUploaded] = useState<UploadPdfResponse | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;

    listDocuments()
      .then((data) => {
        if (!cancelled) {
          setDocuments(data.documents);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : "Could not load documents.");
        }
      })
      .finally(() => {
        if (!cancelled) {
          setLoading(false);
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  async function refreshDocuments() {
    const data = await listDocuments();
    setDocuments(data.documents);
  }

  async function handleUpload(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0];
    event.target.value = "";

    if (!file) {
      return;
    }

    setUploading(true);
    setError(null);
    setUploaded(null);

    try {
      const result = await uploadPdf(file);
      setUploaded(result);
      await refreshDocuments();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Upload failed.");
    } finally {
      setUploading(false);
    }
  }

  async function handleDelete(documentId: string) {
    const confirmed = window.confirm(`Delete ${documentId}?`);
    if (!confirmed) {
      return;
    }

    setDeletingId(documentId);
    setError(null);

    try {
      await deleteDocument(documentId);
      if (uploaded?.document_id === documentId) {
        setUploaded(null);
      }
      await refreshDocuments();
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : "Delete failed.");
    } finally {
      setDeletingId(null);
    }
  }

  const busy = uploading || deletingId !== null;

  return (
    <section>
      <div className="flex flex-wrap items-center justify-between gap-4">
        <h1 className="text-lg text-[#f5f5f5]">Documents</h1>
        <label
          className={`border border-[#ececec] bg-[#ececec] px-4 py-2 text-sm text-[#090909] ${
            busy ? "pointer-events-none opacity-40" : "cursor-pointer"
          }`}
        >
          <input
            type="file"
            accept="application/pdf,.pdf"
            className="sr-only"
            onChange={handleUpload}
            disabled={busy}
          />
          {uploading ? "Uploading..." : "Upload PDF"}
        </label>
      </div>

      {uploaded ? (
        <p className="mt-4 text-sm text-[#c4c4c4]">
          Uploaded <span className="text-[#f5f5f5]">{uploaded.document_id}</span>
          {" · "}
          {uploaded.chunks_indexed} chunks indexed
        </p>
      ) : null}
      {error ? <p className="mt-4 text-sm text-[#d7a0a0]">{error}</p> : null}
      {loading ? <p className="mt-8 text-sm text-[#8d8d8d]">Loading...</p> : null}
      {!loading && (documents.length > 0 || error === null) ? (
        <DocumentList documents={documents} deletingId={deletingId} onDelete={handleDelete} />
      ) : null}
    </section>
  );
}
