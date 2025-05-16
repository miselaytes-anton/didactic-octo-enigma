import sqlite3
import json
import sys

conn = sqlite3.connect('epub_documents.db')
cursor = conn.cursor()

# Get command line argument for document ID, default to 4
doc_id = 4
if len(sys.argv) > 1:
    doc_id = int(sys.argv[1])

# Query the document
cursor.execute('SELECT id, metadata, chapters_html FROM documents WHERE id = ?', (doc_id,))
row = cursor.fetchone()

if row:
    id, metadata_json, chapters_html_json = row
    
    print(f"Document ID: {id}")
    
    # Parse JSON
    try:
        metadata = json.loads(metadata_json)
        print(f"Metadata: {json.dumps(metadata, indent=2)[:200]}...")
    except json.JSONDecodeError as e:
        print(f"Failed to parse metadata JSON: {e}")
    
    # Check if chapters_html is empty
    try:
        chapters_html = json.loads(chapters_html_json)
        
        if not chapters_html:
            print("chapters_html is empty!")
        else:
            print(f"chapters_html contains {len(chapters_html)} entries")
            # Print first chapter key and a sample of content
            first_key = list(chapters_html.keys())[0]
            print(f"First chapter key: {first_key}")
            print(f"Sample content: {chapters_html[first_key][:100]}...")
    except json.JSONDecodeError as e:
        print(f"Failed to parse chapters_html JSON: {e}")
else:
    print(f"No document found with ID {doc_id}")

conn.close()
