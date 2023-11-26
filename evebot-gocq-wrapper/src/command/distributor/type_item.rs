use crate::build_single_text;
use crate::error::BotGroupResult;
use crate::metadata::LOCAL_EVE_SERVICE_PORT;
use crate::utils::fetch::{TypeIDFetch, TypeItem};

#[test]
fn test_jita_price() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let res: TypeItem = TypeIDFetch::Absolute("三钛合金").get().await.unwrap();
        // println!("{:?}", res);
        assert_eq!(
            res,
            TypeItem {
                type_id: 34,
                type_name: "三钛合金".into(),
                published: true
            }
        );
        let res: TypeItem = TypeIDFetch::Fuzzy("三钛").get().await.unwrap();
        // println!("{:?}", res);
        assert_eq!(
            res,
            TypeItem {
                type_id: 34,
                type_name: "三钛合金".into(),
                published: true
            }
        );
        let res: TypeItem = TypeIDFetch::Manual("三%合%").get().await.unwrap();
        // println!("{:?}", res);
        assert_eq!(
            res,
            TypeItem {
                type_id: 34,
                type_name: "三钛合金".into(),
                published: true
            }
        );
    });
}

#[evebot_proc_macro::create_syntax("evebot-gocq-wrapper/syntax/command/type_fetch_id.json")]
pub struct TypeFetchId;

impl TypeFetchId {
    pub async fn get_content(&self) -> BotGroupResult<serde_json::Value> {
        let type_item: TypeItem = reqwest::get(format!(
            "http://localhost:{}/types/{}/",
            LOCAL_EVE_SERVICE_PORT, self.type_id
        ))
        .await?
        .json()
        .await?;
        let text = format!(
            "物品ID：{}\n物品名称：{}\n是否公开：{}",
            type_item.type_id,
            type_item.type_name,
            if type_item.published { "是" } else { "否" }
        );
        Ok(build_single_text!(text))
    }
}

#[test]
fn test_type_getter_id() {
    use crate::command::BotService;
    use crate::server::ParamItem;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    println!("{}\n\n", TypeFetchId::SYNTAX);
    runtime.block_on(async {
        let res = TypeFetchId::parse([ParamItem::Text("34".into())].into_iter().peekable());
        match res {
            Ok(res) => match res.get_content().await {
                Ok(val) => println!("{:#}", val),
                Err(err) => println!("{}", err),
            },
            Err(err) => println!("{}", err),
        }
    });
}

#[evebot_proc_macro::create_syntax("evebot-gocq-wrapper/syntax/command/type_fetch_name.json")]
pub struct TypeFetchName;

impl TypeFetchName {
    pub async fn get_content(&self) -> BotGroupResult<serde_json::Value> {
        let type_item: TypeItem = TypeIDFetch::type_from(&self.pattern)(&self.type_name)?
            .get()
            .await?;
        let text = format!(
            "物品ID：{}\n物品名称：{}\n是否公开：{}",
            type_item.type_id,
            type_item.type_name,
            if type_item.published { "是" } else { "否" }
        );
        Ok(build_single_text!(text))
    }
}

#[test]
fn test_type_getter_name() {
    use crate::command::BotService;
    use crate::server::ParamItem;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    println!("{}\n\n", TypeFetchName::SYNTAX);
    runtime.block_on(async {
        let res = TypeFetchName::parse(
            [
                ParamItem::Text("三钛%金".into()),
                ParamItem::Text("pattern".into()),
                ParamItem::Text("m".into()),
            ]
            .into_iter()
            .peekable(),
        );
        match res {
            Ok(res) => match res.get_content().await {
                Ok(val) => println!("{:#}", val),
                Err(err) => println!("{}", err),
            },
            Err(err) => println!("{}", err),
        }
    });
}
