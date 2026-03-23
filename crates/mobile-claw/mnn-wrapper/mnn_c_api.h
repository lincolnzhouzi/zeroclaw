#ifndef MNN_C_API_H
#define MNN_C_API_H

#include <stdint.h>

#ifdef MNN_WRAPPER_EXPORTS
#ifdef _WIN32
#define MNN_WRAPPER_API __declspec(dllexport)
#else
#define MNN_WRAPPER_API __attribute__((visibility("default")))
#endif
#else
#ifdef _WIN32
#define MNN_WRAPPER_API __declspec(dllimport)
#else
#define MNN_WRAPPER_API
#endif
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct MNNInterpreter MNNInterpreter;
typedef struct MNNSession MNNSession;
typedef struct MNNTensor MNNTensor;
typedef struct MNNLlm MNNLlm;

typedef struct {
  int type;
  int numThread;
  const char **saveTensors;
  int saveTensorCount;
  int backupType;
  void *backendConfig;
} MNNScheduleConfig;

typedef struct {
  int precision;
  int power;
  int memory;
} MNNBackendConfig;

typedef struct {
  const int *dims;
  int dimCount;
} MNNTensorShape;

typedef struct {
  int code;
  int bits;
  int lanes;
} MNNHalideType;

MNN_WRAPPER_API MNNInterpreter *MNN_createInterpreterFromFile(const char *file);
MNN_WRAPPER_API MNNInterpreter *
MNN_createInterpreterFromBuffer(const void *buffer, uint64_t size);
MNN_WRAPPER_API void MNN_destroyInterpreter(MNNInterpreter *net);

MNN_WRAPPER_API MNNSession *MNN_createSession(MNNInterpreter *net,
                                              const MNNScheduleConfig *config);
MNN_WRAPPER_API void MNN_releaseSession(MNNInterpreter *net,
                                        MNNSession *session);
MNN_WRAPPER_API int MNN_runSession(MNNInterpreter *net, MNNSession *session);

MNN_WRAPPER_API MNNTensor *
MNN_getSessionInput(MNNInterpreter *net, MNNSession *session, const char *name);
MNN_WRAPPER_API MNNTensor *MNN_getSessionOutput(MNNInterpreter *net,
                                                MNNSession *session,
                                                const char *name);

MNN_WRAPPER_API MNNTensorShape MNN_getTensorShape(MNNTensor *tensor);
MNN_WRAPPER_API MNNHalideType MNN_getTensorDataType(MNNTensor *tensor);
MNN_WRAPPER_API void MNN_writeTensorToHost(MNNTensor *tensor, const void *data);
MNN_WRAPPER_API void MNN_copyTensorToHost(MNNTensor *tensor, void *data);
MNN_WRAPPER_API void *MNN_getTensorHostMap(MNNTensor *tensor);

MNN_WRAPPER_API const char *MNN_getVersion(void);

MNN_WRAPPER_API MNNLlm *Llm_createLLM(const char *config_path);
MNN_WRAPPER_API void Llm_destroy(MNNLlm *llm);
MNN_WRAPPER_API int Llm_load(MNNLlm *llm);
MNN_WRAPPER_API void Llm_response(MNNLlm *llm, const int *input_ids, int len,
                                  void (*callback)(const char *text, int is_end,
                                                   void *user_data),
                                  void *user_data);
MNN_WRAPPER_API int Llm_tokenizer_encode(MNNLlm *llm, const char *text,
                                         int *tokens, int *len);
MNN_WRAPPER_API int Llm_tokenizer_decode(MNNLlm *llm, int token, char *buffer,
                                         int buffer_size);
MNN_WRAPPER_API int Llm_set_config(MNNLlm *llm, const char *key,
                                   const char *value);
MNN_WRAPPER_API void Llm_reset(MNNLlm *llm);

MNN_WRAPPER_API const char *Llm_getLastError(MNNLlm *llm);
MNN_WRAPPER_API const char *Llm_getGeneratedText(MNNLlm *llm);
MNN_WRAPPER_API int Llm_generate(MNNLlm *llm, const char *prompt,
                                 int max_new_tokens);

#ifdef __cplusplus
}
#endif

#endif
