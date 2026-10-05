class Embedder:
    def __init__(self):
        self.model = None

    def _load_model(self):
        if self.model is None:
            from fastembed import TextEmbedding

            self.model = TextEmbedding(
                model_name="sentence-transformers/all-MiniLM-L6-v2"
            )

    def embed(self, text: str) -> list[float]:
        self._load_model()
        embedding = next(self.model.embed(text))
        return embedding.tolist()
