#include "udpipe_wrapper.h"
#include <sstream>
#include <memory>

using namespace ufal::udpipe;

extern "C" {

// Helper function to duplicate a string for C compatibility
char* duplicate_string(const std::string& str) {
    if (str.empty()) {
        return nullptr;
    }
    char* result = (char*)malloc(str.length() + 1);
    if (result) {
        strcpy(result, str.c_str());
    }
    return result;
}

UDPipeSentence* udpipe_sentence_create() {
    UDPipeSentence* result = (UDPipeSentence*)malloc(sizeof(UDPipeSentence));
    if (result) {
        result->words = nullptr;
        result->word_count = 0;
        result->text = nullptr;
    }
    return result;
}

void udpipe_sentence_destroy(UDPipeSentence* sentence) {
    if (!sentence) return;

    // Free all word data
    for (size_t i = 0; i < sentence->word_count; i++) {
        UDPipeWord* word = &sentence->words[i];
        free(word->form);
        free(word->lemma);
        free(word->upostag);
        free(word->xpostag);
        free(word->feats);
        free(word->deprel);
        free(word->deps);
        free(word->misc);
    }

    free(sentence->words);
    free(sentence->text);
    free(sentence);
}

int udpipe_process_text(
    void* model_ptr,
    const char* text,
    UDPipeSentence* result,
    char** error_msg
) {
    if (!model_ptr || !text || !result) {
        if (error_msg) {
            *error_msg = duplicate_string("Invalid parameters");
        }
        return 0;
    }

    try {
        model* udpipe_model = static_cast<model*>(model_ptr);

        // Create tokenizer
        std::unique_ptr<input_format> tokenizer(
            udpipe_model->new_tokenizer(model::TOKENIZER_NORMALIZED_SPACES)
        );

        if (!tokenizer) {
            if (error_msg) {
                *error_msg = duplicate_string("Failed to create tokenizer");
            }
            return 0;
        }

        // Set text for processing
        tokenizer->set_text(text);

        // Process sentences
        sentence s;
        std::string error;

        if (!tokenizer->next_sentence(s, error)) {
            if (error_msg) {
                *error_msg = duplicate_string(error.empty() ? "No sentence found" : error.c_str());
            }
            return 0;
        }

        // Apply POS tagging
        if (!udpipe_model->tag(s, model::DEFAULT, error)) {
            if (error_msg) {
                *error_msg = duplicate_string(error.empty() ? "POS tagging failed" : error.c_str());
            }
            return 0;
        }

        // Apply dependency parsing
        if (!udpipe_model->parse(s, model::DEFAULT, error)) {
            if (error_msg) {
                *error_msg = duplicate_string(error.empty() ? "Dependency parsing failed" : error.c_str());
            }
            return 0;
        }

        // Extract results
        result->word_count = s.words.size();
        if (result->word_count > 0) {
            result->words = (UDPipeWord*)malloc(sizeof(UDPipeWord) * result->word_count);
            if (!result->words) {
                if (error_msg) {
                    *error_msg = duplicate_string("Memory allocation failed");
                }
                return 0;
            }

            for (size_t i = 0; i < result->word_count; i++) {
                const word& w = s.words[i];
                UDPipeWord* dest = &result->words[i];

                dest->id = w.id;
                dest->form = duplicate_string(w.form);
                dest->lemma = duplicate_string(w.lemma);
                dest->upostag = duplicate_string(w.upostag);
                dest->xpostag = duplicate_string(w.xpostag);
                dest->feats = duplicate_string(w.feats);
                dest->head = w.head;
                dest->deprel = duplicate_string(w.deprel);
                dest->deps = duplicate_string(w.deps);
                dest->misc = duplicate_string(w.misc);
            }
        }

        result->text = duplicate_string(text);

        return 1; // Success

    } catch (const std::exception& e) {
        if (error_msg) {
            *error_msg = duplicate_string(e.what());
        }
        return 0;
    } catch (...) {
        if (error_msg) {
            *error_msg = duplicate_string("Unknown error occurred");
        }
        return 0;
    }
}

size_t udpipe_sentence_get_word_count(void* sentence_ptr) {
    if (!sentence_ptr) return 0;

    sentence* s = static_cast<sentence*>(sentence_ptr);
    return s->words.size();
}

int udpipe_sentence_get_word(
    void* sentence_ptr,
    size_t index,
    UDPipeWord* word
) {
    if (!sentence_ptr || !word) return 0;

    sentence* s = static_cast<sentence*>(sentence_ptr);
    if (index >= s->words.size()) return 0;

    const auto& w = s->words[index];

    word->id = w.id;
    word->form = duplicate_string(w.form);
    word->lemma = duplicate_string(w.lemma);
    word->upostag = duplicate_string(w.upostag);
    word->xpostag = duplicate_string(w.xpostag);
    word->feats = duplicate_string(w.feats);
    word->head = w.head;
    word->deprel = duplicate_string(w.deprel);
    word->deps = duplicate_string(w.deps);
    word->misc = duplicate_string(w.misc);

    return 1; // Success
}

void udpipe_free_string(char* str) {
    free(str);
}

} // extern "C"
