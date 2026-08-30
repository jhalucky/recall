from fastapi import FastAPI
from pydantic import BaseModel

from model import Embedder

app = FastAPI()
embedder = Embedder()

class EmbedRequest(BaseModel):
    text: str

class EmbedResponse(BaseModel):
    embedding: list[float]

@app.post("/embed", response_model=EmbedResponse)
def embed(request: EmbedRequest):
    embedding = embedder.embed(request.text)

    return EmbedResponse(embedding=embedding)
