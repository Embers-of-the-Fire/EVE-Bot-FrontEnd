use itertools::Itertools;
use proc_macro::TokenStream as RawTokenStream;
use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use syn::{ExprPath, LitStr};

type IResult<T> = Result<T, TokenStream>;

pub fn create_distributor(input: RawTokenStream) -> RawTokenStream {
    match create_dis(input) {
        Ok(token) | Err(token) => {
            // std::fs::write(
            //     "./macro-expand/test.output.distributor.rs",
            //     format!(
            //         "{}",
            //         quote! { pub fn test() {
            //             #token
            //         } }
            //     ),
            // )
            // .unwrap();
            token.into()
        }
    }
}

fn create_dis(input: RawTokenStream) -> IResult<TokenStream> {
    let lit_fp = syn::parse::<LitStr>(input).map_err(|e| e.to_compile_error())?;
    let data = get_config_data(&lit_fp);
    let param_ident = Ident::new("param", Span::call_site());
    let grp_it = data.iter().map(|s| s.to_pattern(&param_ident));
    Ok(quote! {
        match &#param_ident.next()? {
            crate::server::ParamItem::Text(_t) => match _t.as_str() {
                #(#grp_it)*
                _ => None
            },
            _ => None
        }
    })
}

fn get_config_data(lit_fp: &LitStr) -> Vec<SubGroup> {
    let fp = lit_fp.value();
    let content = std::fs::read(fp).unwrap();
    serde_json::from_slice(&content).unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubGroup {
    #[serde(alias = "path-ident")]
    path_ident: String,
    #[serde(alias = "path-alias", default)]
    path_alias: Option<Vec<String>>,
    description: String,
    #[serde(alias = "group-name")]
    group_name: String,
    #[serde(default)]
    subcommand: Option<Vec<SubCommand>>,
    #[serde(default)]
    subgroup: Option<Vec<SubGroup>>,
}

impl SubGroup {
    fn to_pattern(&self, param_ident: &Ident) -> TokenStream {
        let help_text = Literal::string(&self.to_help());
        let subcommand_matcher = {
            let _it = self
                .subcommand
                .iter()
                .flatten()
                .map(|s| match s.to_pattern(param_ident) {
                    Ok(v) | Err(v) => v,
                });
            quote! {
                #(#_it)*
            }
        };
        let subgroup_matcher = {
            let _it = self
                .subgroup
                .iter()
                .flatten()
                .map(|s| s.to_pattern(param_ident));
            quote! {
                #(#_it)*
            }
        };
        let path_pattern = {
            let _list = [self.path_ident.to_owned()];
            let val = _list
                .iter()
                .chain(self.path_alias.iter().flatten())
                .map(|s| Literal::string(s));
            quote! {
                #(#val)|*
            }
        };

        quote! {
            #path_pattern => match #param_ident.next() {
                Some(_ident) => match _ident {
                    crate::server::ParamItem::Text(_t) => match _t.as_str() {
                        #subcommand_matcher
                        #subgroup_matcher
                        _ => Some(Ok(crate::build_single_text!(#help_text)))
                    },
                    _ => None
                },
                _ => Some(Ok(crate::build_single_text!(#help_text)))
            },
        }
    }

    fn to_help(&self) -> String {
        let commands_text: String = match &self.subcommand {
            Some(_c) if !_c.is_empty() => {
                let string = "Sub Commands:\n".to_owned();
                string
                    + _c.iter()
                        .map(|_val| format!("{:<10}{}", _val.path_ident, _val.description))
                        .join("\n")
                        .as_str()
                    + "\n"
            }
            _ => String::new(),
        };
        let groups_text: String = match &self.subgroup {
            Some(_c) if !_c.is_empty() => {
                let string = "Sub Groups:\n".to_owned();
                string
                    + _c.iter()
                        .map(|_val| {
                            format!(
                                "{:<10}{:<10}{}",
                                _val.path_ident, _val.group_name, _val.description
                            )
                        })
                        .join("\n")
                        .as_str()
                    + "\n"
            }
            _ => String::new(),
        };

        format!(
            "{title}  <{ident}>\n{commands}{groups}",
            title = &self.group_name,
            ident = &self.path_ident,
            commands = commands_text,
            groups = groups_text,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SubCommand {
    #[serde(alias = "path-ident")]
    path_ident: String,
    #[serde(alias = "path-alias", default)]
    path_alias: Option<Vec<String>>,
    #[serde(alias = "structure-path")]
    structure_path: String,
    #[serde(alias = "no-help", default)]
    no_help: bool,
    description: String,
}

impl SubCommand {
    fn to_pattern(&self, param_ident: &Ident) -> IResult<TokenStream> {
        let raw_path = TokenStream::from_str(&self.structure_path).unwrap();
        let structure_path =
            syn::parse::<ExprPath>(raw_path.into()).map_err(|e| e.to_compile_error())?;
        let path_pattern = [self.path_ident.to_owned()]
            .into_iter()
            .chain(self.path_alias.iter().flatten().cloned())
            .map(|s| Literal::string(&s));

        if self.no_help {
            Ok(quote! {
                #(#path_pattern)|* => crate::get_content!(#structure_path::get_result(#param_ident))
            })
        } else {
            Ok(quote! {
                #(#path_pattern)|* => Some(if #param_ident.peek().is_some() {
                    crate::get_content!(#structure_path::get_result(#param_ident))
                } else {
                    Ok(crate::build_single_text!(#structure_path::SYNTAX_TEXT))
                }),
            })
        }
    }
}
