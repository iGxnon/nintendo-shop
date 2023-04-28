use crate::graphql::model::common::{CurrencyCode, Money};
use crate::graphql::model::product::Product;
use crate::infra::error::Status;
use crate::infra::id::Id;
use async_graphql::*;
use bigdecimal::BigDecimal;
use std::ops::Mul;

pub struct Cart {
    pub id: Id<Cart>,
    pub entries: Vec<CartEntry>,
}

#[derive(SimpleObject)]
pub struct MutationCart {
    pub cart: Cart,
}

pub struct CartEntry {
    pub id: Id<CartEntry>,
    pub quantity: i32,
    pub product: Product,
    pub variant: i32, // the selected variant in product, default 0
}

#[Object]
impl Cart {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    async fn entries(&self) -> &[CartEntry] {
        self.entries.as_slice()
    }

    // todo return map<CurrencyCode, Money>
    async fn total_amount(&self) -> Money {
        self.entries.iter().map(|v| v.calculate_amount()).sum()
    }
}

impl CartEntry {
    fn calculate_amount(&self) -> Money {
        if self.product.variants.is_empty() {
            return Money {
                amount: BigDecimal::from(0),
                currency_code: CurrencyCode::USD,
            };
        }
        let price = &self.product.variants[self.variant as usize].price;
        let total = price.amount.clone().mul(BigDecimal::from(self.quantity));
        Money {
            amount: total,
            currency_code: price.currency_code,
        }
    }
}

#[Object]
impl CartEntry {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    async fn quantity(&self) -> i32 {
        self.quantity
    }

    async fn product(&self) -> &Product {
        &self.product
    }

    async fn total_amount(&self) -> Money {
        self.calculate_amount()
    }

    async fn variant_at(&self) -> i32 {
        self.variant
    }
}

impl TryFrom<volo_gen::cart::v1::Cart> for Cart {
    type Error = Status;

    fn try_from(value: volo_gen::cart::v1::Cart) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            entries: value
                .entries
                .into_iter()
                .map(|v| {
                    Ok(CartEntry {
                        id: v.id.into(),
                        quantity: v.quantity,
                        product: v.product.try_into()?,
                        variant: v.variants,
                    })
                })
                .collect::<Result<Vec<_>, Status>>()?,
        })
    }
}
