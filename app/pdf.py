import pymupdf


def extract_pages_from_pdf(path: str) -> list[dict]:
    document = pymupdf.open(path)

    pages = []

    for page_number, page in enumerate(document, start=1):
        text = page.get_text().strip()

        if text:
            pages.append({
                "page": page_number,
                "text": text,
            })

    document.close()

    return pages