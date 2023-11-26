use crate::constant::eve::server::Server;
use crate::error::{BotErrorGroup, BotGroupResult};
use crate::metadata::LOCAL_EVE_SERVICE_PORT;
use crate::utils::numeric::format_price;
use crate::{build_single_text, fetch_type};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MarketPrice {
    buy: f64,
    sell: f64,
    medium: f64,
}

#[test]
fn test_jita_price() {
    use crate::server::ParamItem;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let res = JitaPrice::parse(
            [
                ParamItem::Text("三%金".into()),
                ParamItem::Text("pattern".into()),
                ParamItem::Text("m".into()),
            ]
            .into_iter()
            .peekable(),
        );
        match res {
            Ok(res) => match res.get_content().await {
                Ok(price) => println!("{:#}", price),
                Err(err) => println!("{}", err),
            },
            Err(err) => println!("{}", err),
        }
    });
}

#[evebot_proc_macro::create_syntax("./evebot-gocq-wrapper/syntax/command/market_jita.json")]
pub struct JitaPrice;

impl JitaPrice {
    /// # Syntax
    ///
    /// ```
    /// eve market jita <item-name> (server <server>)? (sql <bool>)?
    /// ```
    ///
    /// - `item-name`: The name of the item.
    /// - `server`: The name of the server. Possible values: 'se', 'tq'. Default value: 'se'.
    /// - `sql`: Whether to use manual SQL pattern. Default value: false.
    pub async fn get_content(&self) -> BotGroupResult<serde_json::Value> {
        let mut err_group = BotErrorGroup::new();
        let server = match Server::parse_from(self.server) {
            Ok(val) => Some(val),
            Err(err) => {
                err_group.push(err);
                None
            }
        };
        let type_item = fetch_type! {
            pattern: &self.pattern,
            type_name: &self.type_name,
            error: err_group,
        };
        if let Some(server) = server {
            let price: Option<MarketPrice> = if let Some(type_item) = &type_item {
                let resp = reqwest::get(format!(
                    "http://localhost:{}/market/jita/type/{}/{}/",
                    LOCAL_EVE_SERVICE_PORT,
                    server.as_api_like(),
                    type_item.type_id
                ))
                .await;
                match resp {
                    Ok(resp) => match resp.json().await {
                        Ok(js) => Some(js),
                        Err(err) => {
                            err_group.push(err.into());
                            None
                        }
                    },
                    Err(err) => {
                        err_group.push(err.into());
                        None
                    }
                }
            } else {
                None
            };
            match (type_item, price) {
                (Some(type_item), Some(price)) => Ok(build_single_text!(format!(
                    r"物品价格（{}）  {}
收单：{}
卖单：{}
中位价：{}",
                    server.as_readable(),
                    type_item.type_name,
                    format_price(price.buy),
                    format_price(price.sell),
                    format_price(price.medium)
                ))),
                _ => Err(err_group),
            }
        } else {
            Err(err_group)
        }
    }
}
