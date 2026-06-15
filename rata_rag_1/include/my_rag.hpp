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

#include "GeminiEmbeddingClient.hpp"
#include "db_add.hpp"
#include "db_search.hpp"
#include "my_config.hpp"

using json = nlohmann::json;

const std::string DB_PATH = "example.db";
const std::string DATA_PATH = "./data";

// 1ファイル分のデータを保持する構造体
struct TextFile {
    std::string filename;
    std::vector<std::string> lines;
};

// .txt ファイルを読み込んで行を返す
TextFile loadTextFile(const std::filesystem::path& filepath) {
    TextFile tf;
    tf.filename = filepath.filename().string();

    std::ifstream ifs(filepath);
    if (!ifs.is_open()) {
        std::cerr << "[警告] ファイルを開けません: " << filepath << "\n";
        return tf;
    }

    std::string line;
    while (std::getline(ifs, line)) {
        tf.lines.push_back(line);
    }
    return tf;
}

class MyRag {
private:
    std::string m_name;

    public:
    explicit MyRag(std::string str){}

    ~MyRag() {}

    /**
    *
    * @param
    *
    * @return
    */
    int ebmed(std::string query){
        int ret = 0;

        auto embeddings = EmbeddingStart(query);
        std::cout << "vlen=" << embeddings.size() << std::endl;
        try{

            auto vec = embeddings;

            double v1 = vec[0];
            double v2 = vec[1];        
            std::cout << v1 << ", " << v2 << std::endl;

            DbAdd app(DB_PATH);
            app.add_embed(vec, query);

        } catch (const std::exception& e) {
            std::cout << "Error , main" << std::endl;
            return 1;
        }        
        
        return 0;
    }    

    // 読み込んだデータを表示する
    void addTextFiles(const std::vector<TextFile>& files) {
        for (const auto& tf : files) {
            std::cout << "========================================\n";
            std::cout << "ファイル名: " << tf.filename << "\n";
            std::cout << "行数      : " << tf.lines.size() << "\n";
            std::cout << "----------------------------------------\n";
            std:string target = "";
            for (size_t i = 0; i < tf.lines.size(); ++i) {
                //std::cout << "[" << i + 1 << "] " << tf.lines[i] << "\n";
                std::string tmp = tf.lines[i] + "\n";
                target.append(tmp);
            }
            std::cout <<  target << "\n";
            int resp = ebmed(target);
            std::cout << "resp=" << resp << "\n";
        }
        std::cout << "========================================\n";
    }

    std::string rag_search_handler(std::string query){
        try{
            std::string ret = "";
            std::string dirPath = DATA_PATH;

            auto embeddings = EmbeddingStart(query);
            //std::cout << "vlen=" << embeddings.size() << std::endl;
            auto vec = embeddings;
            DbSearch app(DB_PATH);
            if(app.search_embed_size(vec) == false){
                return ret;
            }

            std::string resp_str = app.rag_search(vec);
            
            std::string out_str = "日本語で、回答して欲しい。 \n要約して欲しい。\n\n";
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
            //std::cout << out_str  << std::endl; 
            ret = app.send_chat(out_str);  

            return ret;
        } catch (const std::exception& e) {
            std::cout << "Error , main" << std::endl;
        }  
    }

    void rag_add_handler(){
        try{
            std::string dirPath = DATA_PATH;
            if (!std::filesystem::exists(dirPath) || !std::filesystem::is_directory(dirPath)) {
                std::cerr << "[エラー] 有効なディレクトリではありません: " << dirPath << "\n";
                return;
            }        
            std::cout << "対象フォルダ: " << std::filesystem::absolute(dirPath) << "\n\n";
            std::vector<TextFile> allFiles;

            // フォルダ内の .txt ファイルをすべて列挙
            for (const auto& entry : std::filesystem::directory_iterator(dirPath)) {
                if (entry.is_regular_file() && 
                (entry.path().extension() == ".txt" || entry.path().extension() == ".md") ) {
                    TextFile tf = loadTextFile(entry.path());
                    allFiles.push_back(std::move(tf));
                }
            }
            if (allFiles.empty()) {
                std::cout << ".txt ファイルが見つかりませんでした。\n";
                return;
            }
            std::cout << "読み込んだファイル数: " << allFiles.size() << "\n\n";
            addTextFiles(allFiles);

        } catch (const std::exception& e) {
            std::cout << "Error , main" << std::endl;
        }  
    }    

};
