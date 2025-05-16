import sqlite3
import json
import sys

# Connect to the database
conn = sqlite3.connect('epub_documents.db')
cursor = conn.cursor()

# Get the document we want to modify
document_id = 5  # Change this if necessary
if len(sys.argv) > 1:
    document_id = int(sys.argv[1])

print(f"Looking for document with ID: {document_id}")
cursor.execute('SELECT metadata, chapters_html FROM documents WHERE id = ?', (document_id,))
row = cursor.fetchone()

if not row:
    print(f"Document with ID {document_id} not found.")
    exit(1)

metadata_json, chapters_html_json = row

# Parse the metadata to get the chapter paths
metadata = json.loads(metadata_json)
chapters = metadata.get('chapters', [])

# Create a new chapters_html object with sample HTML for each chapter
chapters_html = {}
for chapter in chapters:
    path = chapter.get('path')
    title = chapter.get('title')
    if path and title:
        # Create a simple HTML content for testing
        html_content = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>{title}</title>
        </head>
        <body>
            <h1>{title}</h1>
            <p>This is sample HTML content for chapter: {path}</p>
            <p>In a real EPUB, this would contain the actual chapter content.</p>
        </body>
        </html>
        """
        chapters_html[path] = html_content

# Update the database
cursor.execute(
    'UPDATE documents SET chapters_html = ? WHERE id = ?',
    (json.dumps(chapters_html), document_id)
)
conn.commit()

print(f"Updated document {document_id} with sample HTML content for {len(chapters_html)} chapters.")
conn.close()
