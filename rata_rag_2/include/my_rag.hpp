#pragma once
#include <iostream>
#include <fstream>
#include <filesystem>
#include <vector>
#include <string>
#include <sstream>
#include <stdexcept>
#include <map>
#include <curl/curl.h>
#include <nlohmann/json.hpp>

#include "qdrant_client.hpp"
#include "http_client.hpp"
#include "GeminiEmbeddingClient.hpp"
#include "my_config.hpp"

using json = nlohmann::json;

const std::string COLLECTION = "document-1";
const std::string API_URL_CHAT = "http://localhost:8090/v1/chat/completions";

struct ChatQuery {
    std::string role;
    std::string content;
};
// これ一行で、QueryReq <=> json の変換が魔法のように可能になります
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(ChatQuery, role, content)

struct ChatRequest {
    std::string model;
    std::vector<ChatQuery> messages;
    double temperature;
};
NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE(ChatRequest, model, messages, temperature)

// 1ファイル分のデータを保持する構造体
struct TextFile {
    std::string filename;
    std::vector<std::string> lines;
};

class MyRag {
private:
    std::string m_name;

    public:
    explicit MyRag(std::string str){}

    ~MyRag() {}

    std::string extractContent(const std::string& jsonStr)
    {
        try {
            auto j = nlohmann::json::parse(jsonStr);
            return j["choices"][0]["message"]["content"].get<std::string>();
        } catch (const std::exception& e) {
            std::cerr << "[ERROR] JSON parse: " << e.what() << "\n";
            return "";
        }
    }
    /**
    *
    * @param
    *
    * @return
    */
    std::string rag_search_handler(std::string query){
        try{
            std::string ret = "";

            auto embeddings = EmbeddingStart(query);
            //std::cout << "vlen=" << embeddings.size() << std::endl;
            auto vec = embeddings;

            QdrantClient qdrant_client("localhost", 6333);
            auto results = qdrant_client.search(COLLECTION, embeddings, 1);
            std::string matches = "";
            for (const auto& r : results) {
                //std::cout << "  score: " << std::fixed << std::setprecision(4) << r.score
                //        << "\n";
                //std::cout << "r.score=" << r.score << "\n";
                std::string content = r.payload["content"].get<std::string>();
                if(r.score > 0.6) {
                    matches = content;
                }
            }
            //std::cout << "\n";

            std::string out_str = "要約して欲しい。\n";
            std::string resp_str = matches;
            if(resp_str.empty()){
                out_str.append("user query: ");
                out_str.append(query);
                out_str.append(" \n");
            }else{
                out_str.append("context:");
                out_str.append(resp_str);
                out_str.append("\n user query: ");
                out_str.append(query);
                out_str.append(" \n");
            }
            ChatQuery req2;
            req2.role = "user";
            req2.content = out_str;
            json j2 = req2;
            std::string json_str2 = j2.dump();
            //std::cout << "json_str2:" << json_str2 << std::endl;
            std::vector<ChatQuery> chat_messages;
            chat_messages.push_back(req2);

            std::string target_msg = "[";
            target_msg.append(json_str2);
            target_msg.append("]");
            ChatRequest req3;
            req3.model = "local-model";
            req3.messages = chat_messages;
            req3.temperature = 0.7;
            json j3 = req3; // 構造体を代入するだけ！
            std::string json_str3 = j3.dump();
            //std::cout << "json_str3:" << json_str3 << std::endl;
            std::string requestBody = json_str3;

            //std::cout << "requestBody:" << requestBody  << std::endl;
            HttpClient client(30 /*timeout*/, true /*verify_ssl*/);
            auto resp2 = client.post_json(API_URL_CHAT, requestBody);
            if (!resp2.error.empty()) {
                std::cerr << "[ERROR] " << resp2.error << "\n";
                return 0;
            }
            std::string reply = extractContent(resp2.body);
            return reply;
        } catch (const std::exception& e) {
            std::cout << "Error , main" << std::endl;
        }  
    }   

};
