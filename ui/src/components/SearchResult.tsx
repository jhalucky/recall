import { Fragment } from "react";
import type { SearchResult as SearchHit } from "@/lib/api";
import { metadataRows, pageFromMetadata } from "@/lib/metadata";

export function SearchResult({ result }: { result: SearchHit }) {
  const page = pageFromMetadata(result.metadata);
  const rows = metadataRows(result.metadata);

  return (
    <article className="border-t border-[#242424] py-5">
      <div className="flex flex-wrap items-baseline gap-x-4 gap-y-1 text-xs">
        <span className="text-sm text-[#c6e08a]">{result.score.toFixed(3)}</span>
        <span className="text-[#e4e4e4]">{result.document_id}</span>
        {page ? <span className="text-[#8d8d8d]">page {page}</span> : null}
      </div>
      <div className="mt-1 break-all text-xs text-[#6f6f6f]">{result.chunk_id}</div>
      <p className="mt-3 whitespace-pre-wrap text-sm leading-6 text-[#e4e4e4]">{result.text}</p>
      {rows.length > 0 ? (
        <dl className="mt-4 grid grid-cols-[max-content_1fr] gap-x-4 gap-y-1 text-xs">
          {rows.map((row) => (
            <Fragment key={row.key}>
              <dt className="text-[#6f6f6f]">{row.key}</dt>
              <dd className="break-all text-[#c4c4c4]">{row.value}</dd>
            </Fragment>
          ))}
        </dl>
      ) : null}
    </article>
  );
}
