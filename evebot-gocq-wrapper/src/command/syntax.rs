use itertools::Itertools;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct ServiceSyntax<const PARAM_ARG_LEN: usize, const POS_ARG_LEN: usize> {
    pub title: &'static str,
    pub arg_prefix: &'static str,
    pub positional_args: [PositionalArg; POS_ARG_LEN],
    pub param_args: [ParamArg; PARAM_ARG_LEN],
    pub description: &'static str,
}

//noinspection DuplicatedCode
impl<const PARAM_ARG_LEN: usize, const POSITIONAL_ARG_LEN: usize> Display
    for ServiceSyntax<PARAM_ARG_LEN, POSITIONAL_ARG_LEN>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r"{title}
{prefix} {positional_args_tag}{param_args_tag}
位置参数：
{positional_args}
额外参数：
{param_args}",
            title = &self.title,
            prefix = &self.arg_prefix,
            positional_args_tag = if POSITIONAL_ARG_LEN == 0 {
                "".to_owned()
            } else {
                self.positional_args
                    .iter()
                    .map(|s| format!("[{}]", s.arg_name))
                    .join(", ")
                    + " "
            },
            param_args_tag = if PARAM_ARG_LEN == 0 {
                ""
            } else {
                "<Param Args>"
            },
            positional_args = if POSITIONAL_ARG_LEN == 0 {
                "无".to_owned()
            } else {
                self.positional_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            },
            param_args = if PARAM_ARG_LEN == 0 {
                "无".to_owned()
            } else {
                self.param_args
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct ParamArg {
    pub alias: Option<&'static [&'static str]>,
    pub arg_name: &'static str,
    pub arg_type: ArgType,
    pub default: Option<ArgValue>,
    pub description: &'static str,
}

//noinspection DuplicatedCode
impl Display for ParamArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{name: <20}Type: {arg_type}.{optional}{default}\n                    {description}{alias}",
            name = &self.arg_name,
            arg_type = &self.arg_type,
            optional = if self.default.is_some() {
                " Optional."
            } else {
                ""
            },
            default = if let Some(d) = &self.default {
                format!(" Default: {}.", d)
            } else {
                "".into()
            },
            description = self.description,
            alias = if let Some(val) = self.alias {
                format!("\n                    Alias: {}", val.join(", "))
            } else {
                "".to_string()
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct PositionalArg {
    pub arg_name: &'static str,
    pub arg_type: ArgType,
    pub description: &'static str,
}

//noinspection DuplicatedCode
impl Display for PositionalArg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{name: <20}Type: {arg_type}.\n                    {description}",
            name = self.arg_name,
            arg_type = self.arg_type,
            description = self.description,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ArgType {
    AnyText,
    EnumText(&'static [&'static str]),
    Float,
    Int,
    Boolean,
}

//noinspection DuplicatedCode
impl Display for ArgType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnyText => write!(f, "TEXT"),
            Self::EnumText(t) => write!(f, "ENUM[{}]", t.join(", ")),
            Self::Float => write!(f, "FLOAT"),
            Self::Int => write!(f, "INT"),
            Self::Boolean => write!(f, "BOOLEAN"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ArgValue {
    AnyText(String),
    EnumText(&'static str),
    Float(f64),
    Int(i64),
    Boolean(bool),
}

//noinspection DuplicatedCode
impl Display for ArgValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AnyText(text) => write!(f, "{}", text),
            Self::EnumText(text) => write!(f, "{}", text),
            Self::Float(val) => write!(f, "{:?}", val),
            Self::Int(val) => write!(f, "{:?}", val),
            Self::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
        }
    }
}
