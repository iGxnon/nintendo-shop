namespace rs product.v1

include "common.thrift"

struct ProductVariant {
    1: required i64 id;
    2: required common.Money price;
    3: required string title;
    4: required i32 inventory_count;
}

struct Product {
    1: required i64 id;
    2: required string title;
    3: required string subTitle = ""
    4: required string description;
    5: required common.CurrencyCode currencyCode;
    6: required list<common.Image> images;
    7: required list<ProductVariant> variants;
}

struct GetProductReq {
    1: required i64 id;
}

struct GetProductRes {
    1: optional Product product;
}

service ProductService {
    void ping();  // used for health check
    GetProductRes getProduct(1: GetProductReq req);
}