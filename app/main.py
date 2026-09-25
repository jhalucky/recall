from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field
from pathlib import Path
from tempfile import NamedTemporaryFile

from fastapi import File, UploadFile
from .pdf import extract_pages_from_pdf

from .recall_client import RecallClient


app = FastAPI(title="RECALL Application")

recall = RecallClient()


class DocumentRequest(BaseModel):
    id: str
    text: str
    metadata: dict = Field(default_factory=dict)
    chunk_size: int = Field(default=100, ge=1)


class MetadataFilterRequest(BaseModel):
    key: str
    value: str | int | float | bool


class SearchRequest(BaseModel):
    query: str
    top_k: int = 3
    document_id: str | None = None
    min_score: float | None = None
    ilters: list[MetadataFilterRequest] = Field(default_factory=list)


@app.get("/health")
def health():
    return {"status": "ok"}


@app.post("/documents")
def add_document(request: DocumentRequest):
    return recall.add_document(
        document_id=request.id,
        text=request.text,
        metadata=request.metadata,
        chunk_size=request.chunk_size,
    )


@app.post("/documents/pdf")
async def add_pdf(file: UploadFile = File(...)):
    if file.content_type != "application/pdf":
        return {"error": "Only PDF files are supported"}

    contents = await file.read()

    with NamedTemporaryFile(suffix=".pdf", delete=False) as temp:
        temp.write(contents)
        temp_path = temp.name

    try:
        pages = extract_pages_from_pdf(temp_path)

        document_id = Path(file.filename or "document.pdf").stem

        return recall.add_document_pages(
            document_id=document_id,
            pages=pages,
            metadata={
                "source": file.filename or "document.pdf",
                "type": "pdf",
            },
            chunk_size=100,
        )

    finally:
        Path(temp_path).unlink(missing_ok=True)

@app.get("/documents")
def list_documents():
    return recall.list_documents()


@app.post("/search")
def search(request: SearchRequest):
    return recall.search(
        query=request.query,
        top_k=request.top_k,
        document_id=request.document_id,
        min_score=request.min_score,
        filters=[
            {"key": filter.key, "value": filter.value}
            for filter in request.filters
        ],
    )


@app.delete("/documents/{document_id}")
def delete_document(document_id: str):
    return recall.delete_document(document_id)