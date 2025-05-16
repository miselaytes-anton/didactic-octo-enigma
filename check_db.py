
import sqlite3
import json

conn = sqlite3.connect('epub_documents.db')
cursor = conn.cursor()
cursor.execute('SELECT id, metadata, chapters_html FROM documents WHERE id = 1')
row = cursor.fetchone()
if row:
    id, metadata, chapters_html = row
    print(f'ID: {id}')
    html_data = json.loads(chapters_html)
    print('Chapter paths:', list(html_data.keys()))
    print('Number of chapters with HTML:', len(html_data))
conn.close()

