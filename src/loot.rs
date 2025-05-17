type Id = usize;

#[derive(Debug)]
pub enum Datum {
    Integer(i64),
    Boolean(bool),
    Character(char),
    String(String),
}

#[derive(Debug)]
pub enum Operation {
    // Op0
    ReadByte,
    PeekByte,
    Void,
    // Op1
    Add1(Box<Expr>),
    Sub1(Box<Expr>),
    ZeroHuh(Box<Expr>),
    CharHuh(Box<Expr>),
    IntegerToChar(Box<Expr>),
    CharToInteger(Box<Expr>),
    WriteByte(Box<Expr>),
    EofObjectHuh(Box<Expr>),
    Box(Box<Expr>),
    Car(Box<Expr>),
    Cdr(Box<Expr>),
    Unbox(Box<Expr>),
    EmptyHuh(Box<Expr>),
    ConsHuh(Box<Expr>),
    BoxHuh(Box<Expr>),
    VectorHuh(Box<Expr>),
    VectorLength(Box<Expr>),
    StringHuh(Box<Expr>),
    StringLength(Box<Expr>),
    // Op2
    Plus(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Less(Box<Expr>, Box<Expr>),
    Equal(Box<Expr>, Box<Expr>),
    EqHuh(Box<Expr>, Box<Expr>),
    Cons(Box<Expr>, Box<Expr>),
    MakeVector(Box<Expr>, Box<Expr>),
    VectorRef(Box<Expr>, Box<Expr>),
    MakeString(Box<Expr>, Box<Expr>),
    StringRef(Box<Expr>, Box<Expr>),
    // Op3
    VectorSetBang(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub enum Pattern {
    Var(Id),
    Literal(Datum),
    Box(Box<Pattern>),
    Cons(Box<Pattern>, Box<Pattern>),
    Conj(Box<Pattern>, Box<Pattern>),
}

#[derive(Debug)]
pub enum Expr {
    Literal(Datum),
    Op(Operation),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Begin(Box<Expr>, Box<Expr>),
    Let(Id, Box<Expr>, Box<Expr>),
    Var(Id),
    App(Box<Expr>, Vec<Expr>),
    Match(Box<Expr>, Vec<Pattern>, Vec<Expr>),
    Lam(Id, Vec<Id>, Box<Expr>),

    /// The decompiler wasn't able to figure out what's going on
    /// TODO: should this have info about the unknown instructions?
    Unknown,
}

#[derive(Debug)]
pub struct Defn(Id, Vec<Id>, Box<Expr>);

#[derive(Debug)]
pub struct Program {
    pub defines: Vec<Defn>,
    pub expr: Box<Expr>,
}
