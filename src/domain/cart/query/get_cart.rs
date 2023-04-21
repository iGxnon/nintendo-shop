use crate::domain::cart::model;
use crate::graphql::Resolver;
use crate::infra::mqsrs::Query;
use crate::infra::resolver::BaseResolver;
use crate::schema::t_carts;
use anyhow::anyhow;
use anyhow::Result;
use bigdecimal::BigDecimal;
use diesel::data_types::PgMoney;
use diesel::sql_types::*;
use diesel::{PgConnection, QueryDsl, QueryableByName, RunQueryDsl, SelectableHelper};
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::DerefMut;
use std::ops::Div;
use volo_gen::cart::v1::{Cart, CartEntry, GetCartReq, GetCartRes};
use volo_gen::common::v1::{CurrencyCode, Image};
use volo_gen::product::v1::{Product, ProductVariant};

// complex join
const JOIN_QUERY: &str = r#"
SELECT
	"t_cart_entries"."id" AS "entry_id",
	"t_cart_entries"."pid" AS "entry_pid",
	"t_cart_entries"."quantity" AS "entry_quantity",
	"t_cart_entries"."variant" AS "entry_variant",
	"t_products"."id" AS "pid",
	"t_products"."title" AS "title",
	"t_products"."sub_title" AS "sub_title",
	"t_products"."description" AS "description",
	"t_products"."currency_code" AS "currency_code",
	"t_product_images"."id" AS "image_id",
	"t_product_images"."url" AS "image_url",
	"t_product_images"."alt_text" AS "image_alt_text",
	"t_product_variants"."id" AS "variant_id",
	"t_product_variants"."price" AS "variant_price",
	"t_product_variants"."title" AS "variant_title",
	"t_product_variants"."inventory_count" AS "variant_inventory_count" 
FROM
	(
		"t_cart_entries"
		INNER JOIN (
			( "t_products" LEFT OUTER JOIN "t_product_images" ON ( "t_product_images"."pid" = "t_products"."id" ) )
			LEFT OUTER JOIN "t_product_variants" ON ( "t_product_variants"."pid" = "t_products"."id" ) 
		) ON ( "t_cart_entries"."pid" = "t_products"."id" ) 
	) 
WHERE
	( "t_cart_entries"."cid" = $1 );
"#;

#[derive(QueryableByName, Clone, Debug)]
struct QueryData {
    #[diesel(sql_type = Int8)]
    entry_id: i64,
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
    #[diesel(sql_type = Nullable<Int8>)]
    image_id: Option<i64>,
    #[diesel(sql_type = Nullable<Varchar>)]
    image_url: Option<String>,
    #[diesel(sql_type = Nullable<Varchar>)]
    image_alt_text: Option<String>,
    #[diesel(sql_type = Nullable<Int8>)]
    variant_id: Option<i64>,
    #[diesel(sql_type = Nullable<Money>)]
    variant_price: Option<PgMoney>,
    #[diesel(sql_type = Nullable<Varchar>)]
    variant_title: Option<String>,
    #[diesel(sql_type = Nullable<Int4>)]
    variant_inventory_count: Option<i32>,
}

#[inline]
fn append_data(
    data: QueryData,
    entry: &mut CartEntry,
    code: CurrencyCode,
    images: &mut HashSet<i64>,
    variants: &mut HashSet<i64>,
) {
    if let Some(image_id) = data.image_id {
        if !images.contains(&image_id) {
            entry.product.images.push(Image {
                url: data.image_url.unwrap().into(),
                alt_text: data.image_alt_text.unwrap().into(),
            });
        }
        images.insert(image_id);
    }
    if let Some(variant_id) = data.variant_id {
        if !variants.contains(&variant_id) {
            entry.product.variants.push(ProductVariant {
                id: variant_id,
                price: volo_gen::common::v1::Money {
                    amount: (BigDecimal::from(data.variant_price.unwrap().0).div(100)
                        as BigDecimal)
                        .to_string()
                        .into(),
                    currency_code: code,
                },
                title: data.variant_title.unwrap().into(),
                inventory_count: data.variant_inventory_count.unwrap(),
            });
        }
        variants.insert(variant_id);
    }
}

fn execute(req: GetCartReq, conn: &mut PgConnection) -> Result<GetCartRes> {
    let cart = t_carts::table
        .find(req.id)
        .select(model::Cart::as_select())
        .get_result(conn)?;

    let data = diesel::sql_query(JOIN_QUERY)
        .bind::<Int8, _>(cart.id)
        .load::<QueryData>(conn)?;
    let mut entries = HashMap::<i64, CartEntry>::new();
    let mut images = HashSet::new();
    let mut variants = HashSet::new();
    for ele in data {
        let code = match &*ele.currency_code.to_uppercase() {
            "USD" => CurrencyCode::Usd,
            "CNY" => CurrencyCode::Cny,
            _ => return Err(anyhow!("error parsing currency_code")),
        };
        if let Some(v) = entries.get_mut(&ele.entry_id) {
            append_data(ele, v, code, &mut images, &mut variants);
            continue;
        }
        let data = ele.clone();
        let entry_id = data.entry_id;
        let mut entry = CartEntry {
            id: ele.entry_id,
            quantity: ele.entry_quantity,
            product: Product {
                id: ele.entry_pid,
                title: ele.title.into(),
                sub_title: ele.sub_title.into(),
                description: ele.description.into(),
                currency_code: code,
                images: vec![],
                variants: vec![],
            },
            variants_idx: ele.entry_variant,
        };
        append_data(data, &mut entry, code, &mut images, &mut variants);
        entries.insert(entry_id, entry);
    }

    Ok(GetCartRes {
        cart: Some(Cart {
            id: cart.id,
            entries: entries.into_values().collect(),
        }),
    })
}

fn execute2(req: GetCartReq, conn: &mut PgConnection) {
    // todo
}

#[test]
fn test() {
    use diesel::prelude::*;

    let mut conn =
        diesel::pg::PgConnection::establish("postgres://postgres:postgres@localhost/shop").unwrap();

    let res = execute(GetCartReq { id: 1 }, &mut conn).unwrap();

    println!("{:?}", res)
}

impl Resolver {
    pub fn create_get_cart(&self) -> impl Query<GetCartReq, Result<GetCartRes>> + '_ {
        move |req: GetCartReq| async move { execute(req, self.resolve(&self.pgsql).get()?.deref_mut()) }
    }
}
