-- Your SQL goes here
create table t_products
(
    id            bigserial             not null
        constraint t_products_pk
            primary key,
    title         varchar default ''    not null,
    sub_title     varchar default ''    not null,
    description   text    default ''    not null,
    currency_code varchar default 'USD' not null
);

comment on table t_products is 'products table';

comment on column t_products.id is 'pk';

comment on constraint t_products_pk on t_products is 'pk';

comment on column t_products.title is 'title of this product';

comment on column t_products.sub_title is 'sub title of this product';

comment on column t_products.description is 'description of this product';

comment on column t_products.currency_code is 'the currency code used in this product';

create table t_product_variants
(
    id              bigserial            not null
        constraint t_product_variants_pk
            primary key,
    pid             bigint               not null
        constraint t_product_variants_t_products_id_fk
            references t_products,
    price           money   default 0.00 not null,
    title           varchar default ''   not null,
    inventory_count integer default 0    not null
);

comment on table t_product_variants is 'product variants';

comment on column t_product_variants.id is 'pk';

comment on constraint t_product_variants_pk on t_product_variants is 'pk';

comment on column t_product_variants.pid is 'product id';

comment on constraint t_product_variants_t_products_id_fk on t_product_variants is 'fk';

comment on column t_product_variants.price is 'price number, money type decimal(17, 2)';

comment on column t_product_variants.title is 'title of this variant';

comment on column t_product_variants.inventory_count is 'count';

create table t_product_images
(
    id       bigserial          not null
        constraint t_product_images_pk
            primary key,
    pid      bigint             not null
        constraint t_product_images_t_products_id_fk
            references t_products,
    url      varchar default '' not null,
    alt_text varchar default '' not null
);

comment on table t_product_images is 'images';

comment on column t_product_images.id is 'pk';

comment on constraint t_product_images_pk on t_product_images is 'pk';

comment on column t_product_images.pid is 'fk';

comment on constraint t_product_images_t_products_id_fk on t_product_images is 'fk';

comment on column t_product_images.url is 'image url';

comment on column t_product_images.alt_text is 'altText of image';

