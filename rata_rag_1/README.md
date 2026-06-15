# rata_rag_1

 Version: 0.9.1

 date    : 2026/06/14 

 update :

***

Rust C++ , Ratatui RAG, SQLite

* embedding : Gemini-embedding-001
* model: gemma-4-E2B
* llama.cpp , llama-server
* LLVM CLang

***
### vector data add

https://github.com/kuc-arc-f/rust_cpp1/tree/main/rs_rag_1

***
## image

* RAG Search

![img1](/images/rata_rag_1.png)


***
* llama-server start
* port 8090: gemma-4-E2B

```
#gemma-4-E2B

/usr/local/llama-b8642/llama-server -m /var/lm_data/unsloth/gemma-4-E2B-it-Q4_K_S.gguf \
 --chat-template-kwargs '{"enable_thinking": false}' --port 8090 
```
***
### related

https://huggingface.co/unsloth/gemma-4-E2B-it-GGUF

***
* LIB add
```
sudo apt install uuid-dev
sudo apt install nlohmann-json3-dev
sudo apt install libsqlite3-dev
sudo apt install libcurl4-openssl-dev
```
***
* rs_rag_1/example.db db file copy
* example.db

***
* env value

```
export LD_LIBRARY_PATH=.
export GEMINI_API_KEY=
```

***
* build
```
make all
cargo build
cargo run
```

***
* UI operate
* edit mode: e key
* quit: Esc key , q

***
### blog

https://zenn.dev/knaka0209/scraps/8130c03dd1d728

***
