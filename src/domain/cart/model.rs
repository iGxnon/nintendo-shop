use crate::domain::product::model::Product;
use crate::schema::{t_cart_entries, t_carts};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = t_carts)]
pub struct Cart {
    pub id: i64,
}

#[derive(Queryable, Selectable, Associations, Identifiable, Debug)]
#[diesel(belongs_to(Product, foreign_key = pid))]
#[diesel(belongs_to(Cart, foreign_key = cid))]
#[diesel(table_name = t_cart_entries)]
pub struct CartEntry {
    pub id: i64,
    pub cid: i64,
    pub pid: i64,
    pub quantity: i32,
    pub variant: i32,
}
