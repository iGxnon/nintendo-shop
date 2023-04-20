use async_graphql::*;
use bigdecimal::BigDecimal;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul, MulAssign};
use std::str::FromStr;

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum CurrencyCode {
    USD,
    CNY,
}

impl From<volo_gen::common::v1::CurrencyCode> for CurrencyCode {
    fn from(value: volo_gen::common::v1::CurrencyCode) -> Self {
        match value {
            volo_gen::common::v1::CurrencyCode::Usd => CurrencyCode::USD,
            volo_gen::common::v1::CurrencyCode::Cny => CurrencyCode::CNY,
        }
    }
}

#[derive(Debug)]
pub struct ParseCurrencyCodeErr;

impl Display for ParseCurrencyCodeErr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseCurrencyCodeErr")
    }
}

impl FromStr for CurrencyCode {
    type Err = ParseCurrencyCodeErr;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match &*s.to_uppercase() {
            "USD" => Ok(CurrencyCode::USD),
            "CNY" => Ok(CurrencyCode::CNY),
            _ => Err(ParseCurrencyCodeErr),
        }
    }
}

#[derive(Clone)]
pub struct Money {
    pub amount: BigDecimal,
    pub currency_code: CurrencyCode,
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

pub struct Image {
    pub url: url::Url,
    pub alt_text: String,
}

#[Object]
impl Image {
    async fn url(&self) -> String {
        self.url.to_string()
    }

    async fn alt_text(&self) -> String {
        self.alt_text.to_string()
    }
}
