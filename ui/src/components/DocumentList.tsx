"use client";

import type { DocumentSummary } from "@/lib/api";

type DocumentListProps = {
  documents: DocumentSummary[];
  deletingId: string | null;
  onDelete: (documentId: string) => void;
};

export function DocumentList({ documents, deletingId, onDelete }: DocumentListProps) {
  if (documents.length === 0) {
    return <p className="mt-8 text-sm text-[#8d8d8d]">No documents indexed.</p>;
  }

  return (
    <div className="mt-8 border-t border-[#242424]">
      <div className="hidden grid-cols-[1fr_6rem_7rem] gap-4 border-b border-[#242424] py-2 text-xs text-[#6f6f6f] sm:grid">
        <span>Document ID</span>
        <span>Chunks</span>
        <span />
      </div>
      <ul>
        {documents.map((document) => {
          const deleting = deletingId === document.document_id;

          return (
            <li
              key={document.document_id}
              className="grid grid-cols-1 gap-2 border-b border-[#242424] py-3 sm:grid-cols-[1fr_6rem_7rem] sm:items-center sm:gap-4"
            >
              <span className="break-all text-sm text-[#ececec]">{document.document_id}</span>
              <span className="text-sm text-[#c4c4c4]">
                <span className="text-[#6f6f6f] sm:hidden">Chunks </span>
                {document.chunks}
              </span>
              <button
                type="button"
                onClick={() => onDelete(document.document_id)}
                disabled={deletingId !== null}
                className="justify-self-start border border-[#333] px-3 py-1 text-xs text-[#d0d0d0] disabled:opacity-40 sm:justify-self-end"
              >
                {deleting ? "Deleting..." : "Delete"}
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
