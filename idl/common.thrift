namespace rs common.v1

enum CurrencyCode {
    USD,
    CNY,
}

struct Image {
    1: required string url;
    2: required string altText = ""
}

struct Money {
    1: required string amount;  // represent all numbers instead of using float
    2: required CurrencyCode currencyCode;
}