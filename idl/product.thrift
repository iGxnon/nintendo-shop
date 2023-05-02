namespace rs product.v1

include "common.thrift"

struct ProductVariant {
    1: required i64 id;
    2: required common.Money price;
    3: required string title;
    4: required i32 inventory_count;
    5: required i32 order_idx = 0;
}

struct Product {
    1: required i64 id;
    2: required string title;
    3: required string sub_title = ""
    4: required string description;
    5: required string currency_code;
    6: required list<common.Image> images;
    7: required list<ProductVariant> variants;
}

struct ProductConnection {
    1: required list<Product> products;
    2: required bool hasPreviousPage;
    3: required bool hasNextPage;
}

service ProductService {
    void ping();  // used for health check
    Product getProduct(1: i64 id);
    ProductConnection listProducts(1: common.PaginationOption params);
}