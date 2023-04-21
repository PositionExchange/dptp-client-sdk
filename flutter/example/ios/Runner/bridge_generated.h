#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct wire_uint_8_list {
  uint8_t *ptr;
  int32_t len;
} wire_uint_8_list;

typedef struct WireSyncReturnStruct {
  uint8_t *ptr;
  int32_t len;
  bool success;
} WireSyncReturnStruct;

typedef int64_t DartPort;

typedef bool (*DartPostCObjectFnType)(DartPort port_id, void *message);

void wire_initialize(int64_t port_, uint64_t chain_id);

void wire_load_tokens(int64_t port_);

void wire_get_router(int64_t port_);

void wire_set_account(int64_t port_, struct wire_uint_8_list *account);

void wire_calculate_price_plp(int64_t port_);

void wire_fetch_data(int64_t port_);

struct wire_uint_8_list *new_uint_8_list(int32_t len);

void free_WireSyncReturnStruct(struct WireSyncReturnStruct val);

void store_dart_post_cobject(DartPostCObjectFnType ptr);

static int64_t dummy_method_to_enforce_bundling(void) {
    int64_t dummy_var = 0;
    dummy_var ^= ((int64_t) (void*) wire_initialize);
    dummy_var ^= ((int64_t) (void*) wire_load_tokens);
    dummy_var ^= ((int64_t) (void*) wire_get_router);
    dummy_var ^= ((int64_t) (void*) wire_set_account);
    dummy_var ^= ((int64_t) (void*) wire_calculate_price_plp);
    dummy_var ^= ((int64_t) (void*) wire_fetch_data);
    dummy_var ^= ((int64_t) (void*) new_uint_8_list);
    dummy_var ^= ((int64_t) (void*) free_WireSyncReturnStruct);
    dummy_var ^= ((int64_t) (void*) store_dart_post_cobject);
    return dummy_var;
}