// @generated automatically by Diesel CLI.

diesel::table! {
    t_cart_entries (id) {
        id -> Int8,
        cid -> Int8,
        pid -> Int8,
        quantity -> Int4,
        variant -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    t_carts (id) {
        id -> Int8,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    t_checkouts (id) {
        id -> Int8,
        cid -> Int8,
        status -> Int4,
        sid -> Nullable<Int8>,
        pid -> Nullable<Int8>,
        shipping_fee -> Nullable<Money>,
        email -> Nullable<Varchar>,
        full_name -> Nullable<Varchar>,
        address -> Nullable<Varchar>,
        phone -> Nullable<Varchar>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    t_payment_methods (id) {
        id -> Int8,
        vendor -> Varchar,
    }
}

diesel::table! {
    t_product_images (id) {
        id -> Int8,
        pid -> Int8,
        url -> Varchar,
        alt_text -> Varchar,
        order_idx -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    t_product_variants (id) {
        id -> Int8,
        pid -> Int8,
        price -> Money,
        title -> Varchar,
        inventory_count -> Int4,
        order_idx -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    t_products (id) {
        id -> Int8,
        title -> Varchar,
        sub_title -> Varchar,
        description -> Text,
        currency_code -> Varchar,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    t_shipping_methods (id) {
        id -> Int8,
        vendor -> Varchar,
    }
}

diesel::joinable!(t_cart_entries -> t_carts (cid));
diesel::joinable!(t_cart_entries -> t_products (pid));
diesel::joinable!(t_checkouts -> t_carts (cid));
diesel::joinable!(t_product_images -> t_products (pid));
diesel::joinable!(t_product_variants -> t_products (pid));

diesel::allow_tables_to_appear_in_same_query!(
    t_cart_entries,
    t_carts,
    t_checkouts,
    t_payment_methods,
    t_product_images,
    t_product_variants,
    t_products,
    t_shipping_methods,
);
