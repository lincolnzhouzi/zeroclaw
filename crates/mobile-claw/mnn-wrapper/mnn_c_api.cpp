#include "mnn_c_api.h"
#include <MNN/Interpreter.hpp>
#include <MNN/expr/Expr.hpp>
#include <MNN/expr/Module.hpp>
#include <MNN/Tensor.hpp>
#include <cstring>
#include <string>
#include <vector>
#include <memory>

#ifdef MNN_BUILD_LLM
#include <llm/llm.hpp>
#endif

extern "C" {

struct MNNInterpreter {
    std::unique_ptr<MNN::Interpreter> interpreter;
};

struct MNNSession {
    MNN::Session* session;
};

struct MNNTensor {
    MNN::Tensor* tensor;
    bool ownsData;
};

MNNInterpreter* MNN_createInterpreterFromFile(const char* file) {
    auto* wrapper = new MNNInterpreter();
    wrapper->interpreter.reset(MNN::Interpreter::createFromFile(file));
    if (!wrapper->interpreter) {
        delete wrapper;
        return nullptr;
    }
    return wrapper;
}

MNNInterpreter* MNN_createInterpreterFromBuffer(const void* buffer, uint64_t size) {
    auto* wrapper = new MNNInterpreter();
    wrapper->interpreter.reset(MNN::Interpreter::createFromBuffer(buffer, size));
    if (!wrapper->interpreter) {
        delete wrapper;
        return nullptr;
    }
    return wrapper;
}

void MNN_destroyInterpreter(MNNInterpreter* net) {
    if (net) {
        delete net;
    }
}

MNNSession* MNN_createSession(MNNInterpreter* net, const MNNScheduleConfig* config) {
    if (!net || !net->interpreter || !config) {
        return nullptr;
    }
    
    MNN::ScheduleConfig mnnConfig;
    mnnConfig.type = static_cast<MNNForwardType>(config->type);
    mnnConfig.numThread = config->numThread;
    
    if (config->saveTensors && config->saveTensorCount > 0) {
        for (int i = 0; i < config->saveTensorCount; ++i) {
            if (config->saveTensors[i]) {
                mnnConfig.saveTensors.push_back(config->saveTensors[i]);
            }
        }
    }
    
    mnnConfig.backendConfig = nullptr;
    
    auto* sessionWrapper = new MNNSession();
    sessionWrapper->session = net->interpreter->createSession(mnnConfig);
    if (!sessionWrapper->session) {
        delete sessionWrapper;
        return nullptr;
    }
    return sessionWrapper;
}

void MNN_releaseSession(MNNInterpreter* net, MNNSession* session) {
    if (net && net->interpreter && session && session->session) {
        net->interpreter->releaseSession(session->session);
        delete session;
    }
}

int MNN_runSession(MNNInterpreter* net, MNNSession* session) {
    if (!net || !net->interpreter || !session || !session->session) {
        return -1;
    }
    return net->interpreter->runSession(session->session);
}

MNNTensor* MNN_getSessionInput(MNNInterpreter* net, MNNSession* session, const char* name) {
    if (!net || !net->interpreter || !session || !session->session) {
        return nullptr;
    }
    auto* tensor = net->interpreter->getSessionInput(session->session, name);
    if (!tensor) {
        return nullptr;
    }
    auto* wrapper = new MNNTensor();
    wrapper->tensor = tensor;
    wrapper->ownsData = false;
    return wrapper;
}

MNNTensor* MNN_getSessionOutput(MNNInterpreter* net, MNNSession* session, const char* name) {
    if (!net || !net->interpreter || !session || !session->session) {
        return nullptr;
    }
    auto* tensor = net->interpreter->getSessionOutput(session->session, name);
    if (!tensor) {
        return nullptr;
    }
    auto* wrapper = new MNNTensor();
    wrapper->tensor = tensor;
    wrapper->ownsData = false;
    return wrapper;
}

MNNTensorShape MNN_getTensorShape(MNNTensor* tensor) {
    MNNTensorShape shape = {nullptr, 0};
    if (!tensor || !tensor->tensor) {
        return shape;
    }
    
    auto* mnnTensor = tensor->tensor;
    shape.dimCount = static_cast<int>(mnnTensor->dimensions());
    
    static thread_local std::vector<int> dimsBuffer;
    dimsBuffer.resize(shape.dimCount);
    for (int i = 0; i < shape.dimCount; ++i) {
        dimsBuffer[i] = mnnTensor->length(i);
    }
    shape.dims = dimsBuffer.data();
    
    return shape;
}

MNNHalideType MNN_getTensorDataType(MNNTensor* tensor) {
    MNNHalideType type = {0, 0, 1};
    if (!tensor || !tensor->tensor) {
        return type;
    }
    
    auto& t = tensor->tensor->getType();
    type.code = t.code;
    type.bits = t.bits;
    type.lanes = t.lanes;
    
    return type;
}

void MNN_writeTensorToHost(MNNTensor* tensor, const void* data) {
    if (!tensor || !tensor->tensor || !data) {
        return;
    }
    tensor->tensor->copyFromHostFloat(static_cast<const float*>(data));
}

void MNN_copyTensorToHost(MNNTensor* tensor, void* data) {
    if (!tensor || !tensor->tensor || !data) {
        return;
    }
    tensor->tensor->copyToHostFloat(static_cast<float*>(data));
}

void* MNN_getTensorHostMap(MNNTensor* tensor) {
    if (!tensor || !tensor->tensor) {
        return nullptr;
    }
    return tensor->tensor->host<void>();
}

const char* MNN_getVersion(void) {
    static std::string version;
    version = MNN::getVersion();
    return version.c_str();
}

}

#ifdef MNN_BUILD_LLM

struct MNNLlm {
    std::shared_ptr<MNN::Transformer::Llm> llm;
    std::string lastError;
};

MNNLlm* Llm_createLLM(const char* config_path) {
    if (!config_path) {
        return nullptr;
    }
    
    auto* wrapper = new MNNLlm();
    try {
        wrapper->llm.reset(MNN::Transformer::Llm::createLLM(config_path));
        if (!wrapper->llm) {
            delete wrapper;
            return nullptr;
        }
    } catch (const std::exception& e) {
        wrapper->lastError = e.what();
        delete wrapper;
        return nullptr;
    }
    return wrapper;
}

void Llm_destroy(MNNLlm* llm) {
    if (llm) {
        if (llm->llm) {
            MNN::Transformer::Llm::destroy(llm->llm.get());
        }
        delete llm;
    }
}

int Llm_load(MNNLlm* llm) {
    if (!llm || !llm->llm) {
        return -1;
    }
    try {
        return llm->llm->load() ? 0 : -1;
    } catch (const std::exception& e) {
        llm->lastError = e.what();
        return -1;
    }
}

void Llm_response(MNNLlm* llm, const int* input_ids, int len,
                  void (*callback)(const char* text, int is_end, void* user_data),
                  void* user_data) {
    if (!llm || !llm->llm || !input_ids || !callback) {
        return;
    }
    
    try {
        std::vector<int> tokens(input_ids, input_ids + len);
        
        llm->llm->response(tokens, [callback, user_data](const char* text, int is_end) {
            callback(text, is_end, user_data);
        });
    } catch (const std::exception& e) {
        llm->lastError = e.what();
        callback("", -1, user_data);
    }
}

int Llm_tokenizer_encode(MNNLlm* llm, const char* text, int* tokens, int* len) {
    if (!llm || !llm->llm || !text || !tokens || !len) {
        return -1;
    }
    
    try {
        auto result = llm->llm->tokenizer_encode(text);
        int copyLen = std::min(*len, static_cast<int>(result.size()));
        for (int i = 0; i < copyLen; ++i) {
            tokens[i] = result[i];
        }
        *len = static_cast<int>(result.size());
        return 0;
    } catch (const std::exception& e) {
        llm->lastError = e.what();
        return -1;
    }
}

int Llm_tokenizer_decode(MNNLlm* llm, int token, char* buffer, int buffer_size) {
    if (!llm || !llm->llm || !buffer || buffer_size <= 0) {
        return -1;
    }
    
    try {
        std::string result = llm->llm->tokenizer_decode(token);
        int copyLen = std::min(static_cast<int>(result.size()), buffer_size - 1);
        std::memcpy(buffer, result.c_str(), copyLen);
        buffer[copyLen] = '\0';
        return 0;
    } catch (const std::exception& e) {
        llm->lastError = e.what();
        return -1;
    }
}

int Llm_set_config(MNNLlm* llm, const char* key, const char* value) {
    if (!llm || !llm->llm || !key || !value) {
        return -1;
    }
    
    try {
        std::string config = "{\"" + std::string(key) + "\":\"" + std::string(value) + "\"}";
        return llm->llm->set_config(config) ? 0 : -1;
    } catch (const std::exception& e) {
        llm->lastError = e.what();
        return -1;
    }
}

void Llm_reset(MNNLlm* llm) {
    if (llm && llm->llm) {
        llm->llm->reset();
    }
}

#endif
