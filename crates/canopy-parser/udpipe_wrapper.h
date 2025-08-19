#ifndef UDPIPE_WRAPPER_H
#define UDPIPE_WRAPPER_H

#include "../../third-party/udpipe/src_lib_only/udpipe.h"
#include <cstring>

extern "C" {

// Sentence processing functions
typedef struct {
    int id;
    char* form;
    char* lemma;
    char* upostag;
    char* xpostag;
    char* feats;
    int head;
    char* deprel;
    char* deps;
    char* misc;
} UDPipeWord;

typedef struct {
    UDPipeWord* words;
    size_t word_count;
    char* text;
} UDPipeSentence;

// Create and destroy UDPipe sentences
UDPipeSentence* udpipe_sentence_create();
void udpipe_sentence_destroy(UDPipeSentence* sentence);

// Process text through UDPipe model
int udpipe_process_text(
    void* model_ptr,
    const char* text,
    UDPipeSentence* result,
    char** error_msg
);

// Extract words from UDPipe sentence
size_t udpipe_sentence_get_word_count(void* sentence_ptr);
int udpipe_sentence_get_word(
    void* sentence_ptr,
    size_t index,
    UDPipeWord* word
);

// String utilities
void udpipe_free_string(char* str);

} // extern "C"

#endif // UDPIPE_WRAPPER_H
