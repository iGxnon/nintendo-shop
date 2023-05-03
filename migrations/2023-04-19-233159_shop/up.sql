-- Your SQL goes here
create table t_products
(
    id            bigserial               not null
        constraint t_products_pk
            primary key,
    title         varchar   default ''    not null,
    sub_title     varchar   default ''    not null,
    description   text      default ''    not null,
    currency_code varchar   default 'USD' not null,
    created_at    timestamp default now() not null,
    updated_at    timestamp default now() not null
);

comment on table t_products is 'products table';

comment on column t_products.id is 'pk';

comment on column t_products.title is 'title of this product';

comment on column t_products.sub_title is 'sub title of this product';

comment on column t_products.description is 'description of this product';

comment on column t_products.currency_code is 'the currency code used in this product';

create table t_product_variants
(
    id              bigserial               not null
        constraint t_product_variants_pk
            primary key,
    pid             bigint                  not null
        constraint t_product_variants_t_products_id_fk
            references t_products,
    price           money     default 0     not null,
    title           varchar   default ''    not null,
    inventory_count int4      default 0     not null,
    order_idx       int4      default 0     not null,
    created_at      timestamp default now() not null,
    updated_at      timestamp default now() not null
);

create index t_product_variants_pid_index on t_product_variants (pid);

comment on table t_product_variants is 'product variants table';

comment on column t_product_variants.id is 'pk';

comment on column t_product_variants.pid is 'product id';

comment on column t_product_variants.price is 'price number, money type decimal(17, 2)';

comment on column t_product_variants.title is 'title of this variant';

comment on column t_product_variants.inventory_count is 'inventory count';

comment on column t_product_variants.order_idx is 'the index in variants of a product';

create table t_product_images
(
    id         bigserial               not null
        constraint t_product_images_pk
            primary key,
    pid        bigint                  not null
        constraint t_product_images_t_products_id_fk
            references t_products,
    url        varchar   default ''    not null,
    alt_text   varchar   default ''    not null,
    order_idx  int4      default 0     not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

create index t_product_images_pid_index on t_product_images (pid);

comment on table t_product_images is 'images';

comment on column t_product_images.id is 'pk';

comment on column t_product_images.pid is 'fk';

comment on column t_product_images.url is 'image url';

comment on column t_product_images.alt_text is 'altText of image';

comment on column t_product_images.order_idx is 'the index in images of a product, 0 is the feature image';

create table t_carts
(
    id         bigserial               not null
        constraint t_carts_pk
            primary key,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

comment on table t_carts is 'a cart for a client';

comment on column t_carts.id is 'pk';

create table t_cart_entries
(
    id         bigserial               not null
        constraint t_cart_entries_pk
            primary key,
    cid        bigint                  not null
        constraint t_cart_entries_t_carts_id_fk
            references t_carts,
    pid        bigint                  not null
        constraint t_cart_entries_t_products_id_fk
            references t_products,
    quantity   integer   default 1     not null,
    variant    integer   default 0     not null,
    created_at timestamp default now() not null,
    updated_at timestamp default now() not null
);

create index t_cart_entries_cid_index on t_cart_entries (cid);

create index t_cart_entries_pid_index on t_cart_entries (pid);

comment on table t_cart_entries is 'cart entries for every variant of products';

comment on column t_cart_entries.id is 'pk';

comment on column t_cart_entries.cid is 'fk to t_carts';

comment on column t_cart_entries.pid is 'fk to t_products';

comment on column t_cart_entries.quantity is 'the quantity of this item';

comment on column t_cart_entries.variant is 'the index of variants selected in product';

create table t_checkouts
(
    id           bigserial               not null
        constraint t_checkouts_pk
            primary key,
    cid          bigint                  not null unique
        constraint t_checkouts_t_carts_id_fk
            references t_carts,
    status       integer   default 0     not null,
    sid          bigint,
    pid          bigint,
    shipping_fee money,
    email        varchar,
    full_name    varchar,
    country_code varchar,
    address      varchar,
    postcode     varchar,
    phone        varchar,
    created_at   timestamp default now() not null,
    updated_at   timestamp default now() not null
);

comment on table t_checkouts is 'order table';

comment on column t_checkouts.id is 'pk';

comment on column t_checkouts.cid is 'fk of cart id';

comment on column t_checkouts.status is 'status of this order, waiting(0), paid(1), expired(2), ...';

comment on column t_checkouts.sid is 'fk of shipping id, optional';

comment on column t_checkouts.pid is 'fk of payment id, optional';

comment on column t_checkouts.shipping_fee is 'calculated shipping fee, optional';

comment on column t_checkouts.email is 'contact email, optional';

comment on column t_checkouts.full_name is 'receiver full name, required when checkout';

comment on column t_checkouts.country_code is 'receiver country code, required when checkout';

comment on column t_checkouts.address is 'receiver address, required when checkout';

comment on column t_checkouts.postcode is 'receiver post code, required when checkout';

comment on column t_checkouts.phone is 'receiver phone, required when checkout';

create table t_shipping_methods
(
    id     bigserial not null
        constraint t_shipping_methods_pk
            primary key,
    vendor varchar   not null
);

comment on table t_shipping_methods is 'shipping method table';

comment on column t_shipping_methods.id is 'pk';

comment on column t_shipping_methods.vendor is 'vendor name';

create table t_payment_methods
(
    id     bigserial not null
        constraint t_payment_methods_pk
            primary key,
    vendor varchar   not null
);

comment on table t_payment_methods is 'payment method table';

comment on column t_payment_methods.id is 'pk';

comment on column t_payment_methods.vendor is 'vendor name';