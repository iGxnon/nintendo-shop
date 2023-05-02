use crate::infra::error::Status;
use async_graphql::*;
use bigdecimal::BigDecimal;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Div, Mul, MulAssign};
use std::str::FromStr;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum CurrencyCode {
    USD,
    CNY,
}

impl FromStr for CurrencyCode {
    type Err = Status;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match &*s.to_uppercase() {
            "USD" => Ok(CurrencyCode::USD),
            "CNY" => Ok(CurrencyCode::CNY),
            _ => Err(Status::internal()
                .with_debug_info(false, format!("Cannot parse currency code: {}", s))),
        }
    }
}

#[derive(Clone)]
pub struct Money {
    pub amount: BigDecimal,
    pub currency_code: CurrencyCode,
}

impl TryFrom<volo_gen::common::v1::Money> for Money {
    type Error = Status;

    fn try_from(value: volo_gen::common::v1::Money) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            amount: BigDecimal::from(value.amount).div(100),
            currency_code: value.currency_code.parse()?,
        })
    }
}

impl PartialEq<Self> for Money {
    fn eq(&self, other: &Self) -> bool {
        assert!(other.currency_code == self.currency_code);
        self.amount.eq(&other.amount)
    }
}

impl Eq for Money {}

impl PartialOrd<Self> for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        assert!(other.currency_code == self.currency_code);
        self.amount.partial_cmp(&other.amount)
    }
}

impl Ord for Money {
    fn cmp(&self, other: &Self) -> Ordering {
        assert!(other.currency_code == self.currency_code);
        self.amount.cmp(&other.amount)
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(rhs.currency_code == self.currency_code);
        Money {
            amount: self.amount.add(rhs.amount),
            currency_code: self.currency_code,
        }
    }
}

impl AddAssign for Money {
    fn add_assign(&mut self, rhs: Self) {
        assert!(rhs.currency_code == self.currency_code);
        self.amount.add_assign(rhs.amount)
    }
}

impl Mul for Money {
    type Output = Money;

    fn mul(self, rhs: Self) -> Self::Output {
        assert!(rhs.currency_code == self.currency_code);
        Money {
            amount: self.amount.mul(rhs.amount),
            currency_code: self.currency_code,
        }
    }
}

impl MulAssign for Money {
    fn mul_assign(&mut self, rhs: Self) {
        assert!(rhs.currency_code == self.currency_code);
        self.amount.mul_assign(rhs.amount)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        if iter.size_hint().0 == 0 {
            return Money {
                amount: BigDecimal::from(0),
                currency_code: CurrencyCode::USD,
            };
        }
        iter.reduce(|lhs, rhs| lhs.add(rhs)).unwrap()
    }
}

#[Object]
impl Money {
    async fn amount(&self) -> String {
        self.amount.to_string()
    }

    async fn currency_code(&self) -> CurrencyCode {
        self.currency_code
    }
}

#[derive(Clone)]
pub struct Image {
    pub url: String,
    pub alt_text: String,
    pub order_idx: i32,
}

#[Object]
impl Image {
    async fn url(&self) -> String {
        self.url.to_string()
    }

    async fn alt_text(&self) -> String {
        self.alt_text.to_string()
    }

    async fn order_idx(&self) -> i32 {
        self.order_idx
    }
}
