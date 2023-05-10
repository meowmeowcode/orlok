use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub enum Op {
    StrEq(String),
    StrNe(String),
    StrStartsWith(String),
    StrEndsWith(String),
    StrContains(String),
    StrIn(Vec<String>),
    IntEq(i64),
    IntNe(i64),
    IntLt(i64),
    IntGt(i64),
    IntLte(i64),
    IntGte(i64),
    IntBetween(i64, i64),
    IntIn(Vec<i64>),
    BoolEq(bool),
    BoolNe(bool),
    FloatEq(f64),
    FloatNe(f64),
    FloatLt(f64),
    FloatGt(f64),
    FloatLte(f64),
    FloatGte(f64),
    DecimalEq(Decimal),
    DecimalNe(Decimal),
    DecimalLt(Decimal),
    DecimalGt(Decimal),
    DecimalLte(Decimal),
    DecimalGte(Decimal),
    DateTimeEq(DateTime<Utc>),
    DateTimeNe(DateTime<Utc>),
    DateTimeLt(DateTime<Utc>),
    DateTimeGt(DateTime<Utc>),
    DateTimeLte(DateTime<Utc>),
    DateTimeGte(DateTime<Utc>),
    UuidEq(Uuid),
    UuidNe(Uuid),
    UuidIn(Vec<Uuid>),
}

#[derive(Clone, Debug)]
pub enum Filter {
    And(Vec<F>),
    Or(Vec<F>),
    IsNone(String),
    Value { field: String, op: Op },
}

pub type F = Filter;

pub trait EqArg {
    fn to_op(self) -> Op;
}

pub trait NeArg {
    fn to_op(self) -> Op;
}

pub trait LtArg {
    fn to_op(self) -> Op;
}

pub trait LteArg {
    fn to_op(self) -> Op;
}

pub trait GtArg {
    fn to_op(self) -> Op;
}

pub trait GteArg {
    fn to_op(self) -> Op;
}

pub trait BetweenArg {
    fn to_op(self) -> Op;
}

pub trait InArg {
    fn to_op(self) -> Op;
}

pub trait ContainsArg {
    fn to_op(self) -> Op;
}

pub trait StartsWithArg {
    fn to_op(self) -> Op;
}

pub trait EndsWithArg {
    fn to_op(self) -> Op;
}

impl EqArg for i64 {
    fn to_op(self) -> Op {
        Op::IntEq(self)
    }
}

impl NeArg for i64 {
    fn to_op(self) -> Op {
        Op::IntNe(self)
    }
}

impl LtArg for i64 {
    fn to_op(self) -> Op {
        Op::IntLt(self)
    }
}

impl GtArg for i64 {
    fn to_op(self) -> Op {
        Op::IntGt(self)
    }
}

impl LteArg for i64 {
    fn to_op(self) -> Op {
        Op::IntLte(self)
    }
}

impl GteArg for i64 {
    fn to_op(self) -> Op {
        Op::IntGte(self)
    }
}

impl BetweenArg for (i64, i64) {
    fn to_op(self) -> Op {
        Op::IntBetween(self.0, self.1)
    }
}

impl InArg for Vec<i64> {
    fn to_op(self) -> Op {
        Op::IntIn(self)
    }
}

impl EqArg for String {
    fn to_op(self) -> Op {
        Op::StrEq(self)
    }
}

impl NeArg for String {
    fn to_op(self) -> Op {
        Op::StrNe(self)
    }
}

impl ContainsArg for String {
    fn to_op(self) -> Op {
        Op::StrContains(self)
    }
}

impl StartsWithArg for String {
    fn to_op(self) -> Op {
        Op::StrStartsWith(self)
    }
}

impl EndsWithArg for String {
    fn to_op(self) -> Op {
        Op::StrEndsWith(self)
    }
}

impl InArg for Vec<String> {
    fn to_op(self) -> Op {
        Op::StrIn(self)
    }
}

impl EqArg for &str {
    fn to_op(self) -> Op {
        Op::StrEq(self.to_string())
    }
}

impl NeArg for &str {
    fn to_op(self) -> Op {
        Op::StrNe(self.to_string())
    }
}

impl ContainsArg for &str {
    fn to_op(self) -> Op {
        Op::StrContains(self.to_string())
    }
}

impl StartsWithArg for &str {
    fn to_op(self) -> Op {
        Op::StrStartsWith(self.to_string())
    }
}

impl EndsWithArg for &str {
    fn to_op(self) -> Op {
        Op::StrEndsWith(self.to_string())
    }
}

impl InArg for Vec<&str> {
    fn to_op(self) -> Op {
        Op::StrIn(self.iter().map(|s| s.to_string()).collect())
    }
}

impl EqArg for bool {
    fn to_op(self) -> Op {
        Op::BoolEq(self)
    }
}

impl NeArg for bool {
    fn to_op(self) -> Op {
        Op::BoolNe(self)
    }
}

impl EqArg for f64 {
    fn to_op(self) -> Op {
        Op::FloatEq(self)
    }
}

impl NeArg for f64 {
    fn to_op(self) -> Op {
        Op::FloatNe(self)
    }
}

impl LtArg for f64 {
    fn to_op(self) -> Op {
        Op::FloatLt(self)
    }
}

impl GtArg for f64 {
    fn to_op(self) -> Op {
        Op::FloatGt(self)
    }
}

impl GteArg for f64 {
    fn to_op(self) -> Op {
        Op::FloatGte(self)
    }
}

impl LteArg for f64 {
    fn to_op(self) -> Op {
        Op::FloatLte(self)
    }
}

impl EqArg for DateTime<Utc> {
    fn to_op(self) -> Op {
        Op::DateTimeEq(self)
    }
}

impl NeArg for DateTime<Utc> {
    fn to_op(self) -> Op {
        Op::DateTimeNe(self)
    }
}

impl LtArg for DateTime<Utc> {
    fn to_op(self) -> Op {
        Op::DateTimeLt(self)
    }
}

impl GtArg for DateTime<Utc> {
    fn to_op(self) -> Op {
        Op::DateTimeGt(self)
    }
}

impl LteArg for DateTime<Utc> {
    fn to_op(self) -> Op {
        Op::DateTimeLte(self)
    }
}

impl GteArg for DateTime<Utc> {
    fn to_op(self) -> Op {
        Op::DateTimeGte(self)
    }
}

impl EqArg for Decimal {
    fn to_op(self) -> Op {
        Op::DecimalEq(self)
    }
}

impl NeArg for Decimal {
    fn to_op(self) -> Op {
        Op::DecimalNe(self)
    }
}

impl LtArg for Decimal {
    fn to_op(self) -> Op {
        Op::DecimalLt(self)
    }
}

impl GtArg for Decimal {
    fn to_op(self) -> Op {
        Op::DecimalGt(self)
    }
}

impl LteArg for Decimal {
    fn to_op(self) -> Op {
        Op::DecimalLte(self)
    }
}

impl GteArg for Decimal {
    fn to_op(self) -> Op {
        Op::DecimalGte(self)
    }
}

impl EqArg for Uuid {
    fn to_op(self) -> Op {
        Op::UuidEq(self)
    }
}

impl NeArg for Uuid {
    fn to_op(self) -> Op {
        Op::UuidNe(self)
    }
}

impl InArg for Vec<Uuid> {
    fn to_op(self) -> Op {
        Op::UuidIn(self)
    }
}

impl Filter {
    pub fn eq(field: impl Into<String>, val: impl EqArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn is_none(field: impl Into<String>) -> Self {
        Self::IsNone(field.into())
    }

    pub fn ne(field: impl Into<String>, val: impl NeArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn lt(field: impl Into<String>, val: impl LtArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn gt(field: impl Into<String>, val: impl GtArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn lte(field: impl Into<String>, val: impl LteArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn gte(field: impl Into<String>, val: impl GteArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn between(field: impl Into<String>, val: impl BetweenArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn in_(field: impl Into<String>, val: impl InArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn contains(field: impl Into<String>, val: impl ContainsArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn starts_with(field: impl Into<String>, val: impl StartsWithArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn ends_with(field: impl Into<String>, val: impl EndsWithArg) -> Self {
        Self::Value {
            field: field.into(),
            op: val.to_op(),
        }
    }

    pub fn and(filters: Vec<F>) -> Self {
        Self::And(filters)
    }

    pub fn or(filters: Vec<F>) -> Self {
        Self::Or(filters)
    }
}

#[derive(Clone, Debug)]
pub enum Order {
    Asc(String),
    Desc(String),
}

#[derive(Clone, Debug)]
pub struct Query {
    pub filter: Option<F>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order: Option<Vec<Order>>,
}

pub type Q = Query;

impl Query {
    pub fn new() -> Self {
        Self {
            filter: None,
            limit: None,
            offset: None,
            order: None,
        }
    }

    pub fn filter(filter: F) -> Self {
        Self {
            filter: Some(filter),
            limit: None,
            offset: None,
            order: None,
        }
    }

    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn order(mut self, order: Vec<Order>) -> Self {
        self.order = Some(order);
        self
    }
}
