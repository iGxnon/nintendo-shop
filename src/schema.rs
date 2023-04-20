// @generated automatically by Diesel CLI.

diesel::table! {
    t_product_images (id) {
        id -> Int8,
        pid -> Int8,
        url -> Varchar,
        alt_text -> Varchar,
    }
}

diesel::table! {
    t_product_variants (id) {
        id -> Int8,
        pid -> Int8,
        price -> Money,
        title -> Varchar,
        inventory_count -> Int4,
    }
}

diesel::table! {
    t_products (id) {
        id -> Int8,
        title -> Varchar,
        sub_title -> Varchar,
        description -> Text,
        currency_code -> Varchar,
    }
}

diesel::joinable!(t_product_images -> t_products (pid));
diesel::joinable!(t_product_variants -> t_products (pid));

diesel::allow_tables_to_appear_in_same_query!(
    t_product_images,
    t_product_variants,
    t_products,
);
