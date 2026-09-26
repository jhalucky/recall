"use client";

import { FormEvent, useState, type ReactNode } from "react";
import type { DocumentSummary, SearchRequest } from "@/lib/api";

const inputClass =
  "w-full border border-[#2a2a2a] bg-[#0e0e0e] px-3 py-2 text-sm text-[#ececec] placeholder:text-[#666] focus:border-[#555] focus:outline-none disabled:opacity-50";

type SearchBarProps = {
  documents: DocumentSummary[];
  searching: boolean;
  onSearch: (request: SearchRequest) => void;
};

function parseFilterValue(raw: string): string | number | boolean {
  const value = raw.trim();

  if (value === "true") {
    return true;
  }

  if (value === "false") {
    return false;
  }

  if (/^-?\d+$/.test(value) || /^-?\d+\.\d+$/.test(value)) {
    return Number(value);
  }

  return value;
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="block">
      <span className="mb-1.5 block text-xs text-[#8d8d8d]">{label}</span>
      {children}
    </label>
  );
}

export function SearchBar({ documents, searching, onSearch }: SearchBarProps) {
  const [query, setQuery] = useState("");
  const [topK, setTopK] = useState("5");
  const [documentId, setDocumentId] = useState("");
  const [minScore, setMinScore] = useState("");
  const [filterKey, setFilterKey] = useState("");
  const [filterValue, setFilterValue] = useState("");
  const [error, setError] = useState<string | null>(null);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const trimmedQuery = query.trim();
    if (!trimmedQuery) {
      setError("Enter a query.");
      return;
    }

    const topKNumber = Number(topK);
    if (!Number.isInteger(topKNumber) || topKNumber < 1) {
      setError("Top K must be a positive whole number.");
      return;
    }

    let minScoreValue: number | null = null;
    if (minScore.trim() !== "") {
      minScoreValue = Number(minScore);
      if (Number.isNaN(minScoreValue)) {
        setError("Minimum score must be a number.");
        return;
      }
    }

    const filters: SearchRequest["filters"] = [];
    const key = filterKey.trim();
    const value = filterValue.trim();

    if (key !== "" || value !== "") {
      if (key === "" || value === "") {
        setError("Filter needs both a key and a value.");
        return;
      }

      filters.push({ key, value: parseFilterValue(value) });
    }

    setError(null);
    onSearch({
      query: trimmedQuery,
      top_k: topKNumber,
      document_id: documentId || null,
      min_score: minScoreValue,
      filters,
    });
  }

  return (
    <form onSubmit={handleSubmit} className="mt-8">
      <div className="flex flex-col gap-3 sm:flex-row">
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="What does the company do?"
          disabled={searching}
          className="w-full border border-[#2a2a2a] bg-[#0e0e0e] px-4 py-4 text-base text-[#ececec] placeholder:text-[#666] focus:border-[#555] focus:outline-none disabled:opacity-50"
        />
        <button
          type="submit"
          disabled={searching}
          className="border border-[#ececec] bg-[#ececec] px-5 py-3 text-sm text-[#090909] disabled:opacity-40 sm:self-stretch"
        >
          {searching ? "Searching..." : "Search"}
        </button>
      </div>

      <div className="mt-4 grid gap-3 sm:grid-cols-3">
        <Field label="Top K">
          <input
            type="number"
            min={1}
            step={1}
            value={topK}
            onChange={(event) => setTopK(event.target.value)}
            disabled={searching}
            className={inputClass}
          />
        </Field>
        <Field label="Document">
          <select
            value={documentId}
            onChange={(event) => setDocumentId(event.target.value)}
            disabled={searching}
            className={inputClass}
          >
            <option value="">All documents</option>
            {documents.map((document) => (
              <option key={document.document_id} value={document.document_id}>
                {document.document_id}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Minimum score">
          <input
            type="number"
            min={0}
            step="0.01"
            value={minScore}
            onChange={(event) => setMinScore(event.target.value)}
            placeholder="Any"
            disabled={searching}
            className={inputClass}
          />
        </Field>
      </div>

      <div className="mt-3 grid gap-3 sm:grid-cols-2">
        <Field label="Filter key">
          <input
            value={filterKey}
            onChange={(event) => setFilterKey(event.target.value)}
            placeholder="page"
            disabled={searching}
            className={inputClass}
          />
        </Field>
        <Field label="Filter value">
          <input
            value={filterValue}
            onChange={(event) => setFilterValue(event.target.value)}
            placeholder="1"
            disabled={searching}
            className={inputClass}
          />
        </Field>
      </div>

      {error ? <p className="mt-3 text-sm text-[#d7a0a0]">{error}</p> : null}
    </form>
  );
}
