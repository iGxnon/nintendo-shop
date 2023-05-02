namespace rs checkout.v1

include "cart.thrift"
include "common.thrift"

struct Shipping {
    1: required i64 id;
    2: required string vendor;
    // calculator fields...
}

struct Payment {
    1: required i64 id;
    2: required string vendor;
    // other fields...
}

struct Checkout {
    // required
    1: required i64 id;
    2: required cart.Cart cart;
    3: required i32 status;
    // optional
    4: optional Shipping shipping;
    5: optional Payment payment;
    6: optional common.Money shipping_fee;
    7: optional string contact_email;
    8: optional string receiver_name;
    9: optional string receiver_address;
    10: optional string receiver_phone;
}

struct PutCheckout {
    1: optional i64 shipping_id;
    2: optional i64 payment_id;
    3: optional string contact_email;
    4: optional string receiver_name;
    5: optional string receiver_address;
    6: optional string receiver_phone;
}

service CheckoutService {
    void ping();
    Checkout getCheckout(1: i64 id);
    Checkout getCheckoutByCardId(1: i64 card_id);
    Checkout putCheckout(1: i64 id, 2: PutCheckout put);
    Checkout putCheckoutByCardId(1: i64 card_id, 2: PutCheckout put);
}