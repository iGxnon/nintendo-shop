use crate::domain::product::model::QueryProduct;
use crate::schema::{t_cart_entries, t_carts};
use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, Debug)]
#[diesel(table_name = t_carts)]
pub struct QueryCart {
    pub id: i64,
}

#[derive(Queryable, Selectable, Associations, Identifiable, Debug)]
#[diesel(belongs_to(QueryProduct, foreign_key = pid))]
#[diesel(belongs_to(QueryCart, foreign_key = cid))]
#[diesel(table_name = t_cart_entries)]
pub struct QueryCartEntry {
    pub id: i64,
    pub cid: i64,
    pub pid: i64,
    pub quantity: i32,
    pub variant: i32,
}
