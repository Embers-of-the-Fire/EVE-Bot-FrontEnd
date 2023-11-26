use crate::constant::eve::server::Server;
use crate::error::{BotError, BotErrorGroup, BotGroupResult};
use crate::metadata::{IMAGE_DIRECTORY, LOCAL_EVE_SERVICE_PORT};
use crate::utils::fetch::{download_image, random_filename};
use crate::{build_single_image, fetch_type};
use serde::{Deserialize, Serialize};

#[test]
fn test_blp_mat_image() {
    use crate::command::BotService;
    use crate::server::ParamItem;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    println!("{}\n\n", BlpMaterial::SYNTAX);
    runtime.block_on(async {
        let res = BlpMaterial::parse(
            [
                ParamItem::Text("帕拉丁级蓝图".into()),
                ParamItem::Text("pattern".into()),
                ParamItem::Text("f".into()),
                ParamItem::Text("mt".into()),
                ParamItem::Text("3".into()),
                ParamItem::Text("em".into()),
                ParamItem::Text("10".into()),
                ParamItem::Text("exp".into()),
                ParamItem::Text("true".into()),
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

#[evebot_proc_macro::create_syntax("evebot-gocq-wrapper/syntax/command/blp_mat.json")]
pub struct BlpMaterial;

impl BlpMaterial {
    /// # Syntax
    ///
    /// ```
    /// eve blp mat <item-name>
    ///     (manu_mat_level <int>)?
    ///     (manu_time_level <int>)?
    ///     (extra_mat <float>)?
    ///     (extra_time <float>)?
    ///     (expand <bool>)?
    ///     (sql <bool>)?
    /// ```
    ///
    /// - `item-name`: The name of the blueprint.
    /// - `manu_mat_level`: The level of all blueprints' material level. Value range: `[0, 10]`. Default value: 0.
    /// - `manu_time_level`: The level of all blueprints' time level. Value range: `[0, 10]`. Default value: 0.
    /// - `extra_mat`: The extra factor of all blueprints' material. Note: This is a percentage.
    /// Value range: `[0.0, ...)`. Default value: 0.0.
    /// - `extra_time`: The extra factor of all blueprints' time. Note: This is a percentage.
    /// Value range: `[0.0, ...)`. Default value: 0.0.
    /// - `expand`: Whether to expand the blueprint. Default value: false.
    /// - `sql`: Whether to use manual SQL pattern. Default value: false.
    pub async fn get_content(&self) -> BotGroupResult<serde_json::Value> {
        let mut err_group = BotErrorGroup::new();
        let type_item = fetch_type! {
            pattern: &self.pattern,
            type_name: &self.type_name,
            error: err_group,
        };

        let value = BlueprintFactor::checked(
            self.manu_mat_level as u8,
            self.manu_time_level as u8,
            self.extra_mat,
            self.extra_time,
        )
        .map_err(|mut e| err_group.as_mut().append(e.as_mut()))
        .ok();

        if !err_group.as_ref().is_empty() || type_item.is_none() {
            return Err(err_group);
        }

        let type_item = type_item.unwrap();
        let resp = reqwest::ClientBuilder::new()
            .build()?
            .post(format!(
                "http://localhost:{}/blueprint/{}/{}/image/",
                LOCAL_EVE_SERVICE_PORT,
                type_item.type_id,
                if self.expand { "recursive" } else { "plain" }
            ))
            .json(&value)
            .send()
            .await?;
        let file_name = random_filename();
        download_image(
            resp,
            format!("{}{}.png", IMAGE_DIRECTORY, file_name).as_ref(),
        )
        .await?;

        Ok(build_single_image! {
            format!("file:///{}{}.png", IMAGE_DIRECTORY, file_name)
        })
    }
}

#[evebot_proc_macro::create_syntax("./evebot-gocq-wrapper/syntax/command/blp_mat_price.json")]
pub struct BlpMaterialPrice;

impl BlpMaterialPrice {
    /// # Syntax
    ///
    /// ```
    /// eve blp mat <item-name>
    ///     (manu_mat_level <int>)?
    ///     (manu_time_level <int>)?
    ///     (extra_mat <float>)?
    ///     (extra_time <float>)?
    ///     (expand <bool>)?
    ///     (sql <bool>)?
    ///     (server <enum>)?
    /// ```
    ///
    /// - `item-name`: The name of the blueprint.
    /// - `manu_mat_level`: The level of all blueprints' material level. Value range: `[0, 10]`. Default value: 0.
    /// - `manu_time_level`: The level of all blueprints' time level. Value range: `[0, 10]`. Default value: 0.
    /// - `extra_mat`: The extra factor of all blueprints' material. Note: This is a percentage.
    /// Value range: `[0.0, ...)`. Default value: 0.0.
    /// - `extra_time`: The extra factor of all blueprints' time. Note: This is a percentage.
    /// Value range: `[0.0, ...)`. Default value: 0.0.
    /// - `expand`: Whether to expand the blueprint. Default value: false.
    /// - `sql`: Whether to use manual SQL pattern. Default value: false.
    /// - `server`: Which server to use. Possible value: 'se', 'tq'. Default value: 'se'.
    pub async fn get_content(&self) -> BotGroupResult<serde_json::Value> {
        let mut err_group = BotErrorGroup::new();

        let type_item = fetch_type! {
            pattern: &self.pattern,
            type_name: &self.type_name,
            error: err_group,
        };

        let value = BlueprintFactor::checked(
            self.manu_mat_level as u8,
            self.manu_time_level as u8,
            self.extra_mat,
            self.extra_time,
        )
        .map_err(|mut e| err_group.as_mut().append(e.as_mut()))
        .ok();

        let server = Server::parse_from(self.server)
            .map_err(|e| err_group.push(e))
            .ok();

        if !err_group.as_ref().is_empty() || type_item.is_none() {
            return Err(err_group);
        }

        let type_item = type_item.unwrap();
        let value = value.unwrap();
        let server = server.unwrap();

        let material: serde_json::Value = reqwest::ClientBuilder::new()
            .build()?
            .post(format!(
                "http://localhost:{}/blueprint/{}/{}/",
                LOCAL_EVE_SERVICE_PORT,
                type_item.type_id,
                if self.expand { "recursive" } else { "plain" }
            ))
            .json(&value)
            .send()
            .await?
            .json()
            .await?;
        let price: serde_json::Value = reqwest::ClientBuilder::new()
            .build()?
            .post(format!(
                "http://localhost:{}/blueprint/market/?s={}",
                LOCAL_EVE_SERVICE_PORT,
                server.as_api_like()
            ))
            .json(&material)
            .send()
            .await?
            .json()
            .await?;
        let image_resp = reqwest::ClientBuilder::new()
            .build()?
            .post(format!(
                "http://localhost:{}/blueprint/market/image/",
                LOCAL_EVE_SERVICE_PORT
            ))
            .json(&price)
            .send()
            .await?;

        let file_name = random_filename();

        download_image(
            image_resp,
            format!("{}{}.png", IMAGE_DIRECTORY, file_name).as_ref(),
        )
        .await?;

        Ok(build_single_image! {
            format!("file:///{}{}.png", IMAGE_DIRECTORY, file_name)
        })
    }
}

#[test]
fn test_blp_mat_price_image() {
    use crate::command::BotService;
    use crate::server::ParamItem;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    println!("{}\n\n", BlpMaterialPrice::SYNTAX);
    runtime.block_on(async {
        let res = BlpMaterialPrice::parse(
            [
                ParamItem::Text("帕拉丁级蓝图".into()),
                ParamItem::Text("pattern".into()),
                ParamItem::Text("f".into()),
                ParamItem::Text("expand".into()),
                ParamItem::Text("yes".into()),
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BlueprintFactor {
    manu_material: u8,
    manu_time: u8,
    extra_material: f64,
    extra_time: f64,
}

impl BlueprintFactor {
    #[inline]
    pub fn new(manu_material: u8, manu_time: u8, extra_material: f64, extra_time: f64) -> Self {
        Self {
            manu_time,
            manu_material,
            extra_material,
            extra_time,
        }
    }

    pub fn check(&self) -> Option<BotErrorGroup> {
        let mut err_group = BotErrorGroup::new();
        if self.manu_material > 10 {
            err_group.push(BotError::Syntax {
                found: Some(format!("{:?}", self.manu_material)),
                expected: Some("Value out of range.".into()),
                note: Some("Expect value in [0, 10]".into()),
            })
        }
        if self.manu_time > 10 {
            err_group.push(BotError::Syntax {
                found: Some(format!("{:?}", self.manu_time)),
                expected: Some("Value out of range.".into()),
                note: Some("Expect value in [0, 10]".into()),
            })
        }
        if self.extra_material < 0.0 {
            err_group.push(BotError::Syntax {
                found: Some(format!("{:?}", self.extra_material)),
                expected: Some("Value out of range.".into()),
                note: Some("Expect value in [0, ...)".into()),
            })
        }
        if self.extra_time < 0.0 {
            err_group.push(BotError::Syntax {
                found: Some(format!("{:?}", self.extra_time)),
                expected: Some("Value out of range.".into()),
                note: Some("Expect value in [0, ...)".into()),
            })
        }
        if err_group.as_ref().is_empty() {
            None
        } else {
            Some(err_group)
        }
    }

    #[inline]
    pub fn checked(
        manu_material: u8,
        manu_time: u8,
        extra_material: f64,
        extra_time: f64,
    ) -> BotGroupResult<Self> {
        let val = Self::new(manu_material, manu_time, extra_material, extra_time);
        if let Some(v) = val.check() {
            Err(v)
        } else {
            Ok(val)
        }
    }
}
