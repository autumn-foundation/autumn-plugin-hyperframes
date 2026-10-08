//! [`Variable`] declarations and [`VariableValue`] values.

/// A typed variable declaration (`data-composition-variables`).
///
/// ```rust
/// use autumn_plugin_hyperframes::Variable;
///
/// let title = Variable::string("title", "Pro").label("Title");
/// let plan = Variable::choice("plan", &["pro", "team"], "pro");
/// # let _ = (title, plan);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[must_use]
pub struct Variable {
    pub(crate) id: String,
    pub(crate) kind: VariableKind,
    pub(crate) label: Option<String>,
    pub(crate) description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum VariableKind {
    String(String),
    Number(f64),
    Color(String),
    Boolean(bool),
    Enum {
        options: Vec<String>,
        default: String,
    },
    Font(String),
    Image(String),
}

impl Variable {
    fn new(id: &str, kind: VariableKind) -> Self {
        Self {
            id: id.to_owned(),
            kind,
            label: None,
            description: None,
        }
    }

    /// A text variable.
    #[must_use]
    pub fn string(id: &str, default: &str) -> Self {
        Self::new(id, VariableKind::String(default.to_owned()))
    }

    /// A number variable. The default must be finite.
    #[must_use]
    pub fn number(id: &str, default: f64) -> Self {
        Self::new(id, VariableKind::Number(default))
    }

    /// A color variable, for example `#6c5ce7`.
    #[must_use]
    pub fn color(id: &str, default: &str) -> Self {
        Self::new(id, VariableKind::Color(default.to_owned()))
    }

    /// An on/off variable.
    #[must_use]
    pub fn boolean(id: &str, default: bool) -> Self {
        Self::new(id, VariableKind::Boolean(default))
    }

    /// A choice from `options` (type `enum`). The default must be one of them.
    #[must_use]
    pub fn choice(id: &str, options: &[&str], default: &str) -> Self {
        Self::new(
            id,
            VariableKind::Enum {
                options: options.iter().map(|o| (*o).to_owned()).collect(),
                default: default.to_owned(),
            },
        )
    }

    /// A font family variable.
    #[must_use]
    pub fn font(id: &str, default: &str) -> Self {
        Self::new(id, VariableKind::Font(default.to_owned()))
    }

    /// An image path variable.
    #[must_use]
    pub fn image(id: &str, default: &str) -> Self {
        Self::new(id, VariableKind::Image(default.to_owned()))
    }

    /// Sets the label that Studio and agents show. The default label is the id.
    #[must_use]
    pub fn label(mut self, label: &str) -> Self {
        self.label = Some(label.to_owned());
        self
    }

    /// Sets a description of what the value means.
    #[must_use]
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_owned());
        self
    }
}

/// A value for a nested composition variable (`data-variable-values`).
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum VariableValue {
    /// Text, a color, a font name or a path.
    String(String),
    /// A whole number.
    Integer(i64),
    /// A number. It must be finite.
    Number(f64),
    /// On or off.
    Boolean(bool),
}

impl From<&str> for VariableValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<String> for VariableValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<i32> for VariableValue {
    fn from(value: i32) -> Self {
        Self::Integer(value.into())
    }
}

impl From<i64> for VariableValue {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<u32> for VariableValue {
    fn from(value: u32) -> Self {
        Self::Integer(value.into())
    }
}

impl From<f64> for VariableValue {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

impl From<bool> for VariableValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}
