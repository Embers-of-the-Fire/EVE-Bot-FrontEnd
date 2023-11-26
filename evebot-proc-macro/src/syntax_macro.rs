use itertools::Itertools;
use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use serde::Deserialize;
use std::fmt::{Display, Formatter};
use syn::ItemStruct;

#[derive(Debug, Clone, Deserialize)]
pub struct ServiceSyntax {
    pub title: String,
    pub arg_prefix: String,
    pub positional_args: Vec<PositionalArg>,
    pub param_args: Vec<ParamArg>,
    pub description: String,
    #[serde(default)]
    pub mixin: Vec<String>,
}

//noinspection DuplicatedCode
impl Display for ServiceSyntax {
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
            positional_args_tag = if self.positional_args.is_empty() {
                "".to_owned()
            } else {
                self.positional_args
                    .iter()
                    .map(|s| format!("[{}]", s.arg_name))
                    .join(", ")
                    + " "
            },
            param_args_tag = if self.param_args.is_empty() {
                ""
            } else {
                "<Param Args>"
            },
            positional_args = if self.positional_args.is_empty() {
                "无".to_owned()
            } else {
                self.positional_args
                    .iter()
                    .map(ToString::to_string)
                    .join("\n")
            },
            param_args = if self.param_args.is_empty() {
                "无".to_owned()
            } else {
                self.param_args.iter().map(ToString::to_string).join("\n")
            }
        )
    }
}

impl ServiceSyntax {
    fn process_mixin(mut self) -> Self {
        let mixins = self.mixin.iter().map(|s| {
            serde_json::from_slice::<'_, Self>(&std::fs::read(&s).unwrap_or_else(print_fs_err(&s)))
                .unwrap()
                .process_mixin()
        });
        for mut mixin in mixins {
            self.param_args.append(&mut mixin.param_args);
            self.positional_args.append(&mut mixin.positional_args);
        }
        self
    }

    fn to_parser(&self, struct_name: &Ident) -> TokenStream {
        let result_struct_type = struct_name.clone();
        let result_struct = {
            let val = self
                .positional_args
                .iter()
                .map(|s| (&s.arg_name, &s.arg_type))
                .chain(self.param_args.iter().map(|s| (&s.arg_name, &s.arg_type)))
                .map(|(s, t)| {
                    let _ident = Ident::new(s, Span::call_site());
                    let t: TokenStream = t.as_value_type();
                    quote! {
                        pub #_ident: #t
                    }
                });
            quote! {
                #[derive(Debug, Clone, Default)]
                pub struct #result_struct_type {
                    #(#val),*
                }
            }
        };
        let result_ident = Ident::new("_result_group", Span::call_site());
        let pos_args = self.positional_args.iter().map(|s| {
            let name_lit = Literal::string(&format!("{} [{}]", &s.arg_name, &s.arg_type));
            let unknown_note_lit = Literal::string(&format!("缺少位置参数 {}", &s.arg_name));
            let name_ident = Ident::new(&format!("__result_{}", &s.arg_name), Span::call_site());
            let cache_val = Ident::new("__cache_val", Span::call_site());
            let parser: TokenStream = s.arg_type.to_parser(ArgTypeParser {
                val_ident: &cache_val,
                result_ident: &result_ident,
                name_lit: &name_lit,
            });
            let ret_type = s.arg_type.as_return_type();
            quote! {
                let #name_ident: Option<#ret_type> = if let Some(#cache_val) = param.next() {
                    #parser
                } else {
                    #result_ident.push(crate::error::BotError::Syntax {
                        found: None,
                        expected: Some(#name_lit.to_string()),
                        note: Some(#unknown_note_lit.to_string()),
                    });
                    None
                };
            }
        });
        let param_args = {
            let map_ident = Ident::new("__map_ident", Span::call_site());
            let getter = self.param_args.iter().map(|s| {
                let name_ident =
                    Ident::new(&format!("__result_{}", &s.arg_name), Span::call_site());
                let name_text = Literal::string(&s.arg_name);
                let name_lit = Literal::string(&format!("{} [{}]", &s.arg_name, &s.arg_type));
                let parser: TokenStream = s.arg_type.to_parser(ArgTypeParser {
                    val_ident: &Ident::new("_v", Span::call_site()),
                    result_ident: &result_ident,
                    name_lit: &name_lit,
                });
                let ret_type = s.arg_type.as_return_type();
                if let Some(d) = &s.default {
                    let _token: TokenStream = d.to_value_token();
                    quote! {
                        let #name_ident: Option<#ret_type> = if let Some(_v) = #map_ident.get(#name_text) {
                            #parser
                        } else {
                            Some(#_token)
                        };
                    }
                } else {
                    let unknown_note_lit =
                        Literal::string(&format!("缺少额外参数 {}", &s.arg_name));
                    quote! {
                        let #name_ident: #ret_type = if let Some(_v) = #map_ident.get(#name_text) {
                            #parser
                        } else {
                            #result_ident.push(crate::error::BotError::Syntax {
                                found: None,
                                expected: Some(#name_lit.to_string()),
                                note: Some(#unknown_note_lit.to_string()),
                            })
                            None
                        };
                    }
                }
            });
            let matcher = self.param_args.iter().flat_map(|s| {
                let _v = Literal::string(&s.arg_name);
                s.alias
                    .as_ref()
                    .unwrap_or(&vec![])
                    .iter()
                    .chain([s.arg_name.to_owned()].iter())
                    .map(|s| {
                        let _s = Literal::string(s);
                        quote! {
                            #_s => #_v
                        }
                    })
                    .collect::<Vec<_>>()
            });
            if self.param_args.is_empty() {
                quote! {}
            } else {
                quote! {
                    let mut #map_ident: ::std::collections::HashMap<::std::string::String, crate::server::ParamItem>
                        = ::std::collections::HashMap::new();
                    while let Some(param_item) = param.next() {
                        if let crate::server::ParamItem::Text(_text) = param_item {
                            if let Some(_val) = param.peek() {
                                #map_ident.insert(
                                    match _text.as_str() {
                                        #(#matcher),*,
                                        _n @ _ => _n
                                    }.to_string(),
                                    (*_val).clone()
                                );
                                param.next();
                            } else {
                                break;
                            }
                        }
                    }
                    #(#getter)*
                }
            }
        };

        let result_val_ident = Ident::new("__result", Span::call_site());
        let assignment = {
            let vals = self
                .param_args
                .iter()
                .map(|s| &s.arg_name)
                .chain(self.positional_args.iter().map(|s| &s.arg_name))
                .map(|s| {
                    let _ident = Ident::new(&format!("__result_{}", s), Span::call_site());
                    let _ass_ident = Ident::new(s, Span::call_site());
                    quote! {
                        #result_val_ident.#_ass_ident = #_ident.unwrap().into();
                    }
                });
            quote! {
                #(#vals)*
            }
        };

        let param_arg_len = Literal::usize_unsuffixed(self.param_args.len());
        let pos_arg_len = Literal::usize_unsuffixed(self.positional_args.len());

        let syntax_constant = {
            let self_ident = self.to_token();
            quote! {
                const SYNTAX: crate::command::ServiceSyntax<#param_arg_len, #pos_arg_len> = #self_ident;
            }
        };

        let syntax_text_constant = {
            let text = Literal::string(&format!("{}", &self));
            quote! {
                const SYNTAX_TEXT: &'static str = #text;
            }
        };

        quote! {
            #result_struct

            impl #result_struct_type {
                fn parse<T>(mut param: ::std::iter::Peekable<T>)
                    -> crate::error::BotGroupResult<#result_struct_type>
                where
                    T: Iterator<Item = crate::server::ParamItem>
                {
                    let mut #result_ident = crate::error::BotErrorGroup::new();
                    #(#pos_args)*
                    #param_args
                    if #result_ident.as_ref().is_empty() {
                        let mut #result_val_ident: #result_struct_type = #result_struct_type::default();
                        #assignment
                        Ok(#result_val_ident)
                    } else {
                        Err(#result_ident)
                    }
                }
            }

            impl crate::command::BotService<#param_arg_len, #pos_arg_len> for #result_struct_type {
                #syntax_constant
                #syntax_text_constant
                type RESULT = #result_struct_type;

                fn get_result<T>(param: ::std::iter::Peekable<T>)
                    -> crate::error::BotGroupResult<#result_struct_type>
                where
                    T: Iterator<Item = crate::server::ParamItem>
                {
                    Self::parse(param)
                }
            }
        }
    }

    fn to_token(&self) -> TokenStream {
        let title_lit = Literal::string(&self.title);
        let prefix_lit = Literal::string(&self.arg_prefix);
        let desc_lit = Literal::string(&self.description);
        let pos_args = self.positional_args.iter().map(PositionalArg::to_token);
        let param_args = self.param_args.iter().map(ParamArg::to_token);
        quote! {
            crate::command::ServiceSyntax {
                title: #title_lit,
                arg_prefix: #prefix_lit,
                description: #desc_lit,
                positional_args: [#(#pos_args),*],
                param_args: [#(#param_args),*],
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParamArg {
    pub alias: Option<Vec<String>>,
    pub arg_name: String,
    pub arg_type: ArgType,
    pub default: Option<ArgValue>,
    pub description: String,
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
            alias = if let Some(val) = &self.alias {
                format!("\n                    Alias: {}", val.join(", "))
            } else {
                "".to_string()
            }
        )
    }
}

impl ParamArg {
    fn to_token(&self) -> TokenStream {
        let default = if let Some(d) = &self.default {
            let d = d.to_token();
            quote! { Some(#d)}
        } else {
            quote! { None }
        };
        let alias = if let Some(d) = &self.alias {
            let d = d.iter().map(|s| Literal::string(s));
            quote! { Some(&[#(#d),*]) }
        } else {
            quote! { None }
        };
        let lit_name = Literal::string(&self.arg_name);
        let arg_type = self.arg_type.to_token();
        let lit_desc = Literal::string(&self.description);
        quote! {
            crate::command::ParamArg {
                arg_name: #lit_name,
                arg_type: #arg_type,
                description: #lit_desc,
                default: #default,
                alias: #alias,
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PositionalArg {
    pub arg_name: String,
    pub arg_type: ArgType,
    pub description: String,
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

impl PositionalArg {
    fn to_token(&self) -> TokenStream {
        let lit_name = Literal::string(&self.arg_name);
        let arg_type = self.arg_type.to_token();
        let lit_desc = Literal::string(&self.description);
        quote! {
            crate::command::PositionalArg {
                arg_name: #lit_name,
                arg_type: #arg_type,
                description: #lit_desc,
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum ArgValue {
    AnyText(String),
    EnumText(String),
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

impl ArgValue {
    pub fn to_value_token(&self) -> TokenStream {
        match self {
            Self::AnyText(t) | Self::EnumText(t) => {
                let t = Literal::string(t);
                quote! { #t }
            }
            Self::Float(t) => {
                let t = Literal::f64_suffixed(*t);
                quote! { #t }
            }
            Self::Int(t) => {
                let t = Literal::i64_suffixed(*t);
                quote! { #t }
            }
            Self::Boolean(t) => {
                if *t {
                    quote! { true }
                } else {
                    quote! { false }
                }
            }
        }
    }

    pub fn to_token(&self) -> TokenStream {
        match self {
            Self::AnyText(s) => {
                let lit_str = Literal::string(s);
                quote! { crate::command::ArgValue::AnyText(#lit_str.to_string()) }
            }
            Self::EnumText(s) => {
                let lit_str = Literal::string(s);
                quote! { crate::command::ArgValue::EnumText(#lit_str) }
            }
            Self::Float(v) => {
                let lit_val = Literal::f64_suffixed(*v);
                quote! { crate::command::ArgValue::Float(#lit_val) }
            }
            Self::Int(v) => {
                let lit_val = Literal::i64_suffixed(*v);
                quote! { crate::command::ArgValue::Int(#lit_val) }
            }
            Self::Boolean(b) => {
                if *b {
                    quote! { crate::command::ArgValue::Boolean(true) }
                } else {
                    quote! { crate::command::ArgValue::Boolean(false) }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum ArgType {
    AnyText,
    EnumText(Vec<String>),
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

struct ArgTypeParser<'a> {
    val_ident: &'a Ident,
    result_ident: &'a Ident,
    name_lit: &'a Literal,
}

impl ArgType {
    fn as_return_type(&self) -> TokenStream {
        match self {
            ArgType::AnyText => quote! { ::std::string::String },
            ArgType::EnumText(_) => quote! { &'static str },
            ArgType::Float => quote! { f64 },
            ArgType::Int => quote! { i64 },
            ArgType::Boolean => quote! { bool },
        }
    }
    fn to_parser(&self, val: ArgTypeParser<'_>) -> TokenStream {
        let val_ident = val.val_ident;
        let result_ident = val.result_ident;
        let name_lit = val.name_lit;
        match self {
            ArgType::AnyText => quote! {
                if let crate::server::ParamItem::Text(t) = &#val_ident {
                    Some(t.to_owned())
                } else {
                    #result_ident.push(crate::error::BotError::Syntax {
                        found: Some(format!("{:?}", #val_ident)),
                        expected: Some(#name_lit.to_string()),
                        note: Some("参数类型错误".to_string()),
                    });
                    None
                }
            },
            ArgType::EnumText(enums) => {
                assert!(!enums.is_empty(), "enum text is empty");
                let pattern = enums.iter().map(|s| {
                    let _lit = Literal::string(s);
                    quote! {
                        #_lit => Some(#_lit),
                    }
                });
                quote! {
                    if let crate::server::ParamItem::Text(t) = &#val_ident {
                        match t.as_str() {
                            #(#pattern)*
                            _ => {
                                #result_ident.push(crate::error::BotError::Syntax {
                                    found: Some(format!("{:?}", #val_ident)),
                                    expected: Some(#name_lit.to_string()),
                                    note: Some("枚举参数非法值".to_string()),
                                });
                                None
                            }
                        }
                    } else {
                        #result_ident.push(crate::error::BotError::Syntax {
                            found: Some(format!("{:?}", #val_ident)),
                            expected: Some(#name_lit.to_string()),
                            note: Some("参数类型错误".to_string()),
                        });
                        None
                    }
                }
            }
            ArgType::Float => quote! {
                match &#val_ident {
                    crate::server::ParamItem::Text(_t) => if let Ok(_f) = _t.parse::<f64>() {
                        Some(_f)
                    } else {
                        #result_ident.push(crate::error::BotError::Syntax {
                            found: Some(format!("{}", #val_ident)),
                            expected: Some(#name_lit.to_string()),
                            note: Some("无法解析入参".to_string()),
                        });
                        None
                    },
                    crate::server::ParamItem::At(_at) => Some(_at.to_owned() as f64),
                }
            },
            ArgType::Int => quote! {
                match &#val_ident {
                    crate::server::ParamItem::Text(_t) => if let Ok(_f) = _t.parse::<i64>() {
                        Some(_f)
                    } else {
                        #result_ident.push(crate::error::BotError::Syntax {
                            found: Some(format!("{}", #val_ident)),
                            expected: Some(#name_lit.to_string()),
                            note: Some("无法解析入参".to_string()),
                        });
                        None
                    },
                    crate::server::ParamItem::At(_at) => Some(_at.to_owned() as i64),
                }
            },
            ArgType::Boolean => quote! {
                match #val_ident {
                    crate::server::ParamItem::Text(_t) => match _t.to_lowercase().as_str() {
                        "true" | "t" | "yes" | "y" => Some(true),
                        "false" | "f" | "no" | "n" => Some(false),
                        _ => {
                        #result_ident.push(crate::error::BotError::Syntax {
                            found: Some(format!("{}", #val_ident)),
                            expected: Some(#name_lit.to_string()),
                            note: Some("无法解析入参".to_string()),
                        });
                        None
                    }
                    },
                    _ => {
                        #result_ident.push(crate::error::BotError::Syntax {
                            found: Some(format!("{}", #val_ident)),
                            expected: Some(#name_lit.to_string()),
                            note: Some("无法解析入参".to_string()),
                        });
                        None
                    }
                }
            },
        }
    }

    fn as_value_type(&self) -> TokenStream {
        match self {
            Self::AnyText => quote! { ::std::string::String },
            Self::EnumText(_) => quote! { &'static str },
            Self::Float => quote! { f64 },
            Self::Int => quote! { i64 },
            Self::Boolean => quote! { bool },
        }
    }

    fn to_token(&self) -> TokenStream {
        match self {
            Self::AnyText => quote! { crate::command::ArgType::AnyText },
            Self::EnumText(val) => quote! { crate::command::ArgType::EnumText(&[#(#val),*]) },
            Self::Float => quote! { crate::command::ArgType::Float },
            Self::Int => quote! { crate::command::ArgType::Int },
            Self::Boolean => quote! { crate::command::ArgType::Boolean },
        }
    }
}

fn print_fs_err<T: Default>(fp: impl AsRef<std::path::Path>) -> Box<dyn Fn(std::io::Error) -> T> {
    let full_path = if fp.as_ref().is_absolute() {
        Ok(fp.as_ref().to_owned())
    } else {
        fp.as_ref().canonicalize()
    };
    let to_find = std::env::current_dir().map(|val| val.join(fp.as_ref()));
    match full_path {
        Ok(val) => Box::new(move |e| {
            panic!(
                "\nerror: {:?}\nfile path: {:?}\nto find: {:?}",
                e, val, to_find
            )
        }),
        Err(err) => Box::new(move |e| {
            panic!(
                "\nerror: {:?}\nfile path error: {:?}\nto find: {:?}",
                e, err, to_find
            )
        }),
    }
}

pub fn arg_producing(attr: RawTokenStream, input: RawTokenStream) -> RawTokenStream {
    let lit_fp = syn::parse_macro_input!(attr as syn::LitStr);
    let s: ItemStruct = syn::parse_macro_input!(input as syn::ItemStruct);
    let struct_ident = s.ident.clone();
    let fp = lit_fp.value();
    let data: ServiceSyntax =
        serde_json::from_slice(&std::fs::read(&fp).unwrap_or_else(print_fs_err(&fp))).unwrap();
    let data = data.process_mixin();
    // std::fs::write(
    //     format!("./macro-expand/test.output.{}.rs", s.ident),
    //     format!("{}", &data.to_parser(&struct_ident)),
    // )
    // .unwrap();
    RawTokenStream::from(data.to_parser(&struct_ident))
}
