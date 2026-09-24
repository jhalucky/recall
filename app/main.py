from fastapi import FastAPI
from pydantic import BaseModel
from pathlib import Path
from tempfile import NamedTemporaryFile

from fastapi import File, UploadFile

from .pdf import extract_text_from_pdf
from .recall_client import RecallClient


app = FastAPI(title="RECALL Application")

recall = RecallClient()


class DocumentRequest(BaseModel):
    id: str
    text: str
    metadata: dict = {}
    chunk_size: int = 100


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
        text = extract_text_from_pdf(temp_path)

        document_id = Path(file.filename or "document.pdf").stem

        result = recall.add_document(
            document_id=document_id,
            text=text,
            metadata={
                "source": file.filename or "document.pdf",
                "type": "pdf",
            },
            chunk_size=100,
        )

        return result

    finally:
        Path(temp_path).unlink(missing_ok=True)