namespace rs common.v1

struct Image {
    1: required string url;
    2: required string alt_text = ""
    3: required i32 order_idx = 0;
}

struct Money {
    1: required i64 amount;  // represent all numbers instead of using float
    2: required string currency_code;
}

// Pagination by graphql
struct PaginationOption {
    1: optional i64 after;
    2: optional i64 before;
    3: optional i32 first;
    4: optional i32 last;
    5: optional string order_by;
}