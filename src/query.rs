#[derive(Clone)]
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
}

#[derive(Clone)]
pub enum F {
    And(Vec<F>),
    Or(Vec<F>),
    IsNone(String),
    Value { field: String, op: Op },
}

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

impl EqArg for &str {
    fn to_op(self) -> Op {
        Op::StrEq(self.to_string())
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

impl F {
    pub fn eq(field: impl ToString, val: impl EqArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn is_none(field: impl ToString) -> Self {
        Self::IsNone(field.to_string())
    }

    pub fn ne(field: impl ToString, val: impl NeArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn lt(field: impl ToString, val: impl LtArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn gt(field: impl ToString, val: impl GtArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn lte(field: impl ToString, val: impl LteArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn gte(field: impl ToString, val: impl GteArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn between(field: impl ToString, val: impl BetweenArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn in_(field: impl ToString, val: impl InArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn contains(field: impl ToString, val: impl ContainsArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn starts_with(field: impl ToString, val: impl StartsWithArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn ends_with(field: impl ToString, val: impl EndsWithArg) -> Self {
        Self::Value {
            field: field.to_string(),
            op: val.to_op(),
        }
    }

    pub fn and(filters: &[F]) -> Self {
        Self::And(filters.to_vec())
    }

    pub fn or(filters: &[F]) -> Self {
        Self::Or(filters.to_vec())
    }
}

pub enum Order {
    Asc(String),
    Desc(String),
}

pub struct Query {
    pub filter: Option<F>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub order: Option<Vec<Order>>,
}

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

    pub fn limit(&mut self, limit: usize) -> &Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(&mut self, offset: usize) -> &Self {
        self.offset = Some(offset);
        self
    }

    pub fn order(&mut self, order: Vec<Order>) -> &Self {
        self.order = Some(order);
        self
    }
}
