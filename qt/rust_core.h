#pragma once

// Opaque, exclusively owned Rust handle. JSON transports data; Rust owns returned strings.
extern "C" {
void *ks_editor_new();
void ks_editor_free(void *editor);
char *ks_editor_request(void *editor, const char *input);
void ks_string_free(char *value);
}
