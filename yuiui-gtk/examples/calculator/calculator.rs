use std::fmt::{self, Write as _};
use std::str::FromStr;

#[derive(Debug)]
pub struct Calculator {
    current_state: f64,
    pending_operant: Option<Operant>,
    pending_operator: Option<Operator>,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            current_state: 0.0,
            pending_operator: None,
            pending_operant: None,
        }
    }

    pub fn update(&mut self, action: Action) {
        match action {
            Action::Digit(digit) => {
                if let Some(operant) = &mut self.pending_operant {
                    operant.push(digit);
                } else {
                    self.pending_operant = Some(Operant::new(Sign::Plus, vec![digit], None));
                }
            }
            Action::Negate => {
                if let Some(operant) = &mut self.pending_operant {
                    operant.negate();
                } else {
                    self.current_state = -self.current_state;
                }
            }
            Action::Dot => {
                if let Some(operant) = &mut self.pending_operant {
                    operant.init_decimal_part();
                } else {
                    self.pending_operant = Some(Operant::new(Sign::Plus, vec![], Some(vec![])));
                }
            }
            Action::Operator(op) => {
                match (
                    self.pending_operator.replace(op),
                    self.pending_operant.take(),
                ) {
                    (Some(operator), Some(operant)) => {
                        self.current_state = operator.eval(self.current_state, operant.to_f64());
                    }
                    (None, Some(operant)) => {
                        self.current_state = operant.to_f64();
                    }
                    _ => {}
                };
            }
            Action::Equal => {
                match (self.pending_operator.take(), self.pending_operant.take()) {
                    (Some(operator), Some(operant)) => {
                        self.current_state = operator.eval(self.current_state, operant.to_f64());
                    }
                    (None, Some(operant)) => {
                        self.current_state = operant.to_f64();
                    }
                    _ => {}
                };
            }
            Action::Clear => {
                self.current_state = 0.0;
                self.pending_operant = None;
                self.pending_operator = None;
            }
        }
    }

    pub fn display(&self) -> Display {
        Display(self)
    }
}

pub struct Display<'a>(&'a Calculator);

impl<'a> fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(operator) = &self.0.pending_operator {
            self.0.current_state.fmt(f)?;
            f.write_char(' ')?;
            f.write_char(operator.into_char())?;
            if let Some(operant) = &self.0.pending_operant {
                f.write_char(' ')?;
                operant.fmt(f)?;
            }
        } else if let Some(operant) = &self.0.pending_operant {
            operant.fmt(f)?;
        } else {
            self.0.current_state.fmt(f)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum Action {
    Digit(Digit),
    Negate,
    Dot,
    Operator(Operator),
    Equal,
    Clear,
}

#[derive(Clone, Copy, Debug)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

impl Operator {
    pub fn eval(&self, lhs: f64, rhs: f64) -> f64 {
        match self {
            Self::Add => lhs + rhs,
            Self::Sub => lhs - rhs,
            Self::Mul => lhs * rhs,
            Self::Div => lhs / rhs,
            Self::Mod => lhs % rhs,
        }
    }

    pub fn into_char(self) -> char {
        match self {
            Self::Add => '+',
            Self::Sub => '-',
            Self::Mul => '×',
            Self::Div => '÷',
            Self::Mod => '%',
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Mod => "mod",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Digit {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Zero,
}

impl Digit {
    pub fn into_char(self) -> char {
        match self {
            Self::One => '1',
            Self::Two => '2',
            Self::Three => '3',
            Self::Four => '4',
            Self::Five => '5',
            Self::Six => '6',
            Self::Seven => '7',
            Self::Eight => '8',
            Self::Nine => '9',
            Self::Zero => '0',
        }
    }
}

#[derive(Debug)]
struct Operant {
    sign: Sign,
    integer_part: Vec<Digit>,
    decimal_part: Option<Vec<Digit>>,
}

impl Operant {
    fn new(sign: Sign, integer_part: Vec<Digit>, decimal_part: Option<Vec<Digit>>) -> Self {
        Self {
            sign,
            integer_part,
            decimal_part,
        }
    }

    fn push(&mut self, digit: Digit) {
        if let Some(decimal_part) = &mut self.decimal_part {
            decimal_part.push(digit);
        } else {
            self.integer_part.push(digit);
        }
    }

    fn init_decimal_part(&mut self) {
        self.decimal_part = Some(Vec::new());
    }

    fn negate(&mut self) {
        self.sign = self.sign.negate();
    }

    fn to_f64(&self) -> f64 {
        f64::from_str(&self.to_string()).unwrap()
    }
}

impl fmt::Display for Operant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.sign.is_minus() {
            f.write_char('-')?;
        }

        if self.integer_part.is_empty() {
            f.write_char('0')?;
        } else {
            for digit in &self.integer_part {
                f.write_char(digit.into_char())?;
            }
        }

        if let Some(decimal_part) = &self.decimal_part {
            f.write_char('.')?;
            for digit in decimal_part {
                f.write_char(digit.into_char())?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum Sign {
    Plus,
    Minus,
}

impl Sign {
    fn is_minus(&self) -> bool {
        match self {
            Self::Plus => false,
            Self::Minus => true,
        }
    }

    fn negate(&self) -> Self {
        match self {
            Self::Plus => Self::Minus,
            Self::Minus => Self::Plus,
        }
    }
}
