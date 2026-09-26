"use client";

import { useEffect, useState } from "react";
import { SearchBar } from "@/components/SearchBar";
import { SearchResult } from "@/components/SearchResult";
import {
  listDocuments,
  searchDocuments,
  type DocumentSummary,
  type SearchRequest,
  type SearchResult as SearchHit,
} from "@/lib/api";

export default function SearchPage() {
  const [documents, setDocuments] = useState<DocumentSummary[]>([]);
  const [documentsError, setDocumentsError] = useState<string | null>(null);
  const [searching, setSearching] = useState(false);
  const [searched, setSearched] = useState(false);
  const [results, setResults] = useState<SearchHit[]>([]);
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
          setDocumentsError(err instanceof Error ? err.message : "Could not load documents.");
        }
      });

    return () => {
      cancelled = true;
    };
  }, []);

  async function handleSearch(request: SearchRequest) {
    setSearching(true);
    setError(null);

    try {
      const response = await searchDocuments(request);
      setResults(response.results);
      setSearched(true);
    } catch (err: unknown) {
      setResults([]);
      setSearched(false);
      setError(err instanceof Error ? err.message : "Search failed.");
    } finally {
      setSearching(false);
    }
  }

  return (
    <section>
      <h1 className="text-lg text-[#f5f5f5]">Search</h1>
      <p className="mt-1 text-sm text-[#8d8d8d]">Semantic search across indexed documents.</p>
      <SearchBar documents={documents} searching={searching} onSearch={handleSearch} />
      {documentsError ? <p className="mt-3 text-sm text-[#d7a0a0]">{documentsError}</p> : null}
      {error ? <p className="mt-4 text-sm text-[#d7a0a0]">{error}</p> : null}
      {searched ? (
        <div className="mt-8">
          <p className="text-xs text-[#6f6f6f]">
            {results.length} {results.length === 1 ? "result" : "results"}
          </p>
          <div className="mt-3">
            {results.map((result) => (
              <SearchResult key={result.chunk_id} result={result} />
            ))}
          </div>
        </div>
      ) : null}
    </section>
  );
}
