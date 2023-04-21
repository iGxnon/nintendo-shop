namespace rs cart.v1

include "product.thrift"

struct Cart {
    1: required i64 id;
    2: required list<CartEntry> entries;
}

struct CartEntry {
    1: required i64 id;
    2: required i32 quantity;
    3: required product.Product product;
    4: required i32 variants_idx;
}

struct GetCartReq {
    1: required i64 id;
}

struct GetCartRes {
    1: optional Cart cart;
}

service CartService {
    void ping();
    GetCartRes getCart(1: GetCartReq req);
}