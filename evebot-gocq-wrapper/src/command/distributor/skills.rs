use crate::build_single_image;
use crate::error::BotGroupResult;
use crate::metadata::{IMAGE_DIRECTORY, LOCAL_EVE_SERVICE_PORT};

#[evebot_proc_macro::create_syntax("evebot-gocq-wrapper/syntax/command/skill_item.json")]
pub struct Skill;

#[test]
fn test_skill_image() {
    use crate::command::BotService;
    use crate::server::ParamItem;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    println!("{}\n\n", Skill::SYNTAX);
    runtime.block_on(async {
        let res = Skill::parse(
            [
                ParamItem::Text("帕拉丁级".into()),
                ParamItem::Text("pattern".into()),
                ParamItem::Text("f".into()),
            ]
            .into_iter()
            .peekable(),
        );
        match res {
            Ok(res) => match res.get_content().await {
                Ok(image) => println!("{:#}", image),
                Err(err) => println!("{}", err),
            },
            Err(err) => println!("{}", err),
        }
    });
}

impl Skill {
    /// # Syntax
    ///
    /// ```
    /// eve item skill <item-name> (sql <bool>)?
    /// ```
    ///
    /// - `item-name`: The name of the item.
    /// - `sql`: Whether to use manual SQL pattern. Default value: false.
    pub async fn get_content(&self) -> BotGroupResult<serde_json::Value> {
        let type_item = crate::utils::fetch::TypeIDFetch::type_from(&self.pattern)(&self.type_name)?
            .get()
            .await?;
        let img_raw = reqwest::get(format!(
            "http://localhost:{}/skill/prereq/{}/image/",
            LOCAL_EVE_SERVICE_PORT, type_item.type_id,
        ))
        .await?;
        let file_name = crate::utils::fetch::random_filename();
        crate::utils::fetch::download_image(
            img_raw,
            format!("{}{}.png", IMAGE_DIRECTORY, file_name).as_ref(),
        )
        .await?;
        Ok(build_single_image! {format!("file:///{}{}.png", IMAGE_DIRECTORY, file_name)})
    }
}
