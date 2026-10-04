from sentence_transformers import SentenceTransformer


class Embedder:
    def __init__(self):
        self.model = None

    def _load_model(self):
        if self.model is None:
            self.model = SentenceTransformer("all-MiniLM-L6-v2")

    def embed(self, text: str) -> list[float]:
        self._load_model()

        embedding = self.model.encode(text)

        return embedding.tolist()
