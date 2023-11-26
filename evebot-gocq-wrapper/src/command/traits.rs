use crate::command::syntax::ServiceSyntax;
use crate::error::BotGroupResult;
use crate::server::ParamItem;

/// # Special Agreement
///
/// All structures implemented this should also implement an asynchronous method called `get_content`.
///
/// The signature should be:
///
/// ```
/// async fn get_content(&self) -> BotGroupResult<impl serde::Serialize> {
///     ...
/// }
/// ```
pub trait BotService<const PARAM_ARG_LEN: usize, const POS_ARG_LEN: usize> {
    type RESULT;
    const SYNTAX: ServiceSyntax<PARAM_ARG_LEN, POS_ARG_LEN>;
    const SYNTAX_TEXT: &'static str = "";

    fn get_result<T>(param: std::iter::Peekable<T>) -> BotGroupResult<Self::RESULT>
    where
        T: Iterator<Item = ParamItem>;
}
