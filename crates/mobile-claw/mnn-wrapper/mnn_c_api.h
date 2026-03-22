#ifndef MNN_C_API_H
#define MNN_C_API_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MNNInterpreter MNNInterpreter;
typedef struct MNNSession MNNSession;
typedef struct MNNTensor MNNTensor;
typedef struct MNNLlm MNNLlm;

typedef enum {
    MNN_FORWARD_CPU = 0,
    MNN_FORWARD_METAL = 1,
    MNN_FORWARD_OPENCL = 2,
    MNN_FORWARD_OPENGL = 3,
    MNN_FORWARD_VULKAN = 4,
    MNN_FORWARD_CUDA = 6,
    MNN_FORWARD_NN = 7,
    MNN_FORWARD_AUTO = 8,
} MNNForwardType;

typedef enum {
    MNN_PRECISION_NORMAL = 0,
    MNN_PRECISION_HIGH = 1,
    MNN_PRECISION_LOW = 2,
    MNN_PRECISION_LOW_BF16 = 3,
} MNNPrecision;

typedef enum {
    MNN_POWER_NORMAL = 0,
    MNN_POWER_HIGH = 1,
    MNN_POWER_LOW = 2,
} MNNPower;

typedef enum {
    MNN_MEMORY_NORMAL = 0,
    MNN_MEMORY_HIGH = 1,
    MNN_MEMORY_LOW = 2,
} MNNMemory;

typedef struct {
    int type;
    int numThread;
    const char** saveTensors;
    int saveTensorCount;
    int backupType;
    void* backendConfig;
} MNNScheduleConfig;

typedef struct {
    int precision;
    int power;
    int memory;
} MNNBackendConfig;

typedef struct {
    const int* dims;
    int dimCount;
} MNNTensorShape;

typedef struct {
    int code;
    int bits;
    int lanes;
} MNNHalideType;

MNNInterpreter* MNN_createInterpreterFromFile(const char* file);
MNNInterpreter* MNN_createInterpreterFromBuffer(const void* buffer, uint64_t size);
void MNN_destroyInterpreter(MNNInterpreter* net);

MNNSession* MNN_createSession(MNNInterpreter* net, const MNNScheduleConfig* config);
void MNN_releaseSession(MNNInterpreter* net, MNNSession* session);
int MNN_runSession(MNNInterpreter* net, MNNSession* session);

MNNTensor* MNN_getSessionInput(MNNInterpreter* net, MNNSession* session, const char* name);
MNNTensor* MNN_getSessionOutput(MNNInterpreter* net, MNNSession* session, const char* name);

MNNTensorShape MNN_getTensorShape(MNNTensor* tensor);
MNNHalideType MNN_getTensorDataType(MNNTensor* tensor);
void MNN_writeTensorToHost(MNNTensor* tensor, const void* data);
void MNN_copyTensorToHost(MNNTensor* tensor, void* data);
void* MNN_getTensorHostMap(MNNTensor* tensor);

const char* MNN_getVersion(void);

MNNLlm* Llm_createLLM(const char* config_path);
void Llm_destroy(MNNLlm* llm);
int Llm_load(MNNLlm* llm);
void Llm_response(MNNLlm* llm, const int* input_ids, int len,
                  void (*callback)(const char* text, int is_end, void* user_data),
                  void* user_data);
int Llm_tokenizer_encode(MNNLlm* llm, const char* text, int* tokens, int* len);
int Llm_tokenizer_decode(MNNLlm* llm, int token, char* buffer, int buffer_size);
int Llm_set_config(MNNLlm* llm, const char* key, const char* value);
void Llm_reset(MNNLlm* llm);

#ifdef __cplusplus
}
#endif

#endif
