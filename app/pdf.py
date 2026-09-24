import fitz


def extract_text_from_pdf(path: str) -> str:
    document = fitz.open(path)

    pages = []

    for page in document:
        text = page.get_text()
        if text.strip():
            pages.append(text)

    document.close()

    return "\n\n".join(pages)