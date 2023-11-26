use crate::command::BotService;
use crate::error::BotGroupResult;
use crate::metadata::BOT_UID;
use crate::server::ParamItem;

#[test]
fn test_distribute() {
    use crate::server::MessageData;
    use serde_json::json;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let text = MessageData(json! {[
            {
                "type": "text",
                "data": {
                    "text": "eve blp mat 勒维亚坦级蓝图 pattern fuzzy expand true"
                }
            },
        ]});
        let res = distribute(text.into_messages().unwrap())
            .await
            .unwrap_or_else(|| {
                Ok(json! {[{
                    "type": "text",
                    "data": {
                        "text": "<NO-DATA>"
                    }
                }]})
            })
            .unwrap_or_else(|e| {
                json! {[{
                    "type": "text",
                    "data": {
                        "text": e.to_string()
                    }
                }]}
            });
        println!(
            "{}",
            res.get(0)
                .and_then(|s| s.get("data"))
                .and_then(|s| s.get("text").or_else(|| s.get("file")))
                .and_then(|s| s.as_str())
                .unwrap()
        );
    });
}

pub async fn distribute(
    param: impl Iterator<Item = ParamItem>,
) -> Option<BotGroupResult<serde_json::Value>> {
    let mut param = param.peekable();
    // Prefix
    match &param.next()? {
        ParamItem::Text(_t) if _t.as_str() != "eve" => return None,
        ParamItem::At(a) => {
            if *a != BOT_UID {
                return None;
            } else if param.peek() == Some(&ParamItem::Text("eve".into())) {
                param.next();
            }
        }
        _ => {}
    }

    evebot_proc_macro::create_distributor!(
        r"D:\WBH\rust\evebot\evebot-gocq-wrapper\src\command\distributor\distributor.json"
    )
}
