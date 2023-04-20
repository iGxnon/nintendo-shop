use crate::graphql::model::common::{CurrencyCode, Image, Money};
use crate::infra::id::Id;
use async_graphql::*;
use bigdecimal::BigDecimal;

pub struct ProductVariant {
    pub id: Id<ProductVariant>,
    pub price: Money,
    pub title: String,
    pub inventory_count: i64,
}

pub struct Product {
    pub id: Id<Product>,
    pub title: String,
    pub sub_title: String,
    pub description: String,
    pub images: Vec<Image>,
    pub variants: Vec<ProductVariant>,
}

#[derive(SimpleObject)]
pub struct PriceRange {
    pub min_variant_price: Money,
    pub max_variant_price: Money,
}

#[Object]
impl ProductVariant {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    async fn price(&self) -> &Money {
        &self.price
    }

    async fn title(&self) -> &String {
        &self.title
    }

    async fn available_for_sale(&self) -> bool {
        self.inventory_count > 0
    }

    async fn inventory_count(&self) -> i64 {
        self.inventory_count
    }
}

#[Object]
impl Product {
    async fn id(&self) -> String {
        self.id.to_string()
    }

    async fn title(&self) -> &String {
        &self.title
    }

    async fn sub_title(&self) -> &String {
        &self.sub_title
    }

    async fn description(&self) -> &String {
        &self.description
    }

    async fn featured_image(&self) -> Option<&Image> {
        if self.images.is_empty() {
            return None;
        }
        Some(&self.images[0])
    }

    async fn images(&self) -> &[Image] {
        self.images.as_slice()
    }

    async fn variants(&self) -> &[ProductVariant] {
        self.variants.as_slice()
    }

    async fn price_range(&self) -> PriceRange {
        if self.variants.is_empty() {
            return PriceRange {
                max_variant_price: Money {
                    amount: BigDecimal::from(-1),
                    currency_code: CurrencyCode::USD,
                },
                min_variant_price: Money {
                    amount: BigDecimal::from(-1),
                    currency_code: CurrencyCode::USD,
                },
            };
        }
        let mut min: Money = self.variants[0].price.clone();
        let mut max: Money = self.variants[0].price.clone();
        for variant in &self.variants {
            if variant.price > max {
                max = variant.price.clone();
            }
            if variant.price < min {
                min = variant.price.clone();
            }
        }
        PriceRange {
            max_variant_price: max,
            min_variant_price: min,
        }
    }
}
