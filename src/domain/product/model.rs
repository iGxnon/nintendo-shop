use crate::infra::error::Status;
use crate::infra::error::{Range, Result};
use crate::schema::{t_product_images, t_product_variants, t_products};
use diesel::data_types::PgMoney;
use diesel::prelude::*;
use volo_gen::common::v1::{Image, Money, PaginationOption};
use volo_gen::product::v1::{Product, ProductConnection, ProductVariant};

const MAX_DATA_LEN: i64 = 100;

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = t_products)]
pub struct QueryProduct {
    pub id: i64,
    pub title: String,
    pub sub_title: String,
    pub description: String,
    pub currency_code: String,
}

#[derive(Insertable)]
#[diesel(table_name = t_products)]
pub struct NewProduct<'a> {
    pub title: &'a str,
    pub sub_title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub currency_code: &'a str,
}

#[derive(AsChangeset)]
#[diesel(table_name = t_products)]
pub struct MutateProduct<'a> {
    pub title: Option<&'a str>,
    pub sub_title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub currency_code: Option<&'a str>,
}

#[derive(Queryable, Selectable, Associations, Identifiable, Debug)]
#[diesel(belongs_to(QueryProduct, foreign_key = pid))]
#[diesel(table_name = t_product_images)]
pub struct QueryProductImage {
    pub id: i64,
    pub pid: i64,
    pub url: String,
    pub alt_text: String,
    pub order_idx: i32,
}

#[derive(Queryable, Selectable, Associations, Identifiable, Debug)]
#[diesel(belongs_to(QueryProduct, foreign_key = pid))]
#[diesel(table_name = t_product_variants)]
pub struct QueryProductVariant {
    pub id: i64,
    pub pid: i64,
    pub price: PgMoney,
    pub title: String,
    pub inventory_count: i32,
    pub order_idx: i32,
}

// Domain model hold an IDL model to representing data layout.
pub struct ProductDomain(Product);

impl ProductDomain {
    pub(in crate::domain) fn merge_query(
        product: QueryProduct,
        images: Vec<QueryProductImage>,
        variants: Vec<QueryProductVariant>,
    ) -> ProductDomain {
        ProductDomain(Product {
            id: product.id,
            title: product.title.into(),
            sub_title: product.sub_title.into(),
            description: product.description.into(),
            currency_code: product.currency_code.to_string().into(),
            images: images
                .into_iter()
                .map(|v| Image {
                    url: v.url.into(),
                    alt_text: v.alt_text.into(),
                    order_idx: v.order_idx,
                })
                .collect(),
            variants: variants
                .into_iter()
                .map(|v| ProductVariant {
                    id: v.id,
                    price: Money {
                        amount: v.price.0,
                        currency_code: product.currency_code.to_string().into(),
                    },
                    title: v.title.into(),
                    inventory_count: v.inventory_count,
                    order_idx: v.order_idx,
                })
                .collect(),
        })
    }

    pub(in crate::domain) fn into_product(self) -> Product {
        self.0
    }

    /// Query a product from database and return a domain model which might be used in
    /// further computing.
    /// Do serial query without a transaction, we dont need strong consistency.
    /// Status maybe returned:
    /// 1. not_found
    /// 2. internal
    pub(in crate::domain) fn query(id: i64, conn: &mut PgConnection) -> Result<ProductDomain> {
        let product = t_products::table
            .find(id)
            .select(QueryProduct::as_select())
            .get_result(conn)
            .map_err(|e| {
                if matches!(e, diesel::NotFound) {
                    Status::not_found(format!("product({})", id))
                } else {
                    Status::internal()
                }
            })?;
        let images = QueryProductImage::belonging_to(&product)
            .select(QueryProductImage::as_select())
            .load(conn)?;
        let variants = QueryProductVariant::belonging_to(&product)
            .select(QueryProductVariant::as_select())
            .load(conn)?;
        Ok(Self::merge_query(product, images, variants))
    }

    /// List products, if the `before` is not set in PaginationOption, it will query
    /// at most `MAX_DATA_LEN` lines from database.
    /// Do serial query without a transaction, we dont need strong consistency.
    /// Status maybe returned:
    /// 1. out_of_range
    /// 2. internal
    pub(in crate::domain) fn list(
        option: PaginationOption,
        conn: &mut PgConnection,
    ) -> Result<ProductConnection> {
        let mut start = 0;
        let mut end = MAX_DATA_LEN;
        if let Some(after) = option.after {
            let total = t_products::table.count().get_result::<i64>(conn)?;
            if after >= total {
                return Err(Status::out_of_range("after", Range::Continuous(0, total)));
            }
            start = after + 1;
            end = start + MAX_DATA_LEN;
        };
        if let Some(before) = option.before {
            if before <= 0 {
                return Err(Status::out_of_range("before", Range::StartAt(start)));
            }
            end = before;
        };
        let products = t_products::table
            .filter(t_products::id.between(start, end))
            .select(QueryProduct::as_select())
            .load(conn)?;
        let images = QueryProductImage::belonging_to(&products)
            .select(QueryProductImage::as_select())
            .load(conn)?
            .grouped_by(&products);
        let variants = QueryProductVariant::belonging_to(&products)
            .select(QueryProductVariant::as_select())
            .load(conn)?
            .grouped_by(&products);
        let mut products = products
            .into_iter()
            .zip(images.into_iter().zip(variants))
            .map(|(product, (images, variants))| Self::merge_query(product, images, variants).0)
            .collect::<Vec<_>>();
        let mut has_previous_page = false;
        let mut has_next_page = false;
        if let Some(first) = option.first.map(|v| v as usize) {
            if first < products.len() {
                products.truncate(first);
                has_next_page = true;
            }
        } else if let Some(last) = option.last.map(|v| v as usize) {
            if last < products.len() {
                products = products.drain((products.len() - last)..).collect();
                has_previous_page = true;
            }
        }
        Ok(ProductConnection {
            products,
            has_previous_page,
            has_next_page,
        })
    }

    // TODO
    pub(in crate::domain) fn create() {}

    // TODO
    pub(in crate::domain) fn mutate() {}
}
