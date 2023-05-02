use crate::graphql::model::cart::Cart;
use crate::graphql::model::common::Money;
use crate::infra::error::Status;
use crate::infra::id::Id;
use async_graphql::*;

#[derive(Copy, Clone)]
#[repr(u32)]
pub enum CheckoutStatus {
    Waiting = 0,
    Paid = 1,
    Expired = 2,
}

impl TryFrom<i32> for CheckoutStatus {
    type Error = Status;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(CheckoutStatus::Waiting),
            1 => Ok(CheckoutStatus::Paid),
            2 => Ok(CheckoutStatus::Expired),
            _ => Err(Status::internal().with_debug_info(
                false,
                format!("Error when paring checkout status, receive {}", value),
            )),
        }
    }
}

pub struct Shipping {
    pub id: Id<Shipping>,
    pub vendor: String,
}

impl From<volo_gen::checkout::v1::Shipping> for Shipping {
    fn from(value: volo_gen::checkout::v1::Shipping) -> Self {
        Self {
            id: value.id.into(),
            vendor: value.vendor.into_string(),
        }
    }
}

pub struct Payment {
    pub id: Id<Payment>,
    pub vendor: String,
}

impl From<volo_gen::checkout::v1::Payment> for Payment {
    fn from(value: volo_gen::checkout::v1::Payment) -> Self {
        Self {
            id: value.id.into(),
            vendor: value.vendor.into_string(),
        }
    }
}

pub struct Checkout {
    pub id: Id<Checkout>,
    pub cart: Cart,
    pub status: CheckoutStatus,
    pub shipping: Option<Shipping>,
    pub payment: Option<Payment>,
    pub shipping_fee: Option<Money>,
    pub contact_email: Option<String>,
    pub receiver_name: Option<String>,
    pub receiver_address: Option<String>,
    pub receiver_phone: Option<String>,
}

#[derive(SimpleObject)]
pub struct MutationCheckout {
    pub checkout: Checkout,
}

impl TryFrom<volo_gen::checkout::v1::Checkout> for Checkout {
    type Error = Status;

    fn try_from(value: volo_gen::checkout::v1::Checkout) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            cart: value.cart.try_into()?,
            status: value.status.try_into()?,
            shipping: value.shipping.map(Into::into),
            payment: value.payment.map(Into::into),
            shipping_fee: if let Some(fee) = value.shipping_fee {
                Some(fee.try_into()?)
            } else {
                None
            },
            contact_email: value.contact_email.map(Into::into),
            receiver_name: value.receiver_name.map(Into::into),
            receiver_address: value.receiver_address.map(Into::into),
            receiver_phone: value.receiver_phone.map(Into::into),
        })
    }
}

#[Object]
impl Shipping {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    async fn vendor(&self) -> String {
        self.vendor.to_string()
    }
}

#[Object]
impl Payment {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    async fn vendor(&self) -> String {
        self.vendor.to_string()
    }
}

#[Object]
impl Checkout {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    async fn cart(&self) -> &Cart {
        &self.cart
    }

    async fn status(&self) -> u32 {
        self.status as u32
    }

    async fn shipping(&self) -> Option<&Shipping> {
        self.shipping.as_ref()
    }

    async fn payment(&self) -> Option<&Payment> {
        self.payment.as_ref()
    }

    async fn shipping_fee(&self) -> Option<&Money> {
        self.shipping_fee.as_ref()
    }

    async fn contact_email(&self) -> Option<&String> {
        self.contact_email.as_ref()
    }

    async fn receiver_name(&self) -> Option<&String> {
        self.receiver_name.as_ref()
    }

    async fn receiver_address(&self) -> Option<&String> {
        self.receiver_address.as_ref()
    }

    async fn receiver_phone(&self) -> Option<&String> {
        self.receiver_phone.as_ref()
    }

    async fn total_amount(&self) -> Option<Money> {
        let sub_total = self.cart.entries.iter().map(|v| v.calculate_amount()).sum();
        self.shipping_fee.clone().map(|fee| fee + sub_total)
    }
}
