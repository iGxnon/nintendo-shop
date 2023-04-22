namespace rs common.v1

struct Image {
    1: required string url;
    2: required string altText = ""
    3: required i64 order_idx;
}

struct Money {
    1: required i64 amount;  // represent all numbers instead of using float
    2: required string currencyCode;
}