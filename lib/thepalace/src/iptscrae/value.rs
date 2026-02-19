//! Value types for Iptscrae runtime.
//!
//! Iptscrae is loosely typed with values that can be integers or strings.
//! The stack holds values that can be manipulated by operations.

/// Runtime value on the stack
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Integer(i32),
    String(String),
    Array(Vec<Value>),
}

impl Value {
    /// Create an integer value
    pub const fn integer(n: i32) -> Self {
        Value::Integer(n)
    }

    /// Create a string value
    pub fn string(s: impl Into<String>) -> Self {
        Value::String(s.into())
    }

    /// Create an array value
    pub fn array(elements: Vec<Value>) -> Self {
        Value::Array(elements)
    }

    /// Try to get integer value
    pub const fn as_integer(&self) -> Option<i32> {
        match self {
            Value::Integer(n) => Some(*n),
            Value::String(_) | Value::Array(_) => None,
        }
    }

    /// Try to get string value
    pub fn as_string(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            Value::Integer(_) | Value::Array(_) => None,
        }
    }

    /// Try to get array value
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            Value::Integer(_) | Value::String(_) => None,
        }
    }

    /// Try to get mutable array value
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        match self {
            Value::Array(arr) => Some(arr),
            Value::Integer(_) | Value::String(_) => None,
        }
    }

    /// Convert to integer (string "123" -> 123, or 0 if invalid)
    pub fn to_integer(&self) -> i32 {
        match self {
            Value::Integer(n) => *n,
            Value::String(s) => s.parse().unwrap_or(0),
            Value::Array(_) => 0,
        }
    }

    /// Convert to boolean (0 or empty string = false, otherwise true)
    pub fn to_bool(&self) -> bool {
        match self {
            Value::Integer(n) => *n != 0,
            Value::String(s) => !s.is_empty(),
            Value::Array(arr) => !arr.is_empty(),
        }
    }

    /// Check if value is an integer
    pub const fn is_integer(&self) -> bool {
        matches!(self, Value::Integer(_))
    }

    /// Check if value is a string
    pub const fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    /// Check if value is an array
    pub const fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }

    /// Get type name for debugging
    pub const fn type_name(&self) -> &'static str {
        match self {
            Value::Integer(_) => "integer",
            Value::String(_) => "string",
            Value::Array(_) => "array",
        }
    }
}

impl From<i32> for Value {
    fn from(n: i32) -> Self {
        Value::Integer(n)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Integer(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_creation() {
        let v1 = Value::integer(42);
        assert_eq!(v1, Value::Integer(42));

        let v2 = Value::string("hello");
        assert_eq!(v2, Value::String("hello".to_string()));
    }

    #[test]
    fn test_value_conversion() {
        let v1 = Value::Integer(42);
        assert_eq!(v1.to_integer(), 42);
        assert_eq!(v1.to_string(), "42");

        let v2 = Value::String("123".to_string());
        assert_eq!(v2.to_integer(), 123);
        assert_eq!(v2.to_string(), "123");

        let v3 = Value::String("not a number".to_string());
        assert_eq!(v3.to_integer(), 0);
    }

    #[test]
    fn test_value_bool_conversion() {
        assert!(Value::Integer(1).to_bool());
        assert!(!Value::Integer(0).to_bool());
        assert!(Value::String("text".to_string()).to_bool());
        assert!(!Value::String("".to_string()).to_bool());
    }

    #[test]
    fn test_value_from() {
        let v1: Value = 42.into();
        assert_eq!(v1, Value::Integer(42));

        let v2: Value = "hello".into();
        assert_eq!(v2, Value::String("hello".to_string()));

        let v3: Value = String::from("world").into();
        assert_eq!(v3, Value::String("world".to_string()));
    }

    #[test]
    fn test_value_display() {
        assert_eq!(format!("{}", Value::Integer(42)), "42");
        assert_eq!(format!("{}", Value::String("hello".to_string())), "hello");
    }
}
