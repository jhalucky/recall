import httpx

class RecallClient:
    def __init__(self, base_url: str = "http://127.0.0.1:3000"):
        self.base_url = base_url.rstrip("/")

    def add_document(
            self,
            document_id: str,
            text: str,
            metadata: dict | None=None,
            chunk_size: int = 100
    ) -> dict:
        payload = {
            "id":document_id,
            "text":text,
            "metadata":metadata or {},
            "chunk_size": chunk_size
        }

        response = httpx.post(
            f"{self.base_url}/documents",
            json=payload,
            timeout=120.0
        )

        response.raise_for_status()

        return response.json()

    def add_document_pages(
        self,
        document_id: str,
        pages: list[dict],
        metadata: dict | None = None,
        chunk_size: int = 100,
    ) -> dict:
        payload = {
            "id": document_id,
            "pages": pages,
            "metadata": metadata or {},
            "chunk_size": chunk_size,
        }

        response = httpx.post(
            f"{self.base_url}/documents/pages",
            json=payload,
            timeout=120.0,
        )

        response.raise_for_status()

        return response.json()

    def list_documents(self) -> dict:
        response = httpx.get(
            f"{self.base_url}/documents",
            timeout=30.0,
        )

        response.raise_for_status()

        return response.json()


    def search(
        self,
        query: str,
        top_k: int = 3,
        document_id: str | None = None,
        min_score: float | None = None,
        filters: list[dict] | None = None,
    ) -> dict:
        payload = {
            "query": query,
            "top_k": top_k,
            "document_id": document_id,
            "min_score": min_score,
            "filters": filters or [],
        }

        response = httpx.post(
            f"{self.base_url}/search",
            json=payload,
            timeout=120.0,
        )

        response.raise_for_status()

        return response.json()


    def delete_document(self, document_id: str) -> dict:
        response = httpx.delete(
            f"{self.base_url}/documents/{document_id}",
            timeout=30.0,
        )

        response.raise_for_status()

        return response.json()