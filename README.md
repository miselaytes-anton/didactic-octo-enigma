curl -X POST http://127.0.0.1:8081/upload -F "file=@/Users/anton/www/epub-server/rust-web-server/test-3.epub" | jq

curl  http://127.0.0.1:8081/document/1/chapter/1/audio

curl  http://127.0.0.1:8081/document/1/chapter/1
