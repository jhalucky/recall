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