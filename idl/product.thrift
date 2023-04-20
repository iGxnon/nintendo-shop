namespace rs product.v1

include "common.thrift"

struct ProductPriceRange {
    1: required common.Money minVariantPrice;
    2: required common.Money maxVariantPrice;
}

struct ProductVariant {
    1: required i64 id;
    2: required common.Money price;
    3: required string title;
    4: required bool availableForSale;
}

struct Product {
    1: optional i64 id;
    2: optional string title;
    3: optional string subTitle = ""
    4: optional string description;
    5: optional list<common.Image> images;
    6: optional list<ProductVariant> variants;
    7: optional ProductPriceRange priceRange;
}


service ProductService {
    void ping();  // used for health check
}