use crate::graphql::model::common::{CurrencyCode, Image, Money};
use crate::infra::error::Status;
use crate::infra::id::Id;
use async_graphql::*;
use bigdecimal::BigDecimal;

#[derive(Clone)]
pub struct ProductVariant {
    pub id: Id<ProductVariant>,
    pub price: Money,
    pub title: String,
    pub inventory_count: i32,
    pub order_idx: i32,
}

#[derive(Clone)]
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

    async fn inventory_count(&self) -> i32 {
        self.inventory_count
    }

    async fn order_idx(&self) -> i32 {
        self.order_idx
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
        self.images.iter().find(|v| v.order_idx == 0)
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
                    amount: BigDecimal::from(0),
                    currency_code: CurrencyCode::USD,
                },
                min_variant_price: Money {
                    amount: BigDecimal::from(0),
                    currency_code: CurrencyCode::USD,
                },
            };
        }
        let mut min_at: usize = 0;
        let mut max_at: usize = 0;
        for (idx, variant) in self.variants.iter().enumerate() {
            if variant.price > self.variants[max_at].price {
                max_at = idx;
            }
            if variant.price < self.variants[min_at].price {
                min_at = idx;
            }
        }
        PriceRange {
            max_variant_price: self.variants[max_at].price.clone(),
            min_variant_price: self.variants[min_at].price.clone(),
        }
    }
}

impl TryFrom<volo_gen::product::v1::Product> for Product {
    type Error = Status;

    fn try_from(value: volo_gen::product::v1::Product) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            id: value.id.into(),
            title: value.title.into_string(),
            sub_title: value.sub_title.into_string(),
            description: value.description.into_string(),
            images: value
                .images
                .into_iter()
                .map(|v| Image {
                    url: v.url.into_string(),
                    alt_text: v.alt_text.into_string(),
                    order_idx: v.order_idx,
                })
                .collect(),
            variants: value
                .variants
                .into_iter()
                .map(|v| {
                    Ok(ProductVariant {
                        id: v.id.into(),
                        price: v.price.try_into()?,
                        title: v.title.into_string(),
                        inventory_count: v.inventory_count,
                        order_idx: v.order_idx,
                    })
                })
                .collect::<Result<Vec<_>, Status>>()?,
        })
    }
}
