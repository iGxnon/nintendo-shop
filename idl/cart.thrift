namespace rs cart.v1

include "product.thrift"

struct Cart {
    1: required i64 id;
    2: required list<CartEntry> entries;
}

struct CartEntry {
    1: required i64 id;
    2: required product.Product product;
    3: required i32 quantity;
    4: required i32 variants;
}

service CartService {
    void ping();
    Cart getCart(1: i64 id);
    i64 createCart();
}