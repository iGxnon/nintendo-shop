use crate::domain::cart::model;
use crate::schema::t_carts;
use anyhow::Result;
use diesel::data_types::PgMoney;
use diesel::sql_types::*;
use diesel::{PgConnection, QueryDsl, QueryableByName, RunQueryDsl, SelectableHelper};
use volo_gen::cart::v1::{Cart, CartEntry, GetCartReq, GetCartRes};
use volo_gen::product::v1::Product;

// complex join
const JOIN_QUERY: &str = r#"
SELECT "t_cart_entries"."id"                  as "entry_id",
       "t_cart_entries"."cid"                 as "entry_cid",
       "t_cart_entries"."pid"                 as "entry_pid",
       "t_cart_entries"."quantity"            as "entry_quantity",
       "t_cart_entries"."variant"             as "entry_variant",
       "t_products"."id"                      as "pid",
       "t_products"."title"                   as "title",
       "t_products"."sub_title"               as "sub_title",
       "t_products"."description"             as "description",
       "t_products"."currency_code"           as "currency_code",
       "t_product_images"."id"                as "image_id",
       "t_product_images"."pid"               as "image_pid",
       "t_product_images"."url"               as "image_url",
       "t_product_images"."alt_text"          as "image_alt_text",
       "t_product_variants"."id"              as "variant_id",
       "t_product_variants"."pid"             as "variant_pid",
       "t_product_variants"."price"           as "variant_price",
       "t_product_variants"."title"           as "variant_title",
       "t_product_variants"."inventory_count" as "variant_inventory_count"
FROM ("t_cart_entries" INNER JOIN (("t_products" LEFT OUTER JOIN "t_product_images"
                                    ON ("t_product_images"."pid" = "t_products"."id")) LEFT OUTER JOIN "t_product_variants"
                                   ON ("t_product_variants"."pid" = "t_products"."id"))
      ON ("t_cart_entries"."pid" = "t_products"."id"))
WHERE ("t_cart_entries"."cid" = ?)
"#;

#[derive(QueryableByName)]
struct QueryData {
    #[diesel(sql_type = Int8)]
    entry_id: i64,
    #[diesel(sql_type = Int8)]
    entry_cid: i64,
    #[diesel(sql_type = Int8)]
    entry_pid: i64,
    #[diesel(sql_type = Int4)]
    entry_quantity: i32,
    #[diesel(sql_type = Int4)]
    entry_variant: i32,
    #[diesel(sql_type = Int8)]
    pid: i64,
    #[diesel(sql_type = Varchar)]
    title: String,
    #[diesel(sql_type = Varchar)]
    sub_title: String,
    #[diesel(sql_type = Text)]
    description: String,
    #[diesel(sql_type = Varchar)]
    currency_code: String,
    #[diesel(sql_type = Int8)]
    image_id: i64,
    #[diesel(sql_type = Int8)]
    image_pid: i64,
    #[diesel(sql_type = Varchar)]
    image_url: String,
    #[diesel(sql_type = Varchar)]
    image_alt_text: String,
    #[diesel(sql_type = Int8)]
    variant_id: i64,
    #[diesel(sql_type = Int8)]
    variant_pid: i64,
    #[diesel(sql_type = Money)]
    variant_price: PgMoney,
    #[diesel(sql_type = Varchar)]
    variant_title: String,
    #[diesel(sql_type = Int4)]
    variant_inventory_count: i32,
}

fn execute(req: GetCartReq, conn: &mut PgConnection) -> Result<GetCartRes> {
    let cart = t_carts::table
        .find(req.id)
        .select(model::Cart::as_select())
        .get_result(conn)?;

    let data = diesel::sql_query(JOIN_QUERY)
        .bind::<Int8, _>(cart.id)
        .load::<QueryData>(conn)?;

    let by = data.group_by(|a, b| a.entry_id == b.entry_id).into_iter();

    Ok(GetCartRes {
        cart: Some(Cart {
            id: cart.id,
            entries: data
                .into_iter()
                .map(|v| CartEntry {
                    id: v.entry_id,
                    quantity: v.entry_quantity,
                    product: Product {
                        id: v.pid,
                        title: Default::default(),
                        sub_title: Default::default(),
                        description: Default::default(),
                        currency_code: Default::default(),
                        images: vec![],
                        variants: vec![],
                    },
                    variants_idx: v.entry_variant,
                })
                .collect(),
        }),
    })
}
