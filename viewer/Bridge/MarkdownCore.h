/* Auto-generated C header for libmarkdown_core static library */
#ifndef MARKDOWN_CORE_H
#define MARKDOWN_CORE_H

#include <stdbool.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Parse markdown text and return a JSON string representing the AST.
 * The caller is responsible for freeing the returned string with markdown_free_string.
 * Returns NULL on failure.
 */
char *markdown_parse(const char *text);

/**
 * Load a UTF-8 text file from disk.
 * The caller is responsible for freeing the returned string with markdown_free_string.
 * Returns NULL if the file cannot be read or is not valid UTF-8.
 */
char *markdown_load_file(const char *path);

/**
 * Returns true if the given file path has a markdown extension.
 */
bool markdown_is_markdown_file(const char *path);

/**
 * Free a string previously returned by this library.
 * Passing NULL is safe and is a no-op.
 */
void markdown_free_string(char *s);

#ifdef __cplusplus
}
#endif

#endif /* MARKDOWN_CORE_H */
