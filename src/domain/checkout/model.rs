use crate::domain::cart::model::{CartDomain, QueryCart};
use crate::infra::error::{PreconditionViolation, Result, Status};
use crate::schema::{t_checkouts, t_payment_methods, t_shipping_methods};
use diesel::data_types::PgMoney;
use diesel::prelude::*;
use volo_gen::checkout::v1::{Checkout, Payment, PutCheckout, Shipping};
use volo_gen::common::v1::Money;

#[derive(Queryable, Selectable, Associations, Identifiable, Debug)]
#[diesel(belongs_to(QueryCart, foreign_key = cid))]
#[diesel(table_name = t_checkouts)]
pub struct QueryCheckout {
    id: i64,
    cid: i64,
    status: i32,
    sid: Option<i64>,
    pid: Option<i64>,
    shipping_fee: Option<PgMoney>,
    email: Option<String>,
    full_name: Option<String>,
    address: Option<String>,
    phone: Option<String>,
}

#[derive(AsChangeset)]
#[diesel(table_name = t_checkouts)]
pub struct MutateCheckout<'a> {
    pub status: Option<i32>,
    pub sid: Option<i64>,
    pub pid: Option<i64>,
    pub email: Option<&'a str>,
    pub full_name: Option<&'a str>,
    pub address: Option<&'a str>,
    pub phone: Option<&'a str>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = t_checkouts)]
pub struct NewCheckout {
    cid: i64,
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = t_payment_methods)]
pub struct QueryPayment {
    vendor: String,
}

pub struct PaymentDomain(Payment);

impl PaymentDomain {
    pub(in crate::domain) fn into_payment(self) -> Payment {
        self.0
    }

    pub(in crate::domain) fn query(id: i64, conn: &mut PgConnection) -> Result<Self> {
        let payment = t_payment_methods::table
            .find(id)
            .select(QueryPayment::as_select())
            .get_result(conn)
            .map_err(|e| {
                if matches!(e, diesel::NotFound) {
                    Status::not_found(format!("payment_method({})", id))
                } else {
                    Status::internal()
                }
            })?;
        Ok(Self(Payment {
            id,
            vendor: payment.vendor.into(),
        }))
    }
}

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = t_shipping_methods)]
pub struct QueryShipping {
    vendor: String,
}

pub struct ShippingDomain(Shipping);

impl ShippingDomain {
    pub(in crate::domain) fn into_shipping(self) -> Shipping {
        self.0
    }

    pub(in crate::domain) fn query(id: i64, conn: &mut PgConnection) -> Result<Self> {
        let shipping = t_shipping_methods::table
            .find(id)
            .select(QueryShipping::as_select())
            .get_result(conn)
            .map_err(|e| {
                if matches!(e, diesel::NotFound) {
                    Status::not_found(format!("shipping_method({})", id))
                } else {
                    Status::internal()
                }
            })?;
        Ok(Self(Shipping {
            id,
            vendor: shipping.vendor.into(),
        }))
    }
}

pub struct CheckoutDomain(Checkout);

impl CheckoutDomain {
    pub(in crate::domain) fn into_checkout(self) -> Checkout {
        self.0
    }

    pub(in crate::domain) fn query(id: i64, conn: &mut PgConnection) -> Result<Self> {
        let checkout = t_checkouts::table
            .find(id)
            .select(QueryCheckout::as_select())
            .get_result(conn)
            .map_err(|e| {
                if matches!(e, diesel::NotFound) {
                    Status::not_found(format!("checkout({})", id))
                } else {
                    Status::internal()
                }
            })?;
        let cart = CartDomain::query(checkout.cid, conn)?.into_cart();
        if cart.entries.is_empty() {
            return Err(Status::failed_precondition().with_precondition(vec![
                PreconditionViolation {
                    r#type: "logic".to_string(),
                    subject: "nintendo-shop/checkout".to_string(),
                    description: "Checkout with an empty cart".to_string(),
                },
            ]));
        }
        let currency_code = cart.entries[0].product.currency_code.clone();
        let shipping = if let Some(sid) = checkout.sid {
            Some(ShippingDomain::query(sid, conn)?.into_shipping())
        } else {
            None
        };
        let payment = if let Some(pid) = checkout.pid {
            Some(PaymentDomain::query(pid, conn)?.into_payment())
        } else {
            None
        };
        Ok(CheckoutDomain(Checkout {
            id,
            cart,
            status: checkout.status,
            shipping,
            shipping_fee: checkout.shipping_fee.map(|money| Money {
                amount: money.0,
                currency_code,
            }),
            contact_email: checkout.email.map(Into::into),
            receiver_name: checkout.full_name.map(Into::into),
            receiver_address: checkout.address.map(Into::into),
            receiver_phone: checkout.phone.map(Into::into),
            payment,
        }))
    }

    pub(in crate::domain) fn query_by_cart_id(cid: i64, conn: &mut PgConnection) -> Result<Self> {
        let checkout = t_checkouts::table
            .filter(t_checkouts::cid.eq(cid))
            .select(QueryCheckout::as_select())
            .get_result(conn)
            .map_err(|e| {
                if matches!(e, diesel::NotFound) {
                    Status::not_found(format!("checkout(cid: {})", cid))
                } else {
                    Status::internal()
                }
            })?;
        Self::query(checkout.id, conn)
    }

    pub(in crate::domain) fn create(cid: i64, conn: &mut PgConnection) -> Result<Self> {
        if t_checkouts::table
            .filter(t_checkouts::cid.eq(cid))
            .select(t_checkouts::id)
            .first::<i64>(conn)
            .is_ok()
        {
            return Err(Status::already_exists(format!("checkout(cid: {})", cid)));
        }
        let id = diesel::insert_into(t_checkouts::table)
            .values(NewCheckout { cid })
            .returning(t_checkouts::id)
            .get_result(conn)?;
        Self::query(id, conn)
    }

    pub(in crate::domain) fn submit_information(
        &mut self,
        put: PutCheckout,
        conn: &mut PgConnection,
    ) -> Result<()> {
        diesel::update(t_checkouts::table)
            .filter(t_checkouts::id.eq(self.0.id))
            .set(MutateCheckout {
                status: None,
                sid: put.shipping_id,
                pid: put.payment_id,
                email: put.contact_email.as_deref(),
                full_name: put.receiver_name.as_deref(),
                address: put.receiver_address.as_deref(),
                phone: put.receiver_phone.as_deref(),
            })
            .execute(conn)?;
        if let Some(sid) = put.shipping_id {
            self.0.shipping = Some(ShippingDomain::query(sid, conn)?.into_shipping());
        }
        if let Some(pid) = put.payment_id {
            self.0.payment = Some(PaymentDomain::query(pid, conn)?.into_payment());
        }
        self.0.contact_email = put.contact_email;
        self.0.receiver_name = put.receiver_name;
        self.0.receiver_address = put.receiver_address;
        self.0.receiver_phone = put.receiver_phone;
        Ok(())
    }
}
