type Id = usize;

#[derive(Debug)]
pub enum Datum {
    Integer(i64),
    Boolean(bool),
    Character(char),
    String(String),
}

impl std::fmt::Display for Datum {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Datum::Integer(i) => write!(f, "{}", i),
            Datum::Boolean(true) => write!(f, "#t"),
            Datum::Boolean(false) => write!(f, "#f"),
            Datum::Character(c) => write!(f, "#\\{}", c),
            Datum::String(s) => write!(f, "{:?}", s),
        }
    }
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

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            // Op0
            Operation::ReadByte => write!(f, "read-byte"),
            Operation::PeekByte => write!(f, "peek-byte"),
            Operation::Void => write!(f, "void"),
            
            // Op1
            Operation::Add1(e) => write!(f, "add1 {}", e),
            Operation::Sub1(e) => write!(f, "sub1 {}", e),
            Operation::ZeroHuh(e) => write!(f, "zero? {}", e),
            Operation::CharHuh(e) => write!(f, "char? {}", e),
            Operation::IntegerToChar(e) => write!(f, "integer->char {}", e),
            Operation::CharToInteger(e) => write!(f, "char->integer {}", e),
            Operation::WriteByte(e) => write!(f, "write-byte {}", e),
            Operation::EofObjectHuh(e) => write!(f, " {}", e), 
            Operation::Box(e) => write!(f, "box {}", e),
            Operation::Car(e) => write!(f, "car {}", e),
            Operation::Cdr(e) => write!(f, "cdr {}", e),
            Operation::Unbox(e) => write!(f, "unbox {}", e),
            Operation::EmptyHuh(e) => write!(f, "empty? {}", e),
            Operation::ConsHuh(e) => write!(f, "cons? {}", e),
            Operation::BoxHuh(e) => write!(f, "box? {}", e),
            Operation::VectorHuh(e) => write!(f, "vector? {}", e),
            Operation::VectorLength(e) => write!(f, "vector-length {}", e),
            Operation::StringHuh(e) => write!(f, "string? {}", e),
            Operation::StringLength(e) => write!(f, "string-length {}", e),
           
            // Op2
            Operation::Plus(e1, e2) => write!(f, "+ {} {}", e1, e2),
            Operation::Sub(e1, e2)=> write!(f, "- {} {}", e1, e2),
            Operation::Less(e1, e2)=> write!(f, "< {} {}", e1, e2),  
            Operation::Equal(e1, e2) => write!(f, "equal? {} {}", e1, e2),
            Operation::EqHuh(e1, e2) => write!(f, "eq? {} {}", e1, e2),
            Operation::Cons(e1, e2) => write!(f, "cons {} {}", e1, e2),
            Operation::MakeVector(e1, e2) => write!(f, "make-vector {} {}", e1, e2),
            Operation::VectorRef(e1, e2) => write!(f, "vector-ref {} {}", e1, e2),
            Operation::MakeString(e1, e2) => write!(f, "make-string {} {}", e1, e2),
            Operation::StringRef(e1, e2) => write!(f, "string-ref {} {}", e1, e2),
            
            // Op3
            Operation::VectorSetBang(e1, e2, e3) => write!(f, "vector-set! {} {} {}", e1, e2, e3),
        }
    }
}

#[derive(Debug)]
pub enum Pattern {
    Var(Id),
    Literal(Datum),
    Box(Box<Pattern>),
    Cons(Box<Pattern>, Box<Pattern>),
    Conj(Box<Pattern>, Box<Pattern>),
}

impl std::fmt::Display for Pattern {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO: uh ??? im lazy
        write!(f, "({:?})", self)
    }
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

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expr::Literal(d) => write!(f, "{}", d),
            Expr::Op(o) => write!(f, "({})", o),
            Expr::If(e1, e2, e3) => write!(f, "(if {} {} {})", e1, e2, e3),
            Expr::Begin(e1, e2) => write!(f, "(begin\n  {}\n  {})", e1, e2),
            Expr::Let(id, e1, e2) => write!(f, "(let ([var{} {}])\n  {})", id, e1, e2),
            Expr::Var(id) => write!(f, "var{}", id),
            Expr::App(proc, es) => { write!(f, "({}", proc); 
                                     for e in es {
                                        write!(f, " {}", e);
                                     }
                                     write!(f, ")") },
            // TODO: add match and lam if we get to those
            _ => write!(f, "({:?})", self)
        }
    }
}


#[derive(Debug)]
pub struct Defn(pub Id, pub Vec<Id>, pub Box<Expr>);

impl std::fmt::Display for Defn {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.1.len() > 0 {
            write!(f, "(define (defn{}", self.0);
            for var in &self.1 {
               write!(f, " var{}", var);
            }
            write!(f, ")\n  {})", self.2)
        } else {
            write!(f, "(define defn{} {})", self.0, self.2)
        }
    }
}

#[derive(Debug)]
pub struct Program {
    pub defines: Vec<Defn>,
    pub expr: Box<Expr>,
}

impl std::fmt::Display for Program {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "#lang racket\n");
        for defn in &self.defines {
            write!(f, "{}\n", defn);
        };
        write!(f, "{}", self.expr)
    }
}
