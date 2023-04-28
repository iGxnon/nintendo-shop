use crate::domain::product::model::{
    ProductDomain, QueryProduct, QueryProductImage, QueryProductVariant,
};
use crate::infra::error::{Result, Status};
use crate::schema::{t_cart_entries, t_carts, t_product_variants, t_products};
use diesel::prelude::*;
use std::collections::HashMap;
use volo_gen::cart::v1::{Cart, CartEntry};

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

#[derive(Insertable, Debug)]
#[diesel(table_name = t_cart_entries)]
pub struct NewCartEntry {
    pub cid: i64,
    pub pid: i64,
    pub quantity: i32,
    pub variant: i32,
}

pub struct CartDomain(Cart);

impl CartDomain {
    pub(in crate::domain) fn into_cart(self) -> Cart {
        self.0
    }

    pub(in crate::domain) fn create(conn: &mut PgConnection) -> Result<CartDomain> {
        let id = diesel::insert_into(t_carts::table)
            .default_values()
            .returning(t_carts::id)
            .get_result::<i64>(conn)?;
        Ok(CartDomain(Cart {
            id,
            entries: vec![],
        }))
    }

    pub(in crate::domain) fn query(id: i64, conn: &mut PgConnection) -> Result<CartDomain> {
        let cart = t_carts::table
            .find(id)
            .select(QueryCart::as_select())
            .get_result(conn)
            .map_err(|e| {
                if matches!(e, diesel::NotFound) {
                    Status::not_found(format!("cart({})", id))
                } else {
                    Status::internal()
                }
            })?;
        let entries: Vec<QueryCartEntry> = t_cart_entries::table
            .select(QueryCartEntry::as_select())
            .filter(t_cart_entries::cid.eq(cart.id))
            .load(conn)?;
        let pids: Vec<i64> = entries.iter().map(|v| v.pid).collect();
        let products: Vec<QueryProduct> = t_products::table
            .filter(t_products::id.eq_any(pids))
            .select(QueryProduct::as_select())
            .load(conn)?;
        let images = QueryProductImage::belonging_to(&products)
            .select(QueryProductImage::as_select())
            .load::<QueryProductImage>(conn)?
            .grouped_by(&products);
        let variants = QueryProductVariant::belonging_to(&products)
            .select(QueryProductVariant::as_select())
            .load::<QueryProductVariant>(conn)?
            .grouped_by(&products);
        let pids = products.iter().map(|v| v.id).collect::<Vec<_>>();
        let products = products
            .into_iter()
            .zip(images.into_iter().zip(variants))
            .map(|(product, (images, variants))| {
                ProductDomain::merge_query(product, images, variants).into_product()
            })
            .zip(pids)
            .map(|(k, v)| (v, k))
            .collect::<HashMap<_, _>>();
        let entries = entries
            .into_iter()
            .map(|v| CartEntry {
                id: v.id,
                quantity: v.quantity,
                product: products[&v.pid].clone(),
                variants: v.variant,
            })
            .collect::<Vec<_>>();
        Ok(CartDomain(Cart { id, entries }))
    }

    pub(in crate::domain) fn add_item(
        &mut self,
        variant_id: i64,
        conn: &mut PgConnection,
    ) -> Result<()> {
        let entry = self.0.entries.iter_mut().find(|v| {
            if let Some(variant) = v.product.variants.get(v.variants as usize) {
                return variant.id == variant_id;
            }
            false
        });
        if let Some(entry) = entry {
            diesel::update(t_cart_entries::table)
                .filter(t_cart_entries::id.eq(entry.id))
                .set(t_cart_entries::quantity.eq(entry.quantity + 1))
                .execute(conn)?;
            entry.quantity += 1
        } else {
            let (pid, order_idx) = t_product_variants::table
                .find(variant_id)
                .select((t_product_variants::pid, t_product_variants::order_idx))
                .get_result(conn)?;
            diesel::insert_into(t_cart_entries::table)
                .values(&NewCartEntry {
                    cid: self.0.id,
                    pid,
                    quantity: 1,
                    variant: order_idx,
                })
                .execute(conn)?;
            self.0.entries.push(CartEntry {
                id: self.0.id,
                product: ProductDomain::query(pid, conn)?.into_product(),
                quantity: 1,
                variants: order_idx,
            })
        }
        Ok(())
    }

    pub(in crate::domain) fn remove_item(
        &mut self,
        entry_id: i64,
        conn: &mut PgConnection,
    ) -> Result<()> {
        diesel::delete(t_cart_entries::table)
            .filter(t_cart_entries::id.eq(entry_id))
            .execute(conn)?;
        self.0.entries.retain(|v| v.id != entry_id);
        Ok(())
    }
}
