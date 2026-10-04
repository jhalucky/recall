import os
import threading
from contextlib import asynccontextmanager

from fastapi import FastAPI
from pydantic import BaseModel

from model import Embedder

embedder = Embedder()


@asynccontextmanager
async def lifespan(_: FastAPI):
    threading.Thread(target=embedder._load_model, daemon=True).start()
    yield


app = FastAPI(lifespan=lifespan)


class EmbedRequest(BaseModel):
    text: str


class EmbedResponse(BaseModel):
    embedding: list[float]


@app.get("/")
@app.get("/health")
def health():
    return {"status": "ok"}


@app.post("/embed", response_model=EmbedResponse)
def embed(request: EmbedRequest):
    embedding = embedder.embed(request.text)

    return EmbedResponse(embedding=embedding)


if __name__ == "__main__":
    import uvicorn

    port = int(os.environ.get("PORT", "8001"))

    uvicorn.run(app, host="0.0.0.0", port=port)
