use crate::schema::{t_product_images, t_product_variants, t_products};
use diesel::data_types::PgMoney;
use diesel::prelude::*;

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
}
