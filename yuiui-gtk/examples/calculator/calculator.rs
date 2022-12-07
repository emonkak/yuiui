use std::fmt::{self, Write as _};
use std::str::FromStr;

#[derive(Debug)]
pub struct Calculator {
    current_value: f64,
    pending_operant: Option<Operant>,
    pending_operator: Option<Operator>,
}

impl Calculator {
    pub fn new() -> Self {
        Self {
            current_value: 0.0,
            pending_operator: None,
            pending_operant: None,
        }
    }

    pub fn push_digit(&mut self, digit: Digit) {
        if let Some(operant) = &mut self.pending_operant {
            operant.push(digit);
        } else {
            self.pending_operant = Some(Operant::new(Sign::Plus, vec![digit], None));
        }
    }

    pub fn push_operator(&mut self, operator: Operator) {
        match (
            self.pending_operator.replace(operator),
            self.pending_operant.take(),
        ) {
            (Some(operator), Some(operant)) => {
                self.current_value = operator.eval(self.current_value, operant.to_f64());
            }
            (None, Some(operant)) => {
                self.current_value = operant.to_f64();
            }
            _ => {}
        };
    }

    pub fn push_dot(&mut self) {
        if let Some(operant) = &mut self.pending_operant {
            operant.init_decimal_part();
        } else {
            self.pending_operant = Some(Operant::new(Sign::Plus, vec![], Some(vec![])));
        }
    }

    pub fn negate(&mut self) {
        if let Some(operant) = &mut self.pending_operant {
            operant.negate();
        } else {
            self.current_value = -self.current_value;
        }
    }

    pub fn evaluate(&mut self) {
        match (self.pending_operator.take(), self.pending_operant.take()) {
            (Some(operator), Some(operant)) => {
                self.current_value = operator.eval(self.current_value, operant.to_f64());
            }
            (None, Some(operant)) => {
                self.current_value = operant.to_f64();
            }
            _ => {}
        }
    }

    pub fn clear(&mut self) {
        self.current_value = 0.0;
        self.pending_operant = None;
        self.pending_operator = None;
    }

    pub fn display(&self) -> Display {
        Display(self)
    }
}

pub struct Display<'a>(&'a Calculator);

impl<'a> fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(operator) = &self.0.pending_operator {
            self.0.current_value.fmt(f)?;
            f.write_char(' ')?;
            f.write_char(operator.into_char())?;
            if let Some(operant) = &self.0.pending_operant {
                f.write_char(' ')?;
                operant.fmt(f)?;
            }
        } else if let Some(operant) = &self.0.pending_operant {
            operant.fmt(f)?;
        } else {
            self.0.current_value.fmt(f)?;
        }
        Ok(())
    }
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
        if matches!(self.sign, Sign::Minus) {
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
    fn negate(&self) -> Self {
        match self {
            Self::Plus => Self::Minus,
            Self::Minus => Self::Plus,
        }
    }
}
